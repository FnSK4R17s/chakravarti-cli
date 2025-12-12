import chalk from 'chalk';
import ora from 'ora';
import { loadAgentPool } from '../orchestration/executor';
import { exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';
import fs from 'fs';

const execAsync = promisify(exec);

export const upCommand = async (
    options: { path?: string; detached?: boolean } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();

    console.log(chalk.bold('\n🚀 Starting Chakravarti Environment\n'));

    const spinner = ora('Checking project configuration...').start();
    try {
        // 1. Verify this is a Chakravarti project
        let config;
        try {
            config = loadAgentPool(projectPath);
            spinner.succeed(`Project: ${chalk.cyan(config.project.name)}`);
        } catch (error: any) {
            if (error.message.includes('missing planner configuration')) {
                spinner.fail('Agent pool not configured');
                console.log(chalk.yellow('\n⚠ Please run `ckrv setup` to configure your agents first.\n'));
                return;
            }
            throw error;
        }

        // 2. Check Docker availability
        spinner.start('Checking Docker...');
        try {
            await execAsync('docker info');
            spinner.succeed('Docker is available');
        } catch (error) {
            spinner.fail('Docker is not running');
            console.log(chalk.yellow('\n⚠ Please start Docker Desktop and try again\n'));
            return;
        }

        // 3. Start GitLab (if not already running)
        spinner.start('Starting GitLab...');
        try {
            const { stdout: gitlabStatus } = await execAsync(
                'docker ps --filter name=chakravarti-gitlab --format "{{.Status}}"'
            );

            if (gitlabStatus.trim()) {
                spinner.succeed('GitLab is already running');
            } else {
                // Find docker-compose file
                const composePath = await findDockerCompose();
                if (composePath) {
                    await execAsync(`docker-compose -f ${composePath} up -d`);
                    spinner.succeed('GitLab started');
                } else {
                    spinner.warn('GitLab compose file not found - run `ckrv gitlab init` first');
                }
            }
        } catch (error: any) {
            spinner.warn(`GitLab: ${error.message}`);
        }

        // 4. Create Docker network (if not exists)
        spinner.start('Setting up Docker network...');
        try {
            await execAsync('docker network create chakravarti-network 2>/dev/null || true');
            spinner.succeed('Docker network ready');
        } catch (error) {
            spinner.succeed('Docker network exists');
        }

        // 4.5. Start Redis (if not already running)
        spinner.start('Starting Redis...');
        try {
            const { stdout: redisStatus } = await execAsync(
                'docker ps --filter name=chakravarti-redis --format "{{.Status}}"'
            );

            if (redisStatus.trim()) {
                spinner.succeed('Redis is already running');
            } else {
                // Start Redis from compose file
                const composePath = await findDockerCompose();
                if (composePath) {
                    await execAsync(`docker-compose -f ${composePath} up -d redis`);
                    spinner.succeed('Redis started');
                } else {
                    // Start standalone Redis
                    await execAsync(`docker run -d --name chakravarti-redis \
                        --network chakravarti-network \
                        -p 6379:6379 \
                        redis:7-alpine redis-server --appendonly yes`);
                    spinner.succeed('Redis started (standalone)');
                }
            }
        } catch (error: any) {
            spinner.warn(`Redis: ${error.message}`);
        }

        // 5. Build executor image (if Dockerfile exists)
        spinner.start('Checking executor image...');
        try {
            const { stdout: imageExists } = await execAsync(
                'docker images chakravarti-executor --format "{{.ID}}"'
            );

            if (!imageExists.trim()) {
                const dockerfilePath = await findDockerfile();
                if (dockerfilePath) {
                    spinner.text = 'Building executor image (this may take a while)...';
                    await execAsync(`docker build -f ${dockerfilePath} -t chakravarti-executor .`);
                    spinner.succeed('Executor image built');
                } else {
                    spinner.info('No executor Dockerfile found');
                }
            } else {
                spinner.succeed('Executor image ready');
            }
        } catch (error: any) {
            spinner.warn(`Executor image: ${error.message}`);
        }

        // 6. Start agent containers (planner, executors, tester)
        spinner.start('Starting agent containers...');
        try {
            const agents: Array<{
                name: string;
                role: string;
                provider: string;
                workspacePath: string;
            }> = [];

            // Add planner (uses main project)
            if (config.agents.planner) {
                agents.push({
                    name: 'planner',
                    role: 'planner',
                    provider: config.agents.planner.provider,
                    workspacePath: projectPath
                });
            }

            // Add executors (use their worktrees)
            const executors = config.agents.executors || [];
            for (const executor of executors) {
                const worktreePath = path.join(projectPath, '.chakravarti/worktrees', executor.name);
                agents.push({
                    name: executor.name,
                    role: 'executor',
                    provider: executor.provider,
                    workspacePath: fs.existsSync(worktreePath) ? worktreePath : projectPath
                });
            }

            // Add tester (uses main project)
            if (config.agents.tester) {
                agents.push({
                    name: 'tester',
                    role: 'tester',
                    provider: config.agents.tester.provider,
                    workspacePath: projectPath
                });
            }

            let startedCount = 0;

            for (const agent of agents) {
                const containerName = `ckrv-${agent.role}-${agent.name}`;

                // Check if already running
                const { stdout: existing } = await execAsync(
                    `docker ps -q --filter "name=${containerName}" 2>/dev/null || true`
                );

                if (existing.trim()) {
                    continue; // Already running
                }

                // Remove stopped container if exists
                await execAsync(`docker rm ${containerName} 2>/dev/null || true`);

                // Determine CLI command based on provider
                const cliCommand = getCliCommand(agent.provider);

                // Get home directory for credential mounts
                const homeDir = process.env.HOME || '/root';

                // Build credential mount options based on provider
                // Note: CLI tools need write access for logs and state
                let credentialMounts = '';

                // Gemini CLI credentials (needs write for logs)
                if (fs.existsSync(path.join(homeDir, '.gemini'))) {
                    credentialMounts += ` -v "${homeDir}/.gemini:/home/node/.gemini"`;
                }

                // Claude CLI credentials  
                if (fs.existsSync(path.join(homeDir, '.claude'))) {
                    credentialMounts += ` -v "${homeDir}/.claude:/home/node/.claude"`;
                }

                // Google Cloud credentials
                if (fs.existsSync(path.join(homeDir, '.config/gcloud'))) {
                    credentialMounts += ` -v "${homeDir}/.config/gcloud:/home/node/.config/gcloud:ro"`;
                }

                // Start container with appropriate workspace, CLI, and credentials
                // For executors: also mount main .git directory so worktrees can access it
                const isExecutor = agent.role === 'executor';
                const gitMount = isExecutor ? `-v "${projectPath}/.git:${projectPath}/.git:rw"` : '';

                await execAsync(`docker run -d \
                    --name ${containerName} \
                    --network chakravarti-network \
                    -v "${agent.workspacePath}:/workspace" \
                    -v "${projectPath}/.chakravarti:/workspace/.chakravarti:ro" \
                    ${gitMount} \
                    ${credentialMounts} \
                    -w /workspace \
                    -e HOME=/home/node \
                    -e AGENT_NAME="${agent.name}" \
                    -e AGENT_ROLE="${agent.role}" \
                    -e AGENT_PROVIDER="${agent.provider}" \
                    -e CLI_COMMAND="${cliCommand}" \
                    -e GOOGLE_API_KEY="${process.env.GOOGLE_API_KEY || ''}" \
                    -e ANTHROPIC_API_KEY="${process.env.ANTHROPIC_API_KEY || ''}" \
                    chakravarti-executor`);

                startedCount++;
            }

            if (agents.length === 0) {
                spinner.warn('No agents configured. Run `ckrv setup` to configure your agent pool.');
            } else if (startedCount > 0) {
                spinner.succeed(`Started ${startedCount} agent container(s)`);
            } else {
                spinner.succeed(`${agents.length} agent(s) already running`);
            }
        } catch (error: any) {
            spinner.warn(`Agents: ${error.message}`);
        }

        // 7. Check GitLab health
        spinner.start('Waiting for GitLab to be ready (this may take 2-3 minutes)...');
        let gitlabReady = false;
        // Wait up to 180 seconds (3 minutes)
        for (let i = 0; i < 180; i++) {
            try {
                const { stdout: health } = await execAsync(
                    'docker inspect chakravarti-gitlab --format="{{.State.Health.Status}}" 2>/dev/null || echo "none"'
                );
                if (health.trim() === 'healthy') {
                    gitlabReady = true;
                    break;
                }
            } catch (error) { }
            await new Promise(r => setTimeout(r, 1000));
        }

        if (gitlabReady) {
            spinner.succeed('GitLab is healthy');

            // Auto-setup GitLab project (create repo, push code)
            const { setupGitLabProject } = await import('../utils/gitlab');
            await setupGitLabProject(projectPath);

        } else {
            spinner.warn('GitLab is still starting (timed out waiting for healthy status)');
        }

        // Get running agents for summary
        let agentList: Array<{ name: string, role: string }> = [];
        try {
            const { stdout } = await execAsync('docker ps --filter "name=ckrv-" --format "{{.Names}}"');
            const names = stdout.trim().split('\n').filter(n => n && !n.includes('gitlab'));
            agentList = names.map(name => {
                const parts = name.split('-');
                return {
                    name: parts.slice(2).join('-'),
                    role: parts[1] || 'agent'
                };
            });
        } catch (error) { }

        // Summary
        console.log(chalk.bold('\n✅ Environment Ready!\n'));
        console.log(chalk.dim('Services:'));
        console.log(`  ${chalk.green('●')} GitLab:  http://localhost:8080`);
        console.log(`  ${chalk.dim('  Login: root / chakravarti123')}`);

        if (agentList.length > 0) {
            console.log('');
            console.log(chalk.dim('Agents:'));

            // Group by role
            const planner = agentList.find(a => a.role === 'planner');
            const executors = agentList.filter(a => a.role === 'executor');
            const tester = agentList.find(a => a.role === 'tester');

            if (planner) {
                console.log(`  ${chalk.blue('●')} planner`);
            }
            executors.forEach(e => {
                console.log(`  ${chalk.green('●')} ${e.name}`);
            });
            if (tester) {
                console.log(`  ${chalk.yellow('●')} tester`);
            }
        }

        console.log('');
        console.log(chalk.dim('Commands:'));
        console.log(`  ${chalk.cyan('ckrv status')}        - Check status`);
        console.log(`  ${chalk.cyan('ckrv chat')}          - Chat with planner`);
        console.log(`  ${chalk.cyan('ckrv gitlab status')} - Check GitLab health`);
        console.log(`  ${chalk.cyan('ckrv down')}          - Stop all services`);
        console.log('');

    } catch (error: any) {
        if (spinner.isSpinning) {
            spinner.stop();
        }
        if (error.message.includes('agent-pool.yaml not found')) {
            // Check if project is initialized (has .chakravarti)
            if (fs.existsSync(path.join(projectPath, '.chakravarti'))) {
                console.log(chalk.yellow('⚠ Agents not configured'));
                console.log(chalk.dim('\nRun: ckrv setup\n'));
            } else {
                console.log(chalk.yellow('⚠ Not a Chakravarti project'));
                console.log(chalk.dim('\nInitialize with: ckrv init\n'));
            }
        } else {
            console.error(chalk.red(`Error: ${error.message}`));
        }
    }
};

export const downCommand = async (
    options: { path?: string } = {}
): Promise<void> => {
    console.log(chalk.bold('\n🛑 Stopping Chakravarti Environment\n'));

    const spinner = ora('Stopping services...').start();

    try {
        // Stop GitLab
        spinner.text = 'Stopping GitLab...';
        const composePath = await findDockerCompose();
        if (composePath) {
            await execAsync(`docker-compose -f ${composePath} down`);
        }

        // Stop any agent containers (planner, executors, tester)
        spinner.text = 'Stopping agent containers...';
        await execAsync('docker ps -q --filter "name=ckrv-" | xargs -r docker stop 2>/dev/null || true');
        await execAsync('docker ps -aq --filter "name=ckrv-" | xargs -r docker rm 2>/dev/null || true');

        spinner.succeed('All services stopped');
        console.log('');
    } catch (error: any) {
        spinner.fail(`Error: ${error.message}`);
    }
};

async function findDockerCompose(): Promise<string | null> {
    const locations = [
        '/apps/chakravarti-cli/docker-compose.gitlab.yml',
        path.join(process.cwd(), 'docker-compose.gitlab.yml'),
        path.join(process.cwd(), '.chakravarti', 'docker-compose.gitlab.yml'),
    ];

    for (const loc of locations) {
        if (fs.existsSync(loc)) {
            return loc;
        }
    }
    return null;
}

async function findDockerfile(): Promise<string | null> {
    const locations = [
        '/apps/chakravarti-cli/Dockerfile.executor',
        path.join(process.cwd(), 'Dockerfile.executor'),
        path.join(process.cwd(), '.chakravarti', 'Dockerfile.executor'),
    ];

    for (const loc of locations) {
        if (fs.existsSync(loc)) {
            return loc;
        }
    }
    return null;
}

/**
 * Get CLI command based on provider
 */
function getCliCommand(provider: string): string {
    const providerMap: Record<string, string> = {
        'gemini-cli': 'gemini',
        'gemini': 'gemini',
        'claude-cli': 'claude',
        'claude': 'claude',
        'anthropic': 'claude',
        'openai': 'openai',
        'gpt': 'openai',
    };
    return providerMap[provider.toLowerCase()] || 'gemini';
}
