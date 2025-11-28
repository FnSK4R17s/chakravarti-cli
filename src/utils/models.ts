interface ModelInfo {
    id: string;
    name: string;
}

/**
 * Get available models for a given agent/provider
 * Note: Most CLIs don't expose model listing commands, so we use curated lists
 * Users should refer to each provider's documentation for the latest models
 */
export const getAvailableModels = async (provider: string): Promise<ModelInfo[]> => {
    switch (provider) {
        case 'gemini-cli':
            // Gemini models - use /model in gemini CLI to see available models
            return [
                { id: 'gemini-3-pro-preview', name: 'Gemini 3 Pro (Preview)' },
                { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro' },
                { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash' },
                { id: 'gemini-2.5-flash-lite', name: 'Gemini 2.5 Flash Lite' },
            ];

        case 'codex-cli':
        case 'codex':
            // Codex models - use /model in codex CLI to see available models
            return [
                { id: 'gpt-5.1-codex-max', name: 'GPT-5.1 Codex Max (Latest)' },
                { id: 'gpt-5.1-codex', name: 'GPT-5.1 Codex' },
                { id: 'gpt-5.1-codex-mini', name: 'GPT-5.1 Codex Mini' },
                { id: 'gpt-5.1', name: 'GPT-5.1' },
            ];

        case 'claude-code':
        case 'claude':
            // Claude models - check https://docs.anthropic.com/claude/docs/models-overview
            return [
                { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet (Latest)' },
                { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus' },
                { id: 'claude-3-sonnet-20240229', name: 'Claude 3 Sonnet' },
                { id: 'claude-3-haiku-20240307', name: 'Claude 3 Haiku' },
            ];

        case 'opencode':
            // OpenCode free models
            return [
                { id: 'opencode-zen', name: 'OpenCode Zen (Big Pickle)' },
                { id: 'esm-code-fast-1', name: 'Grok Code Fast 1' },
            ];
        default:
            return [{ id: 'default', name: 'Default Model' }];
    }
};
