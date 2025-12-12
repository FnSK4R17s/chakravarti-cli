import { exec } from 'child_process';
import { promisify } from 'util';
import chalk from 'chalk';
import ora from 'ora';
import fs from 'fs';
import path from 'path';

const execAsync = promisify(exec);

export const setupGitLabProject = async (projectPath: string): Promise<void> => {
    const spinner = ora('Setting up GitLab project...').start();

    try {
        // 1. Check if GitLab is reachable
        try {
            await execAsync('curl -s --connect-timeout 2 http://localhost:8080/users/sign_in');
        } catch (e) {
            spinner.info('GitLab not reachable (skipping auto-setup)');
            return;
        }

        const projectName = path.basename(projectPath);

        // 2. Create README if missing
        const readmePath = path.join(projectPath, 'README.md');
        if (!fs.existsSync(readmePath)) {
            fs.writeFileSync(readmePath, `# ${projectName}\n\nAI-powered development with Chakravarti.`);
        }

        // 3. Configure Git User
        await execAsync('git config user.name "Chakravarti Admin"', { cwd: projectPath });
        await execAsync('git config user.email "admin@chakravarti.local"', { cwd: projectPath });

        // 4. Create Project via API (using root credentials)
        // First try to get a token or just use basic auth push which auto-creates
        spinner.text = 'Pushing to GitLab (this will auto-create the repo)...';

        // Add remote if missing
        try {
            await execAsync('git remote add lab http://root:chakravarti123@localhost:8080/root/' + projectName + '.git', { cwd: projectPath });
        } catch (e) {
            // Remote might already exist, update it
            await execAsync('git remote set-url lab http://root:chakravarti123@localhost:8080/root/' + projectName + '.git', { cwd: projectPath });
        }

        // 5. Create main branch and push
        try {
            await execAsync('git checkout -b main', { cwd: projectPath });
        } catch (e) {
            await execAsync('git checkout main', { cwd: projectPath });
        }

        // Add and commit
        await execAsync('git add .', { cwd: projectPath });
        try {
            await execAsync('git commit -m "Initial commit"', { cwd: projectPath });
        } catch (e) {
            // Might be nothing to commit
        }

        // Push
        await execAsync('git push -u lab main', { cwd: projectPath });

        spinner.succeed(`GitLab project setup complete: http://localhost:8080/root/${projectName}`);

    } catch (error: any) {
        spinner.warn(`GitLab setup failed: ${error.message}`);
        // Don't throw, just warn
    }
};
