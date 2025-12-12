import Redis from 'ioredis';
import chalk from 'chalk';

export interface TaskMessage {
    id: string;
    from: string;
    to: string;
    type: 'task' | 'response' | 'status' | 'error';
    content: string;
    timestamp: number;
    metadata?: Record<string, any>;
}

export interface MessageQueue {
    redis: Redis;
    publish(channel: string, message: TaskMessage): Promise<void>;
    subscribe(channel: string, callback: (message: TaskMessage) => void): Promise<void>;
    pushTask(queue: string, task: TaskMessage): Promise<void>;
    popTask(queue: string, timeout?: number): Promise<TaskMessage | null>;
    getQueueLength(queue: string): Promise<number>;
    close(): Promise<void>;
}

const REDIS_HOST = process.env.REDIS_HOST || 'chakravarti-redis';
const REDIS_PORT = parseInt(process.env.REDIS_PORT || '6379');

/**
 * Create a message queue connection
 */
export async function createMessageQueue(): Promise<MessageQueue> {
    const redis = new Redis({
        host: REDIS_HOST,
        port: REDIS_PORT,
        maxRetriesPerRequest: 3,
        lazyConnect: true,
    });

    // For subscriptions, we need a separate connection
    const subscriber = new Redis({
        host: REDIS_HOST,
        port: REDIS_PORT,
        maxRetriesPerRequest: 3,
    });

    await redis.connect();

    return {
        redis,

        async publish(channel: string, message: TaskMessage): Promise<void> {
            await redis.publish(channel, JSON.stringify(message));
        },

        async subscribe(channel: string, callback: (message: TaskMessage) => void): Promise<void> {
            await subscriber.subscribe(channel);
            subscriber.on('message', (ch, msg) => {
                if (ch === channel) {
                    try {
                        const parsed = JSON.parse(msg) as TaskMessage;
                        callback(parsed);
                    } catch (e) {
                        console.error('Failed to parse message:', e);
                    }
                }
            });
        },

        async pushTask(queue: string, task: TaskMessage): Promise<void> {
            await redis.lpush(`queue:${queue}`, JSON.stringify(task));
        },

        async popTask(queue: string, timeout: number = 0): Promise<TaskMessage | null> {
            const result = timeout > 0
                ? await redis.brpop(`queue:${queue}`, timeout)
                : await redis.rpop(`queue:${queue}`);

            if (!result) return null;

            const data = Array.isArray(result) ? result[1] : result;
            return JSON.parse(data) as TaskMessage;
        },

        async getQueueLength(queue: string): Promise<number> {
            return redis.llen(`queue:${queue}`);
        },

        async close(): Promise<void> {
            await subscriber.quit();
            await redis.quit();
        }
    };
}

/**
 * Create a unique task ID
 */
export function createTaskId(): string {
    return `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Create a task message
 */
export function createTask(
    from: string,
    to: string,
    content: string,
    metadata?: Record<string, any>
): TaskMessage {
    return {
        id: createTaskId(),
        from,
        to,
        type: 'task',
        content,
        timestamp: Date.now(),
        metadata
    };
}

/**
 * Create a response message
 */
export function createResponse(
    originalTask: TaskMessage,
    from: string,
    content: string,
    metadata?: Record<string, any>
): TaskMessage {
    return {
        id: createTaskId(),
        from,
        to: originalTask.from,
        type: 'response',
        content,
        timestamp: Date.now(),
        metadata: {
            ...metadata,
            inResponseTo: originalTask.id
        }
    };
}

/**
 * Queue names for agents
 */
export const QUEUES = {
    PLANNER: 'planner',
    TESTER: 'tester',
    executor: (name: string) => `executor-${name}`,
};

/**
 * Channel names for pub/sub
 */
export const CHANNELS = {
    BROADCAST: 'chakravarti:broadcast',
    TASKS: 'chakravarti:tasks',
    STATUS: 'chakravarti:status',
};

/**
 * Check if Redis is available
 */
export async function checkRedisConnection(): Promise<boolean> {
    try {
        const redis = new Redis({
            host: 'localhost',
            port: REDIS_PORT,
            maxRetriesPerRequest: 1,
            connectTimeout: 2000,
            lazyConnect: true,
        });
        await redis.connect();
        await redis.ping();
        await redis.quit();
        return true;
    } catch (error) {
        return false;
    }
}
