import fs from 'fs';
import path from 'path';
import chalk from 'chalk';
import ora from 'ora';
import prompts from 'prompts';
import { simpleGit, SimpleGit } from 'simple-git';
import { executeAgent } from '../orchestration/executor';
import { getSpecPrompts } from '../orchestration/prompts';
import { getLabOriginUrl } from '../git/gitlab';

export const sprintCommand = async (
    featureName?: string,
    options: { path?: string; phase?: string } = {}
): Promise<void> => {
    const projectPath = options.path || process.cwd();
    const spinner = ora('Checking prerequisites...').start();
    const git = simpleGit(projectPath);

    // 1. Prerequisites Check
    try {
        // Check if gitlab is configured
        const labUrl = await getLabOriginUrl(projectPath);
        if (!labUrl) {
            spinner.fail('GitLab origin not configured');
            console.log(chalk.yellow('Please run `ckrv up` to start services and configure gitlab.'));
            return;
        }

        // Check if main branch exists on lab
        try {
            await git.fetch('lab');
            const branches = await git.branch(['-r']);
            if (!branches.all.includes('lab/main')) {
                throw new Error('lab/main branch not found');
            }
        } catch (error) {
            spinner.fail('Prerequisites check failed');
            console.log(chalk.yellow('Please ensure you have pushed your main branch to the internal gitlab:'));
            console.log('git push lab main');
            return;
        }

        spinner.succeed('Prerequisites checked');
    } catch (error: any) {
        spinner.fail(`Failed to check prerequisites: ${error.message}`);
        return;
    }

    // 2. Determine Sprint ID
    const sprintsDir = path.join(projectPath, '.chakravarti', 'sprints');
    if (!fs.existsSync(sprintsDir)) {
        fs.mkdirSync(sprintsDir, { recursive: true });
    }

    const existingSprints = fs.readdirSync(sprintsDir)
        .filter(name => name.startsWith('sprint-') && fs.statSync(path.join(sprintsDir, name)).isDirectory());

    // Find max ID
    let maxId = 0;
    existingSprints.forEach(dir => {
        const match = dir.match(/sprint-(\d+)/);
        if (match) {
            const id = parseInt(match[1]);
            if (id > maxId) maxId = id;
        }
    });

    const nextId = (maxId + 1).toString().padStart(3, '0');
    const sprintId = `sprint-${nextId}`;
    const sprintDir = path.join(sprintsDir, sprintId);

    console.log(chalk.blue(`Preparing ${sprintId}...`));

    // 3. Scaffold Directory
    if (!fs.existsSync(sprintDir)) {
        spinner.start(`Scaffolding ${sprintDir}...`);
        fs.mkdirSync(sprintDir, { recursive: true });

        // Copy templates
        const templatesDir = path.join(__dirname, '../templates/sprint');
        const files = ['spec.md', 'plan.md', 'tasks.md'];
        const targetFiles = ['01-spec.md', '02-plan.md', '03-tasks.md'];

        files.forEach((file, index) => {
            const src = path.join(templatesDir, file);
            const dest = path.join(sprintDir, targetFiles[index]);
            if (fs.existsSync(src)) {
                let content = fs.readFileSync(src, 'utf8');
                // Inject feature name if provided
                if (featureName) {
                    content = content.replace('{{FEATURE_NAME}}', featureName);
                }
                fs.writeFileSync(dest, content);
            }
        });
        spinner.succeed(`Scaffolded ${sprintDir}`);
    } else {
        console.log(chalk.yellow(`Directory ${sprintDir} already exists. Resuming...`));
    }

    // 4. Create/Checkout Branch
    try {
        spinner.start(`Switching to branch ${sprintId}...`);
        // Check if branch exists
        const branches = await git.branchLocal();
        if (branches.all.includes(sprintId)) {
            await git.checkout(sprintId);
        } else {
            // Create from lab/main
            await git.checkoutBranch(sprintId, 'lab/main');
        }
        spinner.succeed(`Switched to branch ${sprintId}`);
    } catch (error: any) {
        spinner.fail(`Failed to manage branch: ${error.message}`);
        return;
    }

    // 5. Execution Phase
    // Get Goal if not provided
    let goal = featureName;
    if (!goal) {
        const response = await prompts({
            type: 'text',
            name: 'goal',
            message: 'What is the goal of this sprint? (Elevator Pitch)',
            validate: (input: string) => input.length > 5 ? true : 'Please provide a descriptive goal.'
        });
        goal = response.goal;
    }

    if (!goal) return; // User cancelled

    // Phase 1: Spec
    await runPhase('specify', sprintDir, '01-spec.md', goal!, 'gemini-cli');

    // Phase 2: Plan
    const proceedToPlan = await prompts({
        type: 'confirm',
        name: 'confirm',
        message: 'Review 01-spec.md. Proceed to Technical Plan?',
        initial: true
    });

    if (proceedToPlan.confirm) {
        await runPhase('plan', sprintDir, '02-plan.md', '', 'gemini-cli');
    } else {
        return;
    }

    // Phase 3: Tasks
    const proceedToTasks = await prompts({
        type: 'confirm',
        name: 'confirm',
        message: 'Review 02-plan.md. Proceed to Task Breakdown?',
        initial: true
    });

    if (proceedToTasks.confirm) {
        await runPhase('tasks', sprintDir, '03-tasks.md', '', 'gemini-cli');
        console.log(chalk.green(`\nSprint prepared successfully!`));
        console.log(chalk.white(`Run 'ckrv assign' to start executing tasks.`));
    }
};

