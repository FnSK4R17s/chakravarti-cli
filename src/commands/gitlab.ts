import chalk from 'chalk';
import { exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';
import fs from 'fs';
import { setupLabOrigin } from '../git/gitlab';

const execAsync = promisify(exec);

export const gitlabCommand = async (
    action: string,
    options: { path?: string; name?: string } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();

    try {
        switch (action) {
            case 'init':
                await initGitLab();
                break;

            case 'start':
                await startGitLab();
                break;

            case 'stop':
                await stopGitLab();
                break;

            case 'status':
                await gitLabStatus();
                break;

            case 'setup':
                if (!options.name) {
                    throw new Error('Project name is required. Use --name <project-name>');
                }
                await setupGitLabProject(projectPath, options.name);
                break;

            case 'credentials':
                printGitLabCredentials();
                break;

            default:
                console.log(chalk.red(`Unknown action: ${action}`));
                console.log(chalk.dim('Available actions: init, start, stop, status, setup, credentials'));
        }
    } catch (error: any) {
        console.error(chalk.red(`\n✖ Error: ${error.message}`));
        process.exit(1);
    }
};

/**
 * Initialize GitLab with Docker Compose
 */
async function initGitLab(): Promise<void> {
    console.log(chalk.bold('\n🦊 Initializing Local GitLab\n'));

    // Check if Docker is running
    try {
        await execAsync('docker info');
    } catch (error) {
        throw new Error('Docker is not running. Please start Docker first.');
    }

    // Get the docker-compose file path
    const composePath = path.join(__dirname, '../../docker-compose.gitlab.yml');

    if (!fs.existsSync(composePath)) {
        throw new Error('docker-compose.gitlab.yml not found');
    }

    console.log(chalk.cyan('Starting GitLab container...'));
    console.log(chalk.dim('This may take a few minutes on first run.\n'));

    try {
        await execAsync(`docker-compose -f ${composePath} up -d`);

        console.log(chalk.green('\n✓ GitLab is starting up!'));
        console.log(chalk.bold('\n📋 GitLab Information:'));
        console.log(chalk.dim('  URL: http://localhost:8080'));
        console.log(chalk.dim('  Username: root'));
        console.log(chalk.dim('  Password: chakravarti123'));
        console.log(chalk.dim('  SSH Port: 2222'));
        console.log(chalk.yellow('\n⏳ GitLab takes ~2-3 minutes to fully start.'));
        console.log(chalk.dim('   Run `ckrv gitlab status` to check if it\'s ready.\n'));
    } catch (error: any) {
        throw new Error(`Failed to start GitLab: ${error.message}`);
    }
}

/**
 * Start GitLab
 */
async function startGitLab(): Promise<void> {
    const composePath = path.join(__dirname, '../../docker-compose.gitlab.yml');

    try {
        await execAsync(`docker-compose -f ${composePath} start`);
        console.log(chalk.green('✓ GitLab started'));
    } catch (error: any) {
        throw new Error(`Failed to start GitLab: ${error.message}`);
    }
}

/**
 * Stop GitLab
 */
async function stopGitLab(): Promise<void> {
    const composePath = path.join(__dirname, '../../docker-compose.gitlab.yml');

    try {
        await execAsync(`docker-compose -f ${composePath} stop`);
        console.log(chalk.green('✓ GitLab stopped'));
    } catch (error: any) {
        throw new Error(`Failed to stop GitLab: ${error.message}`);
    }
}

/**
 * Check GitLab status
 */
async function gitLabStatus(): Promise<void> {
    try {
        const { stdout } = await execAsync('docker ps --filter name=chakravarti-gitlab --format "{{.Status}}"');

        if (stdout.trim()) {
            console.log(chalk.green('✓ GitLab is running'));
            console.log(chalk.dim(`  Status: ${stdout.trim()}`));
            console.log(chalk.dim('  URL: http://localhost:8080'));
        } else {
            console.log(chalk.yellow('⚠ GitLab is not running'));
            console.log(chalk.dim('  Run `ckrv gitlab init` to start it'));
        }
    } catch (error: any) {
        console.log(chalk.red('✖ GitLab is not running'));
    }
}

/**
 * Setup GitLab project and configure lab origin
 */
async function setupGitLabProject(projectPath: string, projectName: string): Promise<void> {
    console.log(chalk.bold(`\n🔧 Setting up GitLab project: ${projectName}\n`));

    // Check if GitLab is running
    try {
        await execAsync('docker ps --filter name=chakravarti-gitlab --format "{{.Status}}"');
    } catch (error) {
        throw new Error('GitLab is not running. Run `ckrv gitlab init` first.');
    }

    // Setup lab origin
    const gitlabUrl = `http://localhost:8080/root/${projectName}.git`;
    await setupLabOrigin(projectPath, gitlabUrl);

    console.log(chalk.green('\n✓ GitLab project configured'));
    console.log(chalk.bold('\n📋 Next Steps:'));
    console.log(chalk.dim('  1. Go to http://localhost:8080'));
    console.log(chalk.dim('  2. Login with root / chakravarti123'));
    console.log(chalk.dim(`  3. Create a new project named "${projectName}"`));
    console.log(chalk.dim('  4. Push your code: git push lab main\n'));
}

/**
 * Print GitLab credentials
 */
function printGitLabCredentials(): void {
    console.log(chalk.bold('\n🔐 GitLab Credentials\n'));
    console.log(`  ${chalk.bold('URL:')}      http://localhost:8080`);
    console.log(`  ${chalk.bold('Username:')} root`);
    console.log(`  ${chalk.bold('Password:')} chakravarti123`);
    console.log(`\n  ${chalk.yellow('Note: These are default credentials.')}`);
    console.log(`  ${chalk.dim('Used by Chakravarti agents to push code automatically.')}\n`);
}
