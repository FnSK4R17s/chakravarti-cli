import Docker from 'dockerode';
import chalk from 'chalk';
import path from 'path';

const docker = new Docker();

export interface ContainerConfig {
    executorName: string;
    worktreePath: string;
    image: string;
    ports?: number[];
    env?: Record<string, string>;
}

export interface Container {
    id: string;
    name: string;
    status: string;
}

/**
 * Build executor Docker image
 */
export async function buildExecutorImage(projectPath: string): Promise<void> {
    const imageName = 'chakravarti-executor:latest';

    console.log(chalk.cyan('🐳 Building executor Docker image...'));

    try {
        const stream = await docker.buildImage({
            context: projectPath,
            src: ['Dockerfile.executor', 'package.json', 'package-lock.json']
        }, {
            t: imageName,
            dockerfile: 'Dockerfile.executor'
        });

        await new Promise((resolve, reject) => {
            docker.modem.followProgress(stream, (err, res) =>
                err ? reject(err) : resolve(res)
            );
        });

        console.log(chalk.green(`✓ Built image: ${imageName}`));
    } catch (error: any) {
        throw new Error(`Failed to build executor image: ${error.message}`);
    }
}

/**
 * Create container for executor
 */
export async function createContainer(config: ContainerConfig): Promise<Container> {
    const containerName = `chakravarti-${config.executorName}`;

    try {
        // Check if container already exists
        const existing = await getContainer(containerName);
        if (existing) {
            console.log(chalk.yellow(`⚠ Container ${containerName} already exists`));
            return existing;
        }

        // Prepare port bindings
        const portBindings: any = {};
        const exposedPorts: any = {};

        if (config.ports) {
            config.ports.forEach(port => {
                const portKey = `${port}/tcp`;
                exposedPorts[portKey] = {};
                portBindings[portKey] = [{ HostPort: port.toString() }];
            });
        }

        // Create container
        const container = await docker.createContainer({
            name: containerName,
            Image: config.image,
            Tty: true,
            WorkingDir: '/workspace',
            Env: Object.entries(config.env || {}).map(([k, v]) => `${k}=${v}`),
            HostConfig: {
                Binds: [
                    `${config.worktreePath}:/workspace`
                ],
                PortBindings: portBindings,
                NetworkMode: 'chakravarti-network'
            },
            ExposedPorts: exposedPorts
        });

        console.log(chalk.green(`✓ Created container: ${containerName}`));

        return {
            id: container.id,
            name: containerName,
            status: 'created'
        };
    } catch (error: any) {
        throw new Error(`Failed to create container: ${error.message}`);
    }
}

/**
 * Start container
 */
export async function startContainer(containerId: string): Promise<void> {
    try {
        const container = docker.getContainer(containerId);
        await container.start();

        console.log(chalk.green(`✓ Started container: ${containerId.substring(0, 12)}`));
    } catch (error: any) {
        throw new Error(`Failed to start container: ${error.message}`);
    }
}

/**
 * Stop container
 */
export async function stopContainer(containerId: string): Promise<void> {
    try {
        const container = docker.getContainer(containerId);
        await container.stop();

        console.log(chalk.green(`✓ Stopped container: ${containerId.substring(0, 12)}`));
    } catch (error: any) {
        // Ignore if already stopped
        if (!error.message.includes('304')) {
            throw new Error(`Failed to stop container: ${error.message}`);
        }
    }
}

/**
 * Execute command in container
 */
export async function execInContainer(
    containerId: string,
    command: string[]
): Promise<{ stdout: string; stderr: string; exitCode: number }> {
    try {
        const container = docker.getContainer(containerId);

        const exec = await container.exec({
            Cmd: command,
            AttachStdout: true,
            AttachStderr: true
        });

        const stream = await exec.start({ Detach: false });

        let stdout = '';
        let stderr = '';

        return new Promise((resolve, reject) => {
            stream.on('data', (chunk: Buffer) => {
                const str = chunk.toString();
                // Docker multiplexes stdout/stderr
                if (chunk[0] === 1) {
                    stdout += str.substring(8);
                } else if (chunk[0] === 2) {
                    stderr += str.substring(8);
                }
            });

            stream.on('end', async () => {
                const inspect = await exec.inspect();
                resolve({
                    stdout,
                    stderr,
                    exitCode: inspect.ExitCode || 0
                });
            });

            stream.on('error', reject);
        });
    } catch (error: any) {
        throw new Error(`Failed to execute command in container: ${error.message}`);
    }
}

/**
 * Remove container
 */
export async function removeContainer(containerId: string): Promise<void> {
    try {
        const container = docker.getContainer(containerId);
        await container.remove({ force: true });

        console.log(chalk.green(`✓ Removed container: ${containerId.substring(0, 12)}`));
    } catch (error: any) {
        throw new Error(`Failed to remove container: ${error.message}`);
    }
}

/**
 * Get container by name
 */
export async function getContainer(name: string): Promise<Container | null> {
    try {
        const containers = await docker.listContainers({ all: true });
        const found = containers.find(c => c.Names.includes(`/${name}`));

        if (!found) {
            return null;
        }

        return {
            id: found.Id,
            name: name,
            status: found.State
        };
    } catch (error: any) {
        return null;
    }
}

/**
 * List all chakravarti containers
 */
export async function listContainers(): Promise<Container[]> {
    try {
        const containers = await docker.listContainers({ all: true });

        return containers
            .filter(c => c.Names.some(n => n.includes('chakravarti-')))
            .map(c => ({
                id: c.Id,
                name: c.Names[0].replace('/', ''),
                status: c.State
            }));
    } catch (error: any) {
        throw new Error(`Failed to list containers: ${error.message}`);
    }
}

/**
 * Cleanup container
 */
export async function cleanupContainer(containerId: string): Promise<void> {
    try {
        await stopContainer(containerId);
        await removeContainer(containerId);
    } catch (error: any) {
        throw new Error(`Failed to cleanup container: ${error.message}`);
    }
}

/**
 * Check if Docker is available
 */
export async function isDockerAvailable(): Promise<boolean> {
    try {
        await docker.ping();
        return true;
    } catch (error) {
        return false;
    }
}

/**
 * Create Docker network if it doesn't exist
 */
export async function ensureNetwork(networkName: string = 'chakravarti-network'): Promise<void> {
    try {
        const networks = await docker.listNetworks();
        const exists = networks.some(n => n.Name === networkName);

        if (!exists) {
            await docker.createNetwork({ Name: networkName });
            console.log(chalk.green(`✓ Created Docker network: ${networkName}`));
        }
    } catch (error: any) {
        throw new Error(`Failed to ensure network: ${error.message}`);
    }
}
