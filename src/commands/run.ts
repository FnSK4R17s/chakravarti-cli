import chalk from 'chalk';
import { showBanner } from '../ui/banner';
import { loadAgentPool, executeAgent } from '../orchestration/executor';
import { buildSystemPrompt } from '../orchestration/prompts';

export const runCommand = async (
    prompt: string,
    options: { agent?: string; path?: string } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();

    try {
        // Validate prompt
        if (!prompt || prompt.trim().length === 0) {
            throw new Error('Prompt cannot be empty');
        }

        // Load agent pool configuration
        const config = loadAgentPool(projectPath);

        // Determine which agent to use
        let agentConfig;
        let agentName;
        let role: 'planner' | 'executor' | 'tester';

        if (options.agent) {
            // Use specified agent
            if (options.agent === 'planner') {
                agentConfig = config.agents.planner;
                agentName = 'Planner';
                role = 'planner';
            } else if (options.agent === 'tester') {
                agentConfig = config.agents.tester;
                agentName = 'Tester';
                role = 'tester';
            } else {
                // Check if it's an executor
                const executor = config.agents.executors.find(
                    e => e.name === options.agent
                );
                if (executor) {
                    agentConfig = executor;
                    agentName = executor.name;
                    role = 'executor';
                } else {
                    throw new Error(
                        `Agent '${options.agent}' not found in agent-pool.yaml\n` +
                        `Available agents: planner, tester, ${config.agents.executors.map(e => e.name).join(', ')}`
                    );
                }
            }
        } else {
            // Default to planner
            agentConfig = config.agents.planner;
            agentName = 'Planner';
            role = 'planner';
        }

        console.log(chalk.bold(`\n🤖 Running ${agentName}\n`));
        console.log(chalk.dim(`Provider: ${agentConfig.provider}`));
        console.log(chalk.dim(`Model: ${agentConfig.model || 'default'}`));
        console.log(chalk.dim(`Role: ${role}`));
        console.log(chalk.dim(`User Prompt: ${prompt}\n`));

        // Build wrapped prompt with system instructions
        const wrappedPrompt = buildSystemPrompt(prompt, {
            role,
            agentName,
            projectName: config.project.name,
            projectDescription: config.project.description,
            projectPath,
        });

        // Execute the agent with wrapped prompt
        await executeAgent(
            agentConfig.provider,
            agentConfig.model,
            wrappedPrompt,
            projectPath
        );

    } catch (error: any) {
        console.error(chalk.red(`\n✖ Error: ${error.message}`));

        if (error.message.includes('agent-pool.yaml not found')) {
            console.log(chalk.yellow('\nTip: Run `ckrv init` to initialize a project first.'));
        }

        process.exit(1);
    }
};
