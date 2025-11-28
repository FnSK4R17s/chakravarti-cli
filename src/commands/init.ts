import fs from 'fs';
import path from 'path';
import { simpleGit } from 'simple-git';
import chalk from 'chalk';
import ora from 'ora';

export const initCommand = async (projectPath: string = process.cwd()): Promise<void> => {
    const spinner = ora('Initializing Chakravarti project...').start();

    try {
        // 1. Check/Initialize Git
        const git = simpleGit(projectPath);
        const isRepo = await git.checkIsRepo();

        if (!isRepo) {
            spinner.text = 'Initializing git repository...';
            await git.init();
            spinner.succeed('Git repository initialized');
        } else {
            spinner.info('Git repository already exists');
        }

        // 2. Create .chakravarti directory structure
        spinner.start('Creating .chakravarti directory structure...');
        const chakravartiDir = path.join(projectPath, '.chakravarti');
        const sprintsDir = path.join(chakravartiDir, 'sprints');
        const logsDir = path.join(chakravartiDir, 'logs');

        fs.mkdirSync(chakravartiDir, { recursive: true });
        fs.mkdirSync(sprintsDir, { recursive: true });
        fs.mkdirSync(logsDir, { recursive: true });

        spinner.succeed('Created .chakravarti directory structure');

        // 3. Generate agent-pool.yaml
        spinner.start('Generating agent-pool.yaml...');
        const templatePath = path.join(__dirname, '../templates/agent-pool.yaml');
        const targetPath = path.join(projectPath, 'agent-pool.yaml');

        if (fs.existsSync(targetPath)) {
            spinner.warn('agent-pool.yaml already exists, skipping...');
        } else {
            let template = fs.readFileSync(templatePath, 'utf-8');

            // Get project name from directory
            const projectName = path.basename(projectPath);
            template = template.replace('{{PROJECT_NAME}}', projectName);
            template = template.replace('{{PROJECT_DESCRIPTION}}', `AI-powered development for ${projectName}`);

            fs.writeFileSync(targetPath, template);
            spinner.succeed('Generated agent-pool.yaml');
        }

        // 4. Create initial sprint template
        spinner.start('Creating initial sprint template...');
        const sprintTemplatePath = path.join(__dirname, '../templates/sprint.md');
        const sprintPath = path.join(sprintsDir, 'sprint-001.md');

        if (!fs.existsSync(sprintPath)) {
            let sprintTemplate = fs.readFileSync(sprintTemplatePath, 'utf-8');
            const now = new Date();
            const endDate = new Date(now.getTime() + 14 * 24 * 60 * 60 * 1000); // 2 weeks

            sprintTemplate = sprintTemplate.replace(/{{SPRINT_ID}}/g, '001');
            sprintTemplate = sprintTemplate.replace(/{{SPRINT_NAME}}/g, 'Initial Setup');
            sprintTemplate = sprintTemplate.replace('{{START_DATE}}', now.toISOString().split('T')[0]);
            sprintTemplate = sprintTemplate.replace('{{END_DATE}}', endDate.toISOString().split('T')[0]);

            fs.writeFileSync(sprintPath, sprintTemplate);
            spinner.succeed('Created initial sprint template');
        } else {
            spinner.info('Sprint template already exists');
        }

        // 5. Create .gitignore entry for logs
        const gitignorePath = path.join(projectPath, '.gitignore');
        const gitignoreEntry = '\n# Chakravarti\n.chakravarti/logs/\n';

        if (fs.existsSync(gitignorePath)) {
            const gitignoreContent = fs.readFileSync(gitignorePath, 'utf-8');
            if (!gitignoreContent.includes('.chakravarti/logs/')) {
                fs.appendFileSync(gitignorePath, gitignoreEntry);
            }
        } else {
            fs.writeFileSync(gitignorePath, gitignoreEntry);
        }

        console.log('\n' + chalk.green('✓ Chakravarti project initialized successfully!'));
        console.log(chalk.dim('\nNext steps:'));
        console.log(chalk.cyan('  1. Review and customize agent-pool.yaml'));
        console.log(chalk.cyan('  2. Edit .chakravarti/sprints/sprint-001.md to define your first tasks'));
        console.log(chalk.cyan('  3. Run ckrv run to start the orchestration'));

    } catch (error) {
        spinner.fail('Failed to initialize project');
        console.error(error);
        throw error;
    }
};
