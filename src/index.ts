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
        showBanner();
        const spinner = ora('Checking for installed tools...').start();
        try {
            const results = await checkAgents();
            spinner.stop();
            renderTree(results);
            console.log(chalk.green('\n    Chakravarti CLI is ready to use!'));
        } catch (error) {
            spinner.fail('Failed to check tools');
            console.error(error);
        }
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

program.parse(process.argv);
