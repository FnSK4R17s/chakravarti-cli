import chalk from 'chalk';
import execa from 'execa';
import { loadAgentPool } from '../orchestration/executor';
import { buildSystemPrompt } from '../orchestration/prompts';

export const chatCommand = async (
    options: { agent?: string; path?: string } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();

    try {
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

        console.log(chalk.bold(`\n💬 Starting interactive chat with ${agentName}\n`));
        console.log(chalk.dim(`Provider: ${agentConfig.provider}`));
        console.log(chalk.dim(`Model: ${agentConfig.model || 'default'}`));
        console.log(chalk.dim(`Role: ${role}`));
        console.log(chalk.dim(`Type your messages and press Enter. Press Ctrl+C to exit.\n`));

        // Build initial system prompt with context
        const systemPrompt = buildSystemPrompt('', {
            role,
            agentName,
            projectName: config.project.name,
            projectDescription: config.project.description,
            projectPath,
        });

        // Generate GEMINI.md if using Gemini CLI
        if (agentConfig.provider === 'gemini-cli') {
            await generateGeminiMd(
                projectPath,
                role,
                systemPrompt,
                config.project.name,
                config.project.description
            );
        }

        // Start interactive session based on provider
        await startInteractiveSession(
            agentConfig.provider,
            agentConfig.model,
            systemPrompt,
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

/**
 * Start an interactive session with the agent CLI
 */
async function startInteractiveSession(
    provider: string,
    model: string | undefined,
    systemPrompt: string,
    cwd: string
): Promise<void> {
    // For now, we'll launch the CLI in interactive mode
    // The system prompt will be sent as the first message

    switch (provider) {
        case 'gemini-cli':
            // Gemini CLI supports interactive mode
            const geminiArgs = model ? ['-m', model] : [];

            // Write system prompt to a temp file to send as first message
            const fs = require('fs');
            const path = require('path');
            const tmpFile = path.join(cwd, '.chakravarti', 'logs', 'system-prompt.txt');
            fs.mkdirSync(path.dirname(tmpFile), { recursive: true });
            fs.writeFileSync(tmpFile, systemPrompt);

            console.log(chalk.cyan('\n📝 System context loaded. Starting chat...\n'));
            console.log(chalk.bold('💡 Quick Start Guide:\n'));
            console.log(chalk.dim('  • The agent knows your project structure and role'));
            console.log(chalk.dim('  • Ask questions or give instructions naturally'));
            console.log(chalk.dim('  • Example: "Create a todo list app with React"'));
            console.log(chalk.dim('  • Example: "Update sprint-001.md with these tasks"'));
            console.log(chalk.dim('  • Press Ctrl+C to exit anytime\n'));
            console.log(chalk.yellow('Starting Gemini CLI...\n'));

            // Launch gemini in interactive mode
            await execa('gemini', geminiArgs, {
                cwd,
                stdio: 'inherit',
            });
            break;

        case 'codex-cli':
        case 'codex':
            const codexArgs = model ? ['-m', model] : [];
            await execa('codex', codexArgs, {
                cwd,
                stdio: 'inherit',
            });
            break;

        case 'opencode':
            const opencodeArgs = model ? ['--model', model] : [];
            await execa('opencode', opencodeArgs, {
                cwd,
                stdio: 'inherit',
            });
            break;

        default:
            throw new Error(`Interactive mode not supported for provider: ${provider}`);
    }
}

/**
 * Generate GEMINI.md file for Gemini CLI context
 */
async function generateGeminiMd(
    projectPath: string,
    role: 'planner' | 'executor' | 'tester',
    roleInstructions: string,
    projectName: string,
    projectDescription: string
): Promise<void> {
    const fs = require('fs');
    const path = require('path');

    // Read template
    const templatePath = path.join(__dirname, '../templates/GEMINI.md');
    let template = fs.readFileSync(templatePath, 'utf8');

    // Get project structure
    const { getProjectStructure } = await import('../orchestration/prompts');
    const structure = getProjectStructure(projectPath);

    // Replace placeholders
    template = template
        .replace(/{{ROLE}}/g, role.charAt(0).toUpperCase() + role.slice(1))
        .replace(/{{ROLE_INSTRUCTIONS}}/g, roleInstructions)
        .replace(/{{PROJECT_NAME}}/g, projectName)
        .replace(/{{PROJECT_DESCRIPTION}}/g, projectDescription)
        .replace(/{{PROJECT_PATH}}/g, projectPath)
        .replace(/{{PROJECT_STRUCTURE}}/g, structure);

    // Write to project root
    const geminiMdPath = path.join(projectPath, 'GEMINI.md');
    fs.writeFileSync(geminiMdPath, template);

    console.log(chalk.green('✓ Generated GEMINI.md with agent context\n'));
}

