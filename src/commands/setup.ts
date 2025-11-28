import prompts from 'prompts';
import fs from 'fs';
import path from 'path';
import chalk from 'chalk';
import ora from 'ora';
import { checkAgents } from '../agents';
import { showBanner } from '../ui/banner';

interface AgentAssignment {
    role: 'planner' | 'executor' | 'tester';
    agentName: string;
    provider: string;
}

export const setupCommand = async (projectPath: string = process.cwd()): Promise<void> => {
    showBanner();

    console.log(chalk.bold('\n🔧 Interactive Agent Setup\n'));
    console.log(chalk.dim('Let\'s configure your agent pool based on installed tools.\n'));

    // 1. Detect installed agents
    const spinner = ora('Detecting installed CLI tools...').start();
    const results = await checkAgents();

    // Filter out non-AI tools (Git, VS Code, etc.)
    const nonAITools = ['git-version-control', 'visual-studio-code', 'visual-studio-code-insiders', 'github-copilot', 'cursor'];
    const availableAgents = results.filter(r => {
        const agentId = r.agent.name.toLowerCase().replace(/\s+/g, '-');
        return r.available && !nonAITools.includes(agentId);
    });

    spinner.succeed(`Found ${availableAgents.length} AI agent(s)`);

    if (availableAgents.length === 0) {
        console.log(chalk.yellow('\n⚠ No AI CLI tools detected!'));
        console.log(chalk.dim('Please install at least one AI CLI tool (e.g., gemini, claude, codex) to continue.'));
        return;
    }

    console.log(chalk.dim('\nAvailable tools:'));
    availableAgents.forEach(a => {
        console.log(chalk.cyan(`  • ${a.agent.name}`));
    });

    // 2. Ask user to assign roles
    console.log(chalk.bold('\n📋 Role Assignment\n'));

    const agentChoices = availableAgents.map(a => ({
        title: a.agent.name,
        value: a.agent.name.toLowerCase().replace(/\s+/g, '-'),
    }));

    // First, ask for planner
    const plannerResponse = await prompts({
        type: 'select',
        name: 'planner',
        message: 'Which agent should be the Planner? (specs requirements with user)',
        choices: agentChoices,
    });

    if (!plannerResponse.planner) {
        console.log(chalk.red('\n✖ Setup cancelled'));
        return;
    }

    // Ask for planner model
    const { getAvailableModels } = await import('../utils/models');
    const plannerModels = await getAvailableModels(plannerResponse.planner);
    const plannerModelChoices = plannerModels.map(m => ({
        title: m.name,
        value: m.id,
    }));

    const plannerModelResponse = await prompts({
        type: 'select',
        name: 'model',
        message: 'Which model for the Planner?',
        choices: plannerModelChoices,
    });

    if (!plannerModelResponse.model) {
        console.log(chalk.red('\n✖ Setup cancelled'));
        return;
    }

    // Ask how many executors
    const executorCountResponse = await prompts({
        type: 'number',
        name: 'count',
        message: 'How many Executors do you need? (1-5 recommended)',
        initial: 2,
        min: 1,
        validate: (value) => value >= 1 || 'Must have at least 1 executor',
    });

    if (!executorCountResponse.count) {
        console.log(chalk.red('\n✖ Setup cancelled'));
        return;
    }

    const executorCount = executorCountResponse.count;
    if (executorCount > 5) {
        console.log(chalk.yellow(`⚠ Warning: Using ${executorCount} executors may be resource-intensive.`));
    }

    // Ask for each executor individually
    const executors: Array<{ provider: string; model: string }> = [];
    for (let i = 0; i < executorCount; i++) {
        const executorResponse = await prompts({
            type: 'select',
            name: 'executor',
            message: `Which agent for Executor ${i + 1}?`,
            choices: agentChoices,
        });

        if (!executorResponse.executor) {
            console.log(chalk.red('\n✖ Setup cancelled'));
            return;
        }

        // Get available models for this executor
        const { getAvailableModels } = await import('../utils/models');
        const models = await getAvailableModels(executorResponse.executor);

        const modelChoices = models.map(m => ({
            title: m.name,
            value: m.id,
        }));

        const modelResponse = await prompts({
            type: 'select',
            name: 'model',
            message: `Which model for Executor ${i + 1}?`,
            choices: modelChoices,
        });

        if (!modelResponse.model) {
            console.log(chalk.red('\n✖ Setup cancelled'));
            return;
        }

        executors.push({
            provider: executorResponse.executor,
            model: modelResponse.model,
        });
    }

    // Ask for tester
    const testerResponse = await prompts({
        type: 'select',
        name: 'tester',
        message: 'Which agent should be the Tester? (writes tests & verifies)',
        choices: agentChoices,
    });

    if (!testerResponse.tester) {
        console.log(chalk.red('\n✖ Setup cancelled'));
        return;
    }

    // Ask for tester model
    const testerModels = await getAvailableModels(testerResponse.tester);
    const testerModelChoices = testerModels.map(m => ({
        title: m.name,
        value: m.id,
    }));

    const testerModelResponse = await prompts({
        type: 'select',
        name: 'model',
        message: 'Which model for the Tester?',
        choices: testerModelChoices,
    });

    if (!testerModelResponse.model) {
        console.log(chalk.red('\n✖ Setup cancelled'));
        return;
    }

    // Ask for project details
    const projectResponse = await prompts([
        {
            type: 'text',
            name: 'projectName',
            message: 'Project name?',
            initial: path.basename(projectPath),
        },
        {
            type: 'text',
            name: 'projectDescription',
            message: 'Project description?',
            initial: `AI-powered development for ${path.basename(projectPath)}`,
        },
    ]);

    if (!projectResponse.projectName) {
        console.log(chalk.red('\n✖ Setup cancelled'));
        return;
    }

    const responses = {
        planner: {
            provider: plannerResponse.planner,
            model: plannerModelResponse.model,
        },
        executors: executors,
        tester: {
            provider: testerResponse.tester,
            model: testerModelResponse.model,
        },
        projectName: projectResponse.projectName,
        projectDescription: projectResponse.projectDescription,
    };

    // 3. Generate agent-pool.yaml
    spinner.start('Generating agent-pool.yaml...');

    const config = {
        version: 1.0,
        project: {
            name: responses.projectName,
            description: responses.projectDescription,
            base_branch: 'ckrv/feature/001-initial-setup',
        },
        agents: {
            planner: {
                role: 'planner',
                provider: responses.planner.provider,
                model: responses.planner.model,
                mode: 'interactive',
                responsibilities: [
                    'Collaborate with user to spec requirements',
                    'Create sprint plans with story points',
                    'Assign tasks to executors',
                    'Mediate tester feedback',
                    'Resolve merge conflicts',
                ],
            },
            executors: responses.executors.map((exec, idx) => ({
                name: `executor-${idx + 1}`,
                role: 'executor',
                provider: exec.provider,
                model: exec.model,
                branch_prefix: `ckrv/exec-${idx + 1}`,
            })),
            tester: {
                role: 'tester',
                provider: responses.tester.provider,
                model: responses.tester.model,
                responsibilities: [
                    'Write comprehensive tests',
                    'Verify implementations',
                    'Report issues to planner',
                ],
            },
        },
        communication: {
            type: 'message_queue',
            backend: 'in-memory',
        },
    };

    const yaml = require('js-yaml');
    const yamlContent = yaml.dump(config, { indent: 2, lineWidth: -1 });

    const targetPath = path.join(projectPath, 'agent-pool.yaml');
    fs.writeFileSync(targetPath, yamlContent);

    spinner.succeed('Generated agent-pool.yaml');

    console.log(chalk.green('\n✓ Agent pool configured successfully!\n'));
    console.log(chalk.bold('Configuration Summary:'));
    console.log(chalk.cyan(`  Planner:   ${responses.planner.provider} (${responses.planner.model})`));
    console.log(chalk.cyan(`  Executors:`));
    responses.executors.forEach((exec, idx) => {
        console.log(chalk.cyan(`    ${idx + 1}. ${exec.provider} (${exec.model})`));
    });
    console.log(chalk.cyan(`  Tester:    ${responses.tester.provider} (${responses.tester.model})`));

    console.log(chalk.dim('\nNext steps:'));
    console.log(chalk.dim('  1. Review agent-pool.yaml'));
    console.log(chalk.dim('  2. Define tasks in .chakravarti/sprints/sprint-001.md'));
    console.log(chalk.dim('  3. Run ckrv run to start orchestration'));
};
