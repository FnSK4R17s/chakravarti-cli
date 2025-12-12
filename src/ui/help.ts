import chalk from 'chalk';
import boxen from 'boxen';
import { showBanner } from './banner';

export const showHelp = (): void => {
    showBanner();

    console.log(chalk.bold('\n Usage: chakravarti [OPTIONS] COMMAND [ARGS]...'));
    console.log(chalk.dim('\n Orchestrate multiple CLI agents and coding assistants\n'));

    const options = [
        `${chalk.cyan('--help')}          Show this message and exit.`,
        `${chalk.cyan('--version')}       Display version information.`,
    ].join('\n');

    console.log(boxen(options, {
        title: 'Options',
        titleAlignment: 'left',
        padding: 1,
        margin: 0,
        borderStyle: 'round',
        borderColor: 'gray',
        width: 80,
    }));

    const commands = [
        `${chalk.cyan('init')}          Initialize a new Chakravarti project.`,
        `${chalk.cyan('setup')}         Interactively configure agent pool.`,
        `${chalk.cyan('run')}           Run an agent with a prompt.`,
        `${chalk.cyan('chat')}          Start interactive chat with an agent.`,
        `${chalk.cyan('check')}         Check for installed tools.`,
        `${chalk.cyan('sprint')}        Start a new sprint with automated spec planning.`,
    ].join('\n');

    console.log(boxen(commands, {
        title: 'Commands',
        titleAlignment: 'left',
        padding: 1,
        margin: { top: 1, bottom: 0, left: 0, right: 0 },
        borderStyle: 'round',
        borderColor: 'gray',
        width: 80,
    }));
};
