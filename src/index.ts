#!/usr/bin/env node
import { Command } from 'commander';
import { showBanner } from './ui/banner';
import { checkAgents } from './agents';
import { renderTree } from './ui/tree';
import chalk from 'chalk';
import ora from 'ora';

import { showHelp } from './ui/help';

const program = new Command();

program
    .name('chakravarti')
    .description('Orchestrate multiple CLI agents and coding assistants')
    .version('0.0.1')
    .configureHelp({
        helpWidth: 80,
        sortSubcommands: true,
    })
    .addHelpCommand(false)
    .action(async () => {
        const { statusCommand } = await import('./commands/status');
        await statusCommand();
    });

// Override help
program.on('option:help', () => {
    showHelp();
    process.exit(0);
});

// Intercept --help and -h before commander processes them
if (process.argv.includes('--help') || process.argv.includes('-h')) {
    showHelp();
    process.exit(0);
}

program
    .command('help')
    .description('Show help message')
    .action(() => {
        showHelp();
    });

program
    .command('init [path]')
    .description('Initialize a new Chakravarti project')
    .action(async (projectPath?: string) => {
        const { initCommand } = await import('./commands/init');
        await initCommand(projectPath);
    });

program
    .command('setup [path]')
    .description('Interactively configure agent pool')
    .action(async (projectPath?: string) => {
        const { setupCommand } = await import('./commands/setup');
        await setupCommand(projectPath);
    });

program
    .command('run <prompt>')
    .description('Run an agent with a prompt')
    .option('-a, --agent <name>', 'Specify which agent to use (planner, tester, or executor name)')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (prompt: string, options: { agent?: string; path?: string }) => {
        const { runCommand } = await import('./commands/run');
        await runCommand(prompt, options);
    });

program
    .command('chat')
    .description('Start interactive chat with an agent')
    .option('-a, --agent <name>', 'Specify which agent to chat with (default: planner)')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (options: { agent?: string; path?: string }) => {
        const { chatCommand } = await import('./commands/chat');
        await chatCommand(options);
    });

program
    .command('status')
    .description('Show current project status')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (options: { path?: string }) => {
        const { statusCommand } = await import('./commands/status');
        await statusCommand(options);
    });

program
    .command('gitlab <action>')
    .description('Manage local GitLab instance (init, start, stop, status, setup)')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .option('-n, --name <name>', 'Project name for setup')
    .action(async (action: string, options: { path?: string; name?: string }) => {
        const { gitlabCommand } = await import('./commands/gitlab');
        await gitlabCommand(action, options);
    });

program
    .command('check')
    .description('Check for installed tools')
    .action(async () => {
        showBanner();
        const spinner = ora('Checking for installed tools...').start();
        try {
            const results = await checkAgents();
            spinner.stop();
            renderTree(results);
        } catch (error) {
            spinner.fail('Failed to check tools');
            console.error(error);
        }
    });

program
    .command('up')
    .description('Start Chakravarti environment (GitLab, Docker network, etc.)')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (options: { path?: string }) => {
        const { upCommand } = await import('./commands/up');
        await upCommand(options);
    });

program
    .command('down')
    .description('Stop all Chakravarti services')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (options: { path?: string }) => {
        const { downCommand } = await import('./commands/up');
        await downCommand(options);
    });

program
    .command('worktree <action>')
    .description('Manage Git worktrees (setup, list, create, remove, prune)')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .option('-e, --executor <name>', 'Executor name for create/remove')
    .action(async (action: string, options: { path?: string; executor?: string }) => {
        const { worktreeCommand } = await import('./commands/worktree');
        await worktreeCommand(action, options);
    });

program
    .command('assign')
    .description('Assign task to an executor agent')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .option('-e, --executor <name>', 'Executor name')
    .option('-m, --prompt <text>', 'Task prompt/message')
    .option('-f, --file <path>', 'Read prompt from file')
    .option('-i, --interactive', 'Interactive mode (default if no prompt)')
    .option('-a, --auto', 'Auto-proceed without asking questions')
    .action(async (options: { path?: string; executor?: string; prompt?: string; file?: string; interactive?: boolean; auto?: boolean }) => {
        const { assignCommand } = await import('./commands/assign');
        await assignCommand(options);
    });

program
    .command('executors')
    .description('List executor agents and their status')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (options: { path?: string }) => {
        const { listExecutorsCommand } = await import('./commands/assign');
        await listExecutorsCommand(options);
    });

program
    .command('dispatch')
    .description('Dispatch task to specific executor via Redis queue')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .option('-e, --executor <name>', 'Target executor name')
    .option('-m, --prompt <text>', 'Task prompt/message')
    .option('-f, --file <path>', 'Read prompt from file')
    .option('-w, --wait', 'Wait for response')
    .option('-t, --timeout <seconds>', 'Response timeout in seconds', '300')
    .action(async (options: { path?: string; executor?: string; prompt?: string; file?: string; wait?: boolean; timeout?: string }) => {
        const { dispatchCommand } = await import('./commands/dispatch');
        await dispatchCommand({ ...options, timeout: parseInt(options.timeout || '300') });
    });

program
    .command('queue [action]')
    .description('Manage task queues (status, peek, clear)')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .option('-e, --executor <name>', 'Executor name')
    .option('-n, --count <number>', 'Number of tasks to show', '5')
    .action(async (action: string = 'status', options: { path?: string; executor?: string; count?: string }) => {
        const { queueStatusCommand, peekQueueCommand, clearQueueCommand } = await import('./commands/dispatch');

        switch (action) {
            case 'status':
                await queueStatusCommand(options);
                break;
            case 'peek':
                if (!options.executor) {
                    console.log('Error: --executor required for peek');
                    return;
                }
                await peekQueueCommand(options.executor, { ...options, count: parseInt(options.count || '5') });
                break;
            case 'clear':
                if (!options.executor) {
                    console.log('Error: --executor required for clear (use "all" for all queues)');
                    return;
                }
                await clearQueueCommand(options.executor, options);
                break;
            default:
                console.log('Actions: status, peek, clear');
        }
    });

program
    .command('worker')
    .description('Start worker to consume tasks from queue')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .option('-e, --executor <name>', 'Executor name')
    .option('--once', 'Process one task and exit')
    .action(async (options: { path?: string; executor?: string; once?: boolean }) => {
        const { workerCommand } = await import('./commands/worker');
        await workerCommand(options);
    });

program
    .command('sprint [feature]')
    .description('Start a new sprint with automated spec planning')
    .option('-p, --path <path>', 'Project path', process.cwd())
    .action(async (feature: string | undefined, options: { path?: string }) => {
        const { sprintCommand } = await import('./commands/sprint');
        await sprintCommand(feature, options);
    });

program.parse(process.argv);
