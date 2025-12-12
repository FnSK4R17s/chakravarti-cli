import simpleGit, { SimpleGit } from 'simple-git';
import fs from 'fs';
import path from 'path';
import chalk from 'chalk';

export interface WorktreeConfig {
    executorName: string;
    branchName: string;
    basePath: string;
    projectPath: string;
}

export interface Worktree {
    path: string;
    branch: string;
    commit: string;
    bare?: boolean;
}

/**
 * Create a git worktree for an executor
 */
export async function createWorktree(config: WorktreeConfig): Promise<string> {
    const git: SimpleGit = simpleGit(config.projectPath);

    // Ensure worktree base directory exists
    const worktreeBase = path.join(config.projectPath, config.basePath);
    if (!fs.existsSync(worktreeBase)) {
        fs.mkdirSync(worktreeBase, { recursive: true });
    }

    // Create worktree path
    const worktreePath = path.join(worktreeBase, config.executorName);

    // Check if worktree already exists
    if (fs.existsSync(worktreePath)) {
        console.log(chalk.yellow(`⚠ Worktree already exists at ${worktreePath}`));
        return worktreePath;
    }

    try {
        // Create new branch and worktree
        await git.raw(['worktree', 'add', '-b', config.branchName, worktreePath]);
        console.log(chalk.green(`✓ Created worktree: ${worktreePath}`));
        console.log(chalk.dim(`  Branch: ${config.branchName}`));

        return worktreePath;
    } catch (error: any) {
        throw new Error(`Failed to create worktree: ${error.message}`);
    }
}

/**
 * List all git worktrees
 */
export async function listWorktrees(projectPath: string): Promise<Worktree[]> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        const output = await git.raw(['worktree', 'list', '--porcelain']);
        return parseWorktreeList(output);
    } catch (error: any) {
        throw new Error(`Failed to list worktrees: ${error.message}`);
    }
}

/**
 * Remove a git worktree
 */
export async function removeWorktree(projectPath: string, worktreePath: string): Promise<void> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        // Remove worktree
        await git.raw(['worktree', 'remove', worktreePath, '--force']);
        console.log(chalk.green(`✓ Removed worktree: ${worktreePath}`));
    } catch (error: any) {
        throw new Error(`Failed to remove worktree: ${error.message}`);
    }
}

/**
 * Prune stale worktree references
 */
export async function pruneWorktrees(projectPath: string): Promise<void> {
    const git: SimpleGit = simpleGit(projectPath);

    try {
        await git.raw(['worktree', 'prune']);
        console.log(chalk.green('✓ Pruned stale worktrees'));
    } catch (error: any) {
        throw new Error(`Failed to prune worktrees: ${error.message}`);
    }
}

/**
 * Sync worktree with remote
 */
export async function syncWorktree(worktreePath: string, remote: string = 'lab'): Promise<void> {
    const git: SimpleGit = simpleGit(worktreePath);

    try {
        // Get current branch
        const status = await git.status();
        const branch = status.current;

        if (!branch) {
            throw new Error('No current branch found');
        }

        // Pull from remote
        await git.pull(remote, branch);
        console.log(chalk.green(`✓ Synced worktree with ${remote}/${branch}`));
    } catch (error: any) {
        throw new Error(`Failed to sync worktree: ${error.message}`);
    }
}

/**
 * Parse worktree list output
 */
function parseWorktreeList(output: string): Worktree[] {
    const worktrees: Worktree[] = [];
    const lines = output.split('\n');

    let current: Partial<Worktree> = {};

    for (const line of lines) {
        if (line.startsWith('worktree ')) {
            if (current.path) {
                worktrees.push(current as Worktree);
            }
            current = { path: line.substring(9) };
        } else if (line.startsWith('HEAD ')) {
            current.commit = line.substring(5);
        } else if (line.startsWith('branch ')) {
            current.branch = line.substring(7);
        } else if (line === 'bare') {
            current.bare = true;
        } else if (line === '') {
            if (current.path) {
                worktrees.push(current as Worktree);
                current = {};
            }
        }
    }

    // Add last worktree if exists
    if (current.path) {
        worktrees.push(current as Worktree);
    }

    return worktrees;
}

/**
 * Check if a worktree exists
 */
export async function worktreeExists(projectPath: string, worktreePath: string): Promise<boolean> {
    const worktrees = await listWorktrees(projectPath);
    return worktrees.some(w => w.path === worktreePath);
}

/**
 * Get worktree info
 */
export async function getWorktreeInfo(projectPath: string, worktreePath: string): Promise<Worktree | null> {
    const worktrees = await listWorktrees(projectPath);
    return worktrees.find(w => w.path === worktreePath) || null;
}
