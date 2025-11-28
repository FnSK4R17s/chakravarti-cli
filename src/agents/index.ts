import { Agent } from '../types';
import commandExists from 'command-exists';

const checkCommand = async (command: string): Promise<boolean> => {
    try {
        await commandExists(command);
        return true;
    } catch {
        return false;
    }
};

export const agents: Agent[] = [
    {
        name: 'Git version control',
        type: 'cli',
        check: () => checkCommand('git'),
    },
    {
        name: 'GitHub Copilot',
        type: 'ide',
        check: async () => false, // IDE-based, usually no CLI check unless we check extensions
    },
    {
        name: 'Claude Code',
        type: 'cli',
        check: () => checkCommand('claude'),
    },
    {
        name: 'Gemini CLI',
        type: 'cli',
        check: () => checkCommand('gemini'), // Assuming 'gemini' binary
    },
    {
        name: 'Cursor',
        type: 'ide',
        check: async () => false,
    },
    {
        name: 'Visual Studio Code',
        type: 'cli',
        check: () => checkCommand('code'),
    },
    {
        name: 'Codex CLI',
        type: 'cli',
        check: () => checkCommand('codex'),
    },
    {
        name: 'opencode',
        type: 'cli',
        check: () => checkCommand('opencode'),
    },
];

export const checkAgents = async (): Promise<{ agent: Agent; available: boolean }[]> => {
    const results = await Promise.all(
        agents.map(async (agent) => {
            const available = await agent.check();
            return { agent, available };
        })
    );
    return results;
};