async function runPhase(
    phase: 'specify' | 'plan' | 'tasks',
    sprintDir: string,
    filename: string,
    contextInput: string,
    provider: string
) {
    const filePath = path.join(sprintDir, filename);
    const spinner = ora(`Running Phase: ${phase}...`).start();

    try {
        // Read the template / current file
        const templateContent = fs.readFileSync(filePath, 'utf8');

        // Read previous context if needed
        let previousContext = '';
        if (phase === 'plan') {
            previousContext = fs.readFileSync(path.join(sprintDir, '01-spec.md'), 'utf8');
        } else if (phase === 'tasks') {
            previousContext = fs.readFileSync(path.join(sprintDir, '02-plan.md'), 'utf8');
        }

        // Build Prompt
        const systemPrompt = getSpecPrompts(phase);
        let userPrompt = '';

        if (phase === 'specify') {
            userPrompt = `User Goal: "${contextInput}"\n\nTemplate:\n\`\`\`markdown\n${templateContent}\n\`\`\``;
        } else {
            userPrompt = `Previous Context:\n\`\`\`markdown\n${previousContext}\n\`\`\`\n\nTemplate:\n\`\`\`markdown\n${templateContent}\n\`\`\``;
        }

        // We need to actually run the agent and capture output.
        // The current executor.executeAgent streams to stdio but doesn't return the string.
        // We might need to modify executor.ts or use a temporary file approach.
        // For now, let's assume we can capture it?
        // Wait, executor.ts uses 'inherit' for stdio. We need to capture it to write to file.
        // This is a limitation of the current executor.

        // WORKAROUND: We will invoke the agent command manually here using execa to capture stdout.
        const execaModule = await import('execa');
        const execa = execaModule.default || execaModule;

        // Note: Real implementation should defer to executor.ts logic, but allowing capture.
        // Check if planner container works
        let cmd = 'gemini';
        let args = ['-p', `${systemPrompt}\n\n${userPrompt}`];
        let execFn = execa;

        // Try to find running planner container
        try {
            const { stdout: containerId } = await execa('docker', [
                'ps', '-q', '--filter', 'name=ckrv-planner-planner'
            ]);

            if (containerId.trim()) {
                // Run inside docker
                // Use 'gemini' inside the container. 
                // Note: We need to pass the prompt safely. 
                // Ideally we pipe it to stdin, but 'gemini -p' takes arg.
                cmd = 'docker';
                // interactive mode for TTY? No, just exec.
                args = ['exec', '-i', 'ckrv-planner-planner', 'gemini', '-p', `${systemPrompt}\n\n${userPrompt}`];
                spinner.text = `Generating ${filename} via planner agent (Docker)...`;
            } else {
                spinner.text = `Generating ${filename} via local gemini (Legacy)...`;
            }
        } catch (e) {
            spinner.text = `Generating ${filename} via local gemini (Legacy)...`;
        }

        const logDir = path.join(sprintDir, '../../logs');
        if (!fs.existsSync(logDir)) {
            fs.mkdirSync(logDir, { recursive: true });
        }
        const logFile = path.join(logDir, 'planner.log');
        const timestamp = new Date().toISOString();
        fs.appendFileSync(logFile, `[${timestamp}] Executing: ${cmd} ${args.join(' ')}\n`);

        const result = await execa(cmd, args);

        let output = result.stdout;
        fs.appendFileSync(logFile, `[${timestamp}] Output:\n${output}\n---\n`);

        // Clean markdown fences if agent wraps output
        output = output.replace(/^```markdown\n/, '').replace(/^```\n/, '').replace(/\n```$/, '');

        // Write to file
        fs.writeFileSync(filePath, output);
        spinner.succeed(`Generated ${filename}`);

        // Check for Clarifications
        const clarifications = (output.match(/\[C\]/g) || []).length;
        if (clarifications > 0) {
            console.log(chalk.yellow(`    ⚠ ${clarifications} items marked with [C] for verification.`));
        }

    } catch (error: any) {
        spinner.fail(`Failed phase ${phase}: ${error.message}`);
        throw error;
    }
}
