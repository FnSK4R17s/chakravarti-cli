import chalk from 'chalk';
import { loadAgentPool } from '../orchestration/executor';
import { hasLabOrigin, getLabOriginUrl } from '../git/gitlab';
import { listWorktrees } from '../git/worktree';
import { exec } from 'child_process';
import { promisify } from 'util';
import fs from 'fs';
import path from 'path';

const execAsync = promisify(exec);

export const statusCommand = async (
    options: { path?: string } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();

    console.log(chalk.bold('\n📊 Chakravarti Project Status\n'));

    try {
        // 1. Project Info
        const config = loadAgentPool(projectPath);
        console.log(chalk.bold('Project:'));
        console.log(`  ${chalk.cyan(config.project.name)}`);
        console.log(`  ${chalk.dim(config.project.description)}`);
        console.log('');

        // 2. Agent Pool
        console.log(chalk.bold('Agents:'));
        console.log(`  ${chalk.green('●')} Planner: ${config.agents.planner.provider} (${config.agents.planner.model || 'default'})`);
        config.agents.executors.forEach((executor, idx) => {
            console.log(`  ${chalk.green('●')} ${executor.name}: ${executor.provider} (${executor.model || 'default'})`);
        });
        console.log(`  ${chalk.green('●')} Tester: ${config.agents.tester.provider} (${config.agents.tester.model || 'default'})`);
        console.log('');

        // 3. GitLab Status
        console.log(chalk.bold('GitLab:'));
        try {
            const { stdout: gitlabStatus } = await execAsync('docker ps --filter name=chakravarti-gitlab --format "{{.Status}}"');
            if (gitlabStatus.trim()) {
                const statusMatch = gitlabStatus.match(/\(([^)]+)\)/);
                const health = statusMatch ? statusMatch[1] : 'unknown';
                const healthColor = health === 'healthy' ? chalk.green :
                    health === 'starting' ? chalk.yellow : chalk.red;
                console.log(`  ${healthColor('●')} Running (${health})`);
                console.log(`  ${chalk.dim('URL: http://localhost:8080')}`);

                // Check for lab origin
                if (await hasLabOrigin(projectPath)) {
                    const labUrl = await getLabOriginUrl(projectPath);
                    console.log(`  ${chalk.dim(`Lab origin: ${labUrl}`)}`);
                }
            } else {
                console.log(`  ${chalk.red('○')} Not running`);
                console.log(`  ${chalk.dim('Run: ckrv gitlab init')}`);
            }
        } catch (error) {
            console.log(`  ${chalk.red('○')} Not running`);
        }
        console.log('');

        // 4. Git Worktrees
        console.log(chalk.bold('Worktrees:'));
        try {
            const worktrees = await listWorktrees(projectPath);
            const executorWorktrees = worktrees.filter(w => !w.bare && w.path.includes('.chakravarti/worktrees'));
            if (executorWorktrees.length > 0) {
                executorWorktrees.forEach(wt => {
                    const name = path.basename(wt.path);
                    console.log(`  ${chalk.green('●')} ${name} (${wt.branch || 'detached'})`);
                });
            } else {
                console.log(`  ${chalk.dim('No active executor worktrees')}`);
            }
        } catch (error) {
            console.log(`  ${chalk.dim('No worktrees found')}`);
        }
        console.log('');

        // 5. Sprints
        console.log(chalk.bold('Sprints:'));
        const sprintsDir = path.join(projectPath, '.chakravarti', 'sprints');
        if (fs.existsSync(sprintsDir)) {
            const sprintFiles = fs.readdirSync(sprintsDir)
                .filter(f => f.startsWith('sprint-') && f.endsWith('.md'))
                .sort()
                .reverse();

            if (sprintFiles.length > 0) {
                sprintFiles.slice(0, 3).forEach(file => {
                    console.log(`  ${chalk.cyan('●')} ${file.replace('.md', '')}`);
                });
                if (sprintFiles.length > 3) {
                    console.log(`  ${chalk.dim(`... and ${sprintFiles.length - 3} more`)}`);
                }
            } else {
                console.log(`  ${chalk.dim('No sprints created yet')}`);
            }
        } else {
            console.log(`  ${chalk.dim('No sprints directory')}`);
        }
        console.log('');

        // 6. Quick Actions
        console.log(chalk.bold('Quick Actions:'));
        console.log(`  ${chalk.cyan('ckrv chat')}           - Chat with planner`);
        console.log(`  ${chalk.cyan('ckrv gitlab status')}  - Check GitLab health`);
        console.log(`  ${chalk.cyan('ckrv help')}           - Show all commands`);
        console.log('');

    } catch (error: any) {
        if (error.message.includes('agent-pool.yaml not found')) {
            console.log(chalk.yellow('⚠ Not a Chakravarti project'));
            console.log(chalk.dim('\nInitialize with: ckrv init\n'));
        } else {
            console.error(chalk.red(`Error: ${error.message}`));
        }
    }
};
