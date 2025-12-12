import chalk from 'chalk';
import ora from 'ora';
import { exec, spawn } from 'child_process';
import { promisify } from 'util';
import Redis from 'ioredis';
import { loadAgentPool } from '../orchestration/executor';
import { createTask, createTaskId, TaskMessage, QUEUES, CHANNELS } from '../messaging/queue';
import fs from 'fs';
import path from 'path';

const execAsync = promisify(exec);

interface DispatchOptions {
    path?: string;
    executor?: string;
    prompt?: string;
    file?: string;
    wait?: boolean;
    timeout?: number;
}

const REDIS_HOST = process.env.REDIS_HOST || 'localhost';
const REDIS_PORT = parseInt(process.env.REDIS_PORT || '6379');

/**
 * Dispatch a task to a specific executor via Redis queue
 */
export const dispatchCommand = async (options: DispatchOptions = {}): Promise<void> => {
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

        // Get the prompt
        let prompt = options.prompt || '';

        if (options.file) {
            const filePath = path.resolve(options.file);
            if (fs.existsSync(filePath)) {
                prompt = fs.readFileSync(filePath, 'utf-8');
            } else {
                console.log(chalk.red(`File not found: ${options.file}`));
                return;
            }
        }

        if (!prompt) {
            console.log(chalk.red('No prompt provided. Use -m or -f'));
            return;
        }

        // Connect to Redis
        const spinner = ora('Connecting to Redis...').start();

        let redis: Redis;
        try {
            redis = new Redis({
                host: REDIS_HOST,
                port: REDIS_PORT,
                maxRetriesPerRequest: 3,
            });
            await redis.ping();
            spinner.succeed('Connected to Redis');
        } catch (error: any) {
            spinner.fail(`Redis connection failed: ${error.message}`);
            console.log(chalk.dim('Make sure Redis is running: ckrv up'));
            return;
        }

        // Create task message
        const task = createTask('planner', executorName, prompt, {
            projectPath,
            autoApprove: true,
        });

        // Push to executor's queue
        const queueName = QUEUES.executor(executorName);
        spinner.start(`Dispatching to ${chalk.cyan(executorName)}...`);

        await redis.lpush(`queue:${queueName}`, JSON.stringify(task));

        spinner.succeed(`Task dispatched to ${chalk.cyan(executorName)}`);
        console.log(chalk.dim(`  Task ID: ${task.id}`));
        console.log(chalk.dim(`  Queue: queue:${queueName}`));
        console.log(chalk.dim(`  Prompt: ${prompt.slice(0, 100)}${prompt.length > 100 ? '...' : ''}`));

        // Optionally wait for response
        if (options.wait) {
            const timeout = options.timeout || 300; // 5 min default
            spinner.start(`Waiting for response (timeout: ${timeout}s)...`);

            const responseQueue = `queue:responses:${task.id}`;
            const result = await redis.brpop(responseQueue, timeout);

            if (result) {
                const response = JSON.parse(result[1]) as TaskMessage;
                spinner.succeed('Response received');
                console.log(chalk.dim('\n' + '─'.repeat(60)));
                console.log(response.content);
                console.log(chalk.dim('─'.repeat(60)));
            } else {
                spinner.warn('Timeout waiting for response');
            }
        }

        await redis.quit();

    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
};

/**
 * Show queue status for all executors
 */
export const queueStatusCommand = async (options: { path?: string } = {}): Promise<void> => {
    const projectPath = options.path || process.cwd();

    try {
        const config = loadAgentPool(projectPath);
        const executors = config.agents.executors || [];

        // Connect to Redis
        let redis: Redis;
        try {
            redis = new Redis({
                host: REDIS_HOST,
                port: REDIS_PORT,
                maxRetriesPerRequest: 1,
            });
            await redis.ping();
        } catch (error: any) {
            console.log(chalk.red(`Redis not available: ${error.message}`));
            return;
        }

        console.log(chalk.bold('\n📬 Task Queues\n'));

        // Planner queue
        const plannerLen = await redis.llen('queue:planner');
        console.log(`  ${chalk.blue('●')} planner: ${plannerLen} task(s)`);

        // Executor queues
        for (const executor of executors) {
            const queueName = QUEUES.executor(executor.name);
            const len = await redis.llen(`queue:${queueName}`);
            const color = len > 0 ? chalk.yellow : chalk.green;
            console.log(`  ${color('●')} ${executor.name}: ${len} task(s)`);
        }

        // Tester queue
        const testerLen = await redis.llen('queue:tester');
        console.log(`  ${chalk.magenta('●')} tester: ${testerLen} task(s)`);

        console.log('');
        await redis.quit();

    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
};

/**
 * Clear queue for an executor
 */
export const clearQueueCommand = async (
    executor: string,
    options: { path?: string } = {}
): Promise<void> => {
    try {
        const redis = new Redis({
            host: REDIS_HOST,
            port: REDIS_PORT,
        });

        const queueName = executor === 'all'
            ? '*'
            : `queue:${QUEUES.executor(executor)}`;

        if (executor === 'all') {
            const keys = await redis.keys('queue:*');
            if (keys.length > 0) {
                await redis.del(...keys);
                console.log(chalk.green(`✓ Cleared ${keys.length} queue(s)`));
            } else {
                console.log(chalk.dim('No queues to clear'));
            }
        } else {
            await redis.del(queueName);
            console.log(chalk.green(`✓ Cleared queue: ${queueName}`));
        }

        await redis.quit();

    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
};

/**
 * Peek at tasks in a queue without removing them
 */
export const peekQueueCommand = async (
    executor: string,
    options: { path?: string; count?: number } = {}
): Promise<void> => {
    try {
        const redis = new Redis({
            host: REDIS_HOST,
            port: REDIS_PORT,
        });

        const queueName = `queue:${QUEUES.executor(executor)}`;
        const count = options.count || 5;

        const tasks = await redis.lrange(queueName, 0, count - 1);

        console.log(chalk.bold(`\n📋 Queue: ${executor} (${tasks.length} shown)\n`));

        if (tasks.length === 0) {
            console.log(chalk.dim('  No pending tasks'));
        } else {
            tasks.forEach((taskStr, i) => {
                const task = JSON.parse(taskStr) as TaskMessage;
                console.log(`  ${i + 1}. ${chalk.dim(task.id)}`);
                console.log(`     From: ${task.from}`);
                console.log(`     ${task.content.slice(0, 80)}${task.content.length > 80 ? '...' : ''}`);
                console.log('');
            });
        }

        await redis.quit();

    } catch (error: any) {
        console.error(chalk.red(`Error: ${error.message}`));
    }
};
