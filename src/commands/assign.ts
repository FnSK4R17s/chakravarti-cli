import chalk from 'chalk';
import ora from 'ora';
import { exec, spawn } from 'child_process';
import { promisify } from 'util';
import { loadAgentPool } from '../orchestration/executor';
import fs from 'fs';
import path from 'path';

const execAsync = promisify(exec);

interface AssignOptions {
    path?: string;
    executor?: string;
    prompt?: string;
    file?: string;
    interactive?: boolean;
    auto?: boolean;
}

export const assignCommand = async (options: AssignOptions = {}): Promise<void> => {
    const projectPath = options.path || process.cwd();

    try {
        // Load config
        const config = loadAgentPool(projectPath);

        // Get executor name or default to first executor
        let executorName = options.executor;
        if (!executorName) {
            const executors = config.agents.executors || [];
            if (executors.length === 0) {
                console.log(chalk.red('No executors defined in agent-pool.yaml'));
                return;
            }
            executorName = executors[0].name;
        }

        const containerName = `ckrv-executor-${executorName}`;

        // Check if container is running
        const { stdout: running } = await execAsync(
            `docker ps -q --filter "name=${containerName}" 2>/dev/null || true`
        );

        if (!running.trim()) {
            console.log(chalk.red(`Executor ${executorName} is not running`));
            console.log(chalk.dim('Run: ckrv up'));
            return;
        }

        // Get the prompt
        let prompt = options.prompt || '';

        if (options.file) {
            // Read prompt from file
            const filePath = path.resolve(options.file);
            if (fs.existsSync(filePath)) {
                prompt = fs.readFileSync(filePath, 'utf-8');
            } else {
                console.log(chalk.red(`File not found: ${options.file}`));
                return;
            }
        }

        // Add auto-proceed instruction if --auto flag is set
        if (options.auto && prompt) {
            prompt = `${prompt}\n\nIMPORTANT: Proceed immediately without asking questions. Make reasonable assumptions and complete the task autonomously. Do not wait for confirmation.`;
        }

        if (options.interactive || !prompt) {
            // Start interactive session with the executor
            console.log(chalk.bold(`\n🤖 Connecting to ${chalk.cyan(executorName)}...\n`));

            const proc = spawn('docker', [
                'exec', '-it', containerName, 'gemini'
            ], {
                stdio: 'inherit'
            });

            proc.on('close', (code) => {
                console.log(chalk.dim(`\nSession ended (exit code: ${code})`));
            });

            return;
        }

        // Non-interactive: Send prompt to executor using Gemini CLI
        console.log(chalk.bold(`\n📤 Sending task to ${chalk.cyan(executorName)}...\n`));
        console.log(chalk.dim('Prompt:'));
        console.log(chalk.white(prompt.slice(0, 500) + (prompt.length > 500 ? '...' : '')));
        console.log('');

        const spinner = ora('Executing task...').start();

        try {
            // Write prompt to a temp file in the container using base64 to avoid escaping issues
            const base64Prompt = Buffer.from(prompt).toString('base64');
            const tempFile = `/tmp/task-${Date.now()}.txt`;

            await execAsync(
                `docker exec ${containerName} sh -c 'echo "${base64Prompt}" | base64 -d > ${tempFile}'`
            );

            // Execute gemini with the prompt file, using yolo mode for auto-approval
            const { stdout } = await execAsync(
                `docker exec ${containerName} sh -c 'cat ${tempFile} | gemini --yolo --output-format text 2>&1; rm ${tempFile}'`,
                {
                    maxBuffer: 10 * 1024 * 1024, // 10MB buffer
                    timeout: 300000 // 5 minute timeout
                }
            );

            spinner.succeed('Task completed');
            console.log(chalk.dim('\n' + '─'.repeat(60) + '\n'));
            console.log(stdout);
            console.log(chalk.dim('─'.repeat(60)));

        } catch (error: any) {
            spinner.fail('Task failed');
            if (error.stdout) {
                console.log(chalk.dim('\nPartial output:'));
                console.log(error.stdout);
            }
            console.error(chalk.red(`\nError: ${error.message}`));
        }

    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
};

/**
 * List available executors and their status
 */
export const listExecutorsCommand = async (options: { path?: string } = {}): Promise<void> => {
    const projectPath = options.path || process.cwd();

    try {
        const config = loadAgentPool(projectPath);
        const executors = config.agents.executors || [];

        console.log(chalk.bold('\n🤖 Executors\n'));

        for (const executor of executors) {
            const containerName = `ckrv-executor-${executor.name}`;

            const { stdout: running } = await execAsync(
                `docker ps -q --filter "name=${containerName}" 2>/dev/null || true`
            );

            const status = running.trim()
                ? chalk.green('● running')
                : chalk.red('○ stopped');

            console.log(`  ${status}  ${executor.name}`);
            console.log(chalk.dim(`          Provider: ${executor.provider}`));
            console.log(chalk.dim(`          Container: ${containerName}`));
            console.log('');
        }

        console.log(chalk.dim('Commands:'));
        console.log(chalk.dim('  ckrv assign -e <name>           - Interactive session'));
        console.log(chalk.dim('  ckrv assign -e <name> -p "..."  - Send prompt'));
        console.log(chalk.dim('  ckrv assign -e <name> -f file   - Send file as prompt'));
        console.log('');

    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
};
