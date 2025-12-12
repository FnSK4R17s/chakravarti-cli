import fs from 'fs';
import path from 'path';
import yaml from 'js-yaml';
import execa from 'execa';
import chalk from 'chalk';
import ora from 'ora';

interface AgentConfig {
    role: string;
    provider: string;
    model?: string;
    mode?: string;
    responsibilities?: string[];
}

interface ExecutorConfig extends AgentConfig {
    name: string;
    branch_prefix?: string;
}

interface AgentPoolConfig {
    version: number;
    project: {
        name: string;
        description: string;
        base_branch?: string;
    };
    agents: {
        planner: AgentConfig;
        executors: ExecutorConfig[];
        tester: AgentConfig;
    };
    communication?: {
        type: string;
        backend: string;
    };
}

/**
 * Load and parse agent-pool.yaml configuration
 */
export const loadAgentPool = (projectPath: string = process.cwd()): AgentPoolConfig => {
    const configPath = path.join(projectPath, 'agent-pool.yaml');

    if (!fs.existsSync(configPath)) {
        throw new Error(
            `agent-pool.yaml not found in ${projectPath}\n` +
            `Run 'ckrv setup' to configure your agent pool first.`
        );
    }

    const fileContents = fs.readFileSync(configPath, 'utf8');
    const config = yaml.load(fileContents) as AgentPoolConfig;

    // Basic validation
    if (!config.agents || !config.agents.planner) {
        throw new Error('Invalid agent-pool.yaml: missing planner configuration');
    }

    return config;
};

/**
 * Execute an agent with a given prompt
 */
export const executeAgent = async (
    provider: string,
    model: string | undefined,
    prompt: string,
    cwd: string = process.cwd()
): Promise<void> => {
    const spinner = ora(`Executing ${provider}...`).start();

    try {
        // Build command based on provider
        const { command, args } = buildAgentCommand(provider, model, prompt);

        spinner.text = `Running ${provider} with ${model || 'default model'}...`;

        // Execute the agent CLI
        const subprocess = execa(command, args, {
            cwd,
            stdio: 'inherit', // Stream output directly to user
        });

        await subprocess;
        spinner.succeed(`${provider} completed successfully`);
    } catch (error: any) {
        spinner.fail(`${provider} failed`);
        if (error.stderr) {
            console.error(chalk.red(error.stderr));
        }
        throw error;
    }
};

/**
 * Build command and args for different agent providers
 */
function buildAgentCommand(
    provider: string,
    model: string | undefined,
    prompt: string
): { command: string; args: string[] } {
    switch (provider) {
        case 'gemini-cli':
            return {
                command: 'gemini',
                args: model ? ['-m', model, '-p', prompt] : ['-p', prompt],
            };

        case 'codex-cli':
        case 'codex':
            return {
                command: 'codex',
                args: model ? ['-m', model, prompt] : [prompt],
            };

        case 'opencode':
            // OpenCode uses interactive mode by default
            return {
                command: 'opencode',
                args: model ? ['--model', model, prompt] : [prompt],
            };

        case 'claude-code':
        case 'claude':
            return {
                command: 'claude',
                args: model ? ['--model', model, prompt] : [prompt],
            };

        default:
            throw new Error(`Unsupported provider: ${provider}`);
    }
}
