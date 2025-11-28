import chalk from 'chalk';
import { CheckResult } from '../types';

export const renderTree = (results: CheckResult[]): void => {
    console.log(chalk.bold('\n    Check Available Tools'));

    results.forEach((result, index) => {
        const isLast = index === results.length - 1;
        const prefix = isLast ? '└── ' : '├── ';

        const icon = result.available ? chalk.green('●') : chalk.dim('○');
        const name = result.available
            ? chalk.white(result.agent.name)
            : chalk.dim(result.agent.name);
        const status = result.available
            ? chalk.dim('(available)')
            : result.agent.type === 'ide'
                ? chalk.dim('(IDE-based, no CLI check)')
                : chalk.dim('(not found)');

        console.log(`    ${prefix}${icon} ${name} ${status}`);
    });
    console.log(''); // Empty line at the end
};
