import chalk from 'chalk';
import ora from 'ora';
import path from 'path';
import { loadAgentPool } from '../orchestration/executor';
import { createWorktree, listWorktrees, removeWorktree, pruneWorktrees } from '../git/worktree';
import { setupLabOrigin, hasLabOrigin } from '../git/gitlab';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export const worktreeCommand = async (
    action: string,
    options: { path?: string; name?: string; executor?: string } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();

    switch (action) {
        case 'setup':
            await setupWorktrees(projectPath);
            break;
        case 'list':
            await listAllWorktrees(projectPath);
            break;
        case 'create':
            if (!options.executor) {
                console.log(chalk.red('Error: --executor is required'));
                return;
            }
            await createExecutorWorktree(projectPath, options.executor);
            break;
        case 'remove':
            if (!options.executor) {
                console.log(chalk.red('Error: --executor is required'));
                return;
            }
            await removeExecutorWorktree(projectPath, options.executor);
            break;
        case 'prune':
            await pruneWorktrees(projectPath);
            break;
        default:
            console.log(chalk.yellow(`Unknown action: ${action}`));
            console.log('Available actions: setup, list, create, remove, prune');
    }
};

async function setupWorktrees(projectPath: string): Promise<void> {
    console.log(chalk.bold('\n🌳 Setting Up Worktrees\n'));

    const spinner = ora('Loading configuration...').start();

    try {
        // Load config
        const config = loadAgentPool(projectPath);
        spinner.succeed(`Project: ${chalk.cyan(config.project.name)}`);

        // Check for lab origin
        spinner.start('Checking lab origin...');
        const hasLab = await hasLabOrigin(projectPath);

        if (!hasLab) {
            spinner.warn('Lab origin not configured');
            console.log(chalk.dim('  Run: ckrv gitlab setup --name <project-name>'));
            console.log(chalk.dim('  (First create project in GitLab at http://localhost:8080)\n'));
        } else {
            spinner.succeed('Lab origin configured');
        }

        // Create worktrees for each executor
        const executors = config.agents.executors || [];

        if (executors.length === 0) {
            console.log(chalk.yellow('\n⚠ No executors defined in agent-pool.yaml'));
            return;
        }

        console.log(chalk.dim(`\nCreating worktrees for ${executors.length} executor(s)...\n`));

        for (const executor of executors) {
            spinner.start(`Creating worktree for ${executor.name}...`);
            try {
                await createWorktree({
                    executorName: executor.name,
                    branchName: `executor/${executor.name}`,
                    basePath: '.chakravarti/worktrees',
                    projectPath
                });
                spinner.succeed(`Worktree: ${executor.name}`);
            } catch (error: any) {
                if (error.message.includes('already exists')) {
                    spinner.info(`Worktree ${executor.name} already exists`);
                } else {
                    spinner.fail(`Failed: ${error.message}`);
                }
            }
        }

        // Summary
        console.log(chalk.bold('\n✅ Worktrees Ready!\n'));
        console.log(chalk.dim('Location: .chakravarti/worktrees/'));
        console.log(chalk.dim('Branches: executor/<name>\n'));

    } catch (error: any) {
        spinner.fail(`Error: ${error.message}`);
    }
}

async function listAllWorktrees(projectPath: string): Promise<void> {
    console.log(chalk.bold('\n🌳 Git Worktrees\n'));

    try {
        const worktrees = await listWorktrees(projectPath);

        if (worktrees.length === 0) {
            console.log(chalk.dim('No worktrees found'));
            return;
        }

        worktrees.forEach(wt => {
            const isMain = !wt.path.includes('worktrees');
            const icon = isMain ? chalk.blue('●') : chalk.green('●');
            const name = path.basename(wt.path);
            console.log(`  ${icon} ${name}`);
            console.log(chalk.dim(`    Path: ${wt.path}`));
            console.log(chalk.dim(`    Branch: ${wt.branch || 'detached'}`));
        });
        console.log('');
    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
}

async function createExecutorWorktree(projectPath: string, executorName: string): Promise<void> {
    const spinner = ora(`Creating worktree for ${executorName}...`).start();

    try {
        await createWorktree({
            executorName,
            branchName: `executor/${executorName}`,
            basePath: '.chakravarti/worktrees',
            projectPath
        });
        spinner.succeed(`Created worktree: ${executorName}`);
    } catch (error: any) {
        spinner.fail(`Failed: ${error.message}`);
    }
}

async function removeExecutorWorktree(projectPath: string, executorName: string): Promise<void> {
    const spinner = ora(`Removing worktree for ${executorName}...`).start();

    try {
        const worktreePath = path.join(projectPath, '.chakravarti/worktrees', executorName);
        await removeWorktree(projectPath, worktreePath);
        spinner.succeed(`Removed worktree: ${executorName}`);
    } catch (error: any) {
        spinner.fail(`Failed: ${error.message}`);
    }
}
