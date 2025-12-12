import chalk from 'chalk';
import ora from 'ora';
import { exec, spawn } from 'child_process';
import { promisify } from 'util';
import Redis from 'ioredis';
import { loadAgentPool } from '../orchestration/executor';
import { TaskMessage, QUEUES } from '../messaging/queue';

const execAsync = promisify(exec);

const REDIS_HOST = process.env.REDIS_HOST || 'localhost';
const REDIS_PORT = parseInt(process.env.REDIS_PORT || '6379');

// Check if docker is available (running on host vs inside container)
async function isDockerAvailable(): Promise<boolean> {
    try {
        await execAsync('docker --version');
        return true;
    } catch {
        return false;
    }
}

interface WorkerOptions {
    path?: string;
    executor?: string;
    once?: boolean;
}

/**
 * Start a worker that consumes tasks from the executor's queue
 */
export const workerCommand = async (options: WorkerOptions = {}): Promise<void> => {
    const projectPath = options.path || process.cwd();
    const executorName = options.executor || process.env.AGENT_NAME;

    if (!executorName) {
        console.log(chalk.red('No executor specified. Use -e or set AGENT_NAME'));
        return;
    }

    console.log(chalk.bold(`\n🔄 Starting Worker: ${chalk.cyan(executorName)}\n`));

    // Connect to Redis
    let redis: Redis;
    try {
        redis = new Redis({
            host: REDIS_HOST,
            port: REDIS_PORT,
        });
        await redis.ping();
        console.log(chalk.green('✓ Connected to Redis'));
    } catch (error: any) {
        console.log(chalk.red(`Redis connection failed: ${error.message}`));
        return;
    }

    const queueName = `queue:${QUEUES.executor(executorName)}`;
    console.log(chalk.dim(`Listening on: ${queueName}`));
    console.log(chalk.dim('Press Ctrl+C to stop\n'));

    // Process loop
    const processTask = async (): Promise<boolean> => {
        try {
            // Block wait for task (10 second intervals to allow graceful shutdown)
            const result = await redis.brpop(queueName, 10);

            if (!result) {
                return true; // Continue waiting
            }

            const task = JSON.parse(result[1]) as TaskMessage;

            console.log(chalk.bold(`\n📥 Task Received: ${chalk.dim(task.id)}`));
            console.log(chalk.dim(`From: ${task.from}`));
            console.log(chalk.dim(`Content: ${task.content.slice(0, 100)}...`));

            // Execute the task using assign command logic
            const spinner = ora('Executing task...').start();

            try {
                // Prepare prompt with auto-proceed
                const prompt = `${task.content}\n\nIMPORTANT: Proceed immediately without asking questions. Make reasonable assumptions and complete the task autonomously.`;

                // Detect if running inside container (check for AGENT_NAME env var or no docker)
                const isInsideContainer = !!process.env.AGENT_NAME || !await isDockerAvailable();

                let stdout: string;

                if (isInsideContainer) {
                    // Running inside container - call gemini directly
                    const fs = await import('fs');
                    const tempFile = `/tmp/task-${Date.now()}.txt`;
                    fs.writeFileSync(tempFile, prompt);

                    const result = await execAsync(
                        `cat ${tempFile} | gemini --yolo --output-format text 2>&1; rm ${tempFile}`,
                        {
                            maxBuffer: 10 * 1024 * 1024,
                            timeout: 300000,
                            cwd: '/workspace'
                        }
                    );
                    stdout = result.stdout;
                } else {
                    // Running on host - use docker exec
                    const containerName = `ckrv-executor-${executorName}`;
                    const base64Prompt = Buffer.from(prompt).toString('base64');
                    const tempFile = `/tmp/task-${Date.now()}.txt`;

                    await execAsync(
                        `docker exec ${containerName} sh -c 'echo "${base64Prompt}" | base64 -d > ${tempFile}'`
                    );

                    const result = await execAsync(
                        `docker exec ${containerName} sh -c 'cat ${tempFile} | gemini --yolo --output-format text 2>&1; rm ${tempFile}'`,
                        {
                            maxBuffer: 10 * 1024 * 1024,
                            timeout: 300000
                        }
                    );
                    stdout = result.stdout;
                }

                spinner.succeed('Task completed');
                console.log(chalk.dim('\n' + '─'.repeat(50)));
                console.log(stdout.slice(0, 500));
                if (stdout.length > 500) console.log(chalk.dim('... (truncated)'));
                console.log(chalk.dim('─'.repeat(50)));

                // Auto-commit and push to GitLab
                const gitSpinner = ora('Committing changes to GitLab...').start();
                try {
                    const workspacePath = isInsideContainer ? '/workspace' :
                        `${projectPath}/.chakravarti/worktrees/${executorName}`;

                    // Check for changes
                    const { stdout: statusOutput } = await execAsync(
                        'git status --porcelain',
                        { cwd: workspacePath }
                    );

                    if (statusOutput.trim()) {
                        // Stage all changes
                        // Stage changes (exclude .chakravarti which is a readonly mount)
                        await execAsync('git add -- . ":!.chakravarti"', { cwd: workspacePath });

                        // Commit with task reference
                        const commitMsg = `[${executorName}] ${task.content.slice(0, 50)}${task.content.length > 50 ? '...' : ''}\n\nTask ID: ${task.id}`;
                        await execAsync(`git commit -m "${commitMsg.replace(/"/g, '\\"')}"`, { cwd: workspacePath });

                        // Get current branch
                        const { stdout: branchOutput } = await execAsync('git rev-parse --abbrev-ref HEAD', { cwd: workspacePath });
                        const branch = branchOutput.trim() || executorName;

                        // Push to GitLab
                        // Inside container: use chakravarti-gitlab:80
                        // On host: use localhost:8080
                        const gitlabUrl = isInsideContainer
                            ? 'http://root:chakravarti123@chakravarti-gitlab:80/root/chakra-test.git'
                            : 'http://root:chakravarti123@localhost:8080/root/chakra-test.git';

                        try {
                            await execAsync(`git push ${gitlabUrl} ${branch} 2>&1`, {
                                cwd: workspacePath,
                                timeout: 30000
                            });
                            gitSpinner.succeed(`Pushed to GitLab: ${chalk.cyan(branch)}`);
                        } catch (pushError: any) {
                            // Try creating the branch remotely
                            await execAsync(`git push -u ${gitlabUrl} ${branch} 2>&1`, {
                                cwd: workspacePath,
                                timeout: 30000
                            });
                            gitSpinner.succeed(`Created and pushed branch: ${chalk.cyan(branch)}`);
                        }
                    } else {
                        gitSpinner.info('No changes to commit');
                    }
                } catch (gitError: any) {
                    gitSpinner.warn(`Git: ${gitError.message.slice(0, 80)}`);
                }

                // Push response to response queue
                const response: TaskMessage = {
                    id: `resp-${Date.now()}`,
                    from: executorName,
                    to: task.from,
                    type: 'response',
                    content: stdout,
                    timestamp: Date.now(),
                    metadata: { inResponseTo: task.id }
                };

                await redis.lpush(`queue:responses:${task.id}`, JSON.stringify(response));

            } catch (error: any) {
                spinner.fail('Task failed');
                console.error(chalk.red(error.message));

                // Push error response
                const errorResponse: TaskMessage = {
                    id: `resp-${Date.now()}`,
                    from: executorName,
                    to: task.from,
                    type: 'error',
                    content: error.message,
                    timestamp: Date.now(),
                    metadata: { inResponseTo: task.id }
                };

                await redis.lpush(`queue:responses:${task.id}`, JSON.stringify(errorResponse));
            }

            return !options.once; // Continue if not --once mode

        } catch (error: any) {
            console.error(chalk.red(`Worker error: ${error.message}`));
            return true; // Continue on error
        }
    };

    // Main loop
    let running = true;

    process.on('SIGINT', () => {
        console.log(chalk.dim('\nShutting down worker...'));
        running = false;
    });

    while (running) {
        const shouldContinue = await processTask();
        if (!shouldContinue) break;
    }

    await redis.quit();
    console.log(chalk.green('\n✓ Worker stopped'));
};

/**
 * Run a single task from the queue (for testing)
 */
export const runOneTaskCommand = async (options: WorkerOptions = {}): Promise<void> => {
    await workerCommand({ ...options, once: true });
};
