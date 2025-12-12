import simpleGit, { SimpleGit } from 'simple-git';
import chalk from 'chalk';

export interface GitLabConfig {
    url: string;
    token?: string;
}

/**
 * Setup local GitLab as lab origin
 */
export async function setupLabOrigin(projectPath: string, gitlabUrl: string): Promise<void> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        // Check if lab remote already exists
        const remotes = await git.getRemotes(true);
        const labRemote = remotes.find(r => r.name === 'lab');

        if (labRemote) {
            // Update existing remote
            await git.remote(['set-url', 'lab', gitlabUrl]);
            console.log(chalk.green(`✓ Updated lab origin: ${gitlabUrl}`));
        } else {
            // Add new remote
            await git.addRemote('lab', gitlabUrl);
            console.log(chalk.green(`✓ Added lab origin: ${gitlabUrl}`));
        }
    } catch (error: any) {
        throw new Error(`Failed to setup lab origin: ${error.message}`);
    }
}

/**
 * Create branch in lab origin
 */
export async function createLabBranch(
    projectPath: string,
    branchName: string,
    baseBranch: string = 'main'
): Promise<void> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        // Create branch from base
        await git.checkoutBranch(branchName, baseBranch);

        // Push to lab origin
        await git.push('lab', branchName, ['--set-upstream']);

        console.log(chalk.green(`✓ Created branch in lab origin: ${branchName}`));
    } catch (error: any) {
        throw new Error(`Failed to create lab branch: ${error.message}`);
    }
}

/**
 * Push to lab origin
 */
export async function pushToLab(worktreePath: string, branchName: string): Promise<void> {
    const git: SimpleGit = simpleGit(worktreePath);

    try {
        await git.push('lab', branchName);
        console.log(chalk.green(`✓ Pushed to lab/${branchName}`));
    } catch (error: any) {
        throw new Error(`Failed to push to lab: ${error.message}`);
    }
}

/**
 * Pull from lab origin
 */
export async function pullFromLab(worktreePath: string, branchName: string): Promise<void> {
    const git: SimpleGit = simpleGit(worktreePath);

    try {
        await git.pull('lab', branchName);
        console.log(chalk.green(`✓ Pulled from lab/${branchName}`));
    } catch (error: any) {
        throw new Error(`Failed to pull from lab: ${error.message}`);
    }
}

/**
 * Check if lab origin is configured
 */
export async function hasLabOrigin(projectPath: string): Promise<boolean> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        const remotes = await git.getRemotes();
        return remotes.some(r => r.name === 'lab');
    } catch (error: any) {
        return false;
    }
}

/**
 * Get lab origin URL
 */
export async function getLabOriginUrl(projectPath: string): Promise<string | null> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        const remotes = await git.getRemotes(true);
        const labRemote = remotes.find(r => r.name === 'lab');
        return labRemote?.refs.fetch || null;
    } catch (error: any) {
        return null;
    }
}
