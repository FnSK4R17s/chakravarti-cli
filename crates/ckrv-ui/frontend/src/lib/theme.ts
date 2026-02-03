/**
 * @module theme
 * @description
 * Centralized theme tokens for consistent styling across components.
 * Uses shadcn/ui CSS variables with custom semantic extensions.
 *
 * THEME SWAPPING: The underlying CSS variables come from the installed
 * shadcn theme. Change themes with:
 *   npx shadcn@latest add https://tweakcn.com/r/themes/<theme-name>.json
 *
 * IMPORTANT: Always use these constants instead of hardcoding colors.
 * Use Tailwind classes like `text-primary`, `bg-muted`, `text-destructive`.
 *
 * @see index.css for CSS variable definitions
 * @see https://tweakcn.com for community themes
 *
 * @example
 * // Use Tailwind classes directly (preferred):
 * <span className="text-destructive">Error</span>
 * <span className="text-primary">Active</span>
 * <span className="text-muted-foreground">Muted text</span>
 *
 * // Or use these exports for dynamic mapping:
 * import { getStatusClass } from '@/lib/theme';
 * <span className={getStatusClass(status)}>{status}</span>
 */

// ============================================================
// STATUS MAPPINGS
// ============================================================
// Map execution/task statuses to Tailwind classes

/**
 * Status to Tailwind class mapping.
 * Uses shadcn semantic colors + custom semantic extensions.
 *
 * Color meanings in darkmatter theme:
 * - primary: Orange/warm - main action, active state
 * - secondary: Teal - secondary action
 * - muted-foreground: Gray - inactive, pending
 * - destructive: Teal (in darkmatter) - but we use --error for red
 * - success: Green (custom) - completed
 * - warning: Amber (custom) - warning states
 * - error: Red (custom) - failed states
 * - info: Teal (custom) - info/waiting
 */
export const STATUS_CLASSES = {
    /** Pending - muted gray */
    pending: 'text-muted-foreground',
    /** Waiting - info/teal */
    waiting: 'text-info',
    /** Running - primary orange (animated) */
    running: 'text-primary',
    /** Completed - success green */
    completed: 'text-success',
    /** Failed - error red */
    failed: 'text-error',
    /** Warning - warning amber */
    warning: 'text-warning',
    /** Highlighted - primary orange */
    highlighted: 'text-primary',
} as const;

/**
 * Status background classes (dim/subtle backgrounds).
 */
export const STATUS_BG_CLASSES = {
    pending: 'bg-muted',
    waiting: 'bg-info/10',
    running: 'bg-primary/10',
    completed: 'bg-success/10',
    failed: 'bg-error/10',
    warning: 'bg-warning/10',
    highlighted: 'bg-primary/10',
} as const;

/**
 * Status border classes.
 */
export const STATUS_BORDER_CLASSES = {
    pending: 'border-border',
    waiting: 'border-info',
    running: 'border-primary',
    completed: 'border-success',
    failed: 'border-error',
    warning: 'border-warning',
    highlighted: 'border-primary',
} as const;

// ============================================================
// LOG TYPE MAPPINGS
// ============================================================
// Map log entry types to Tailwind classes

/**
 * Log type to Tailwind class mapping.
 */
export const LOG_CLASSES = {
    /** Error logs - red */
    error: 'text-error',
    /** Success logs - green */
    success: 'text-success',
    /** Warning logs - amber */
    warning: 'text-warning',
    /** Info/debug logs - muted gray */
    info: 'text-muted-foreground',
    /** Batch operation logs - primary orange */
    batch: 'text-primary',
    /** Highlighted entries - primary */
    highlight: 'text-primary',
    /** Standard output - foreground */
    stdout: 'text-foreground',
    /** Timestamps - muted */
    timestamp: 'text-muted-foreground',
} as const;

// ============================================================
// AGENT COLOR MAPPINGS
// ============================================================
// Map AI agent providers to Tailwind classes

/**
 * Agent provider color classes.
 * Uses chart colors for visual distinction.
 */
export const AGENT_CLASSES = {
    /** Anthropic Claude - primary (orange) */
    anthropic: {
        text: 'text-primary',
        bg: 'bg-primary/10',
        border: 'border-primary',
    },
    /** OpenAI - success (green) */
    openai: {
        text: 'text-success',
        bg: 'bg-success/10',
        border: 'border-success',
    },
    /** Google Gemini - info (teal) */
    google: {
        text: 'text-info',
        bg: 'bg-info/10',
        border: 'border-info',
    },
    /** OpenRouter - warning (amber) */
    openrouter: {
        text: 'text-warning',
        bg: 'bg-warning/10',
        border: 'border-warning',
    },
    /** Default/unknown - muted */
    default: {
        text: 'text-muted-foreground',
        bg: 'bg-muted',
        border: 'border-border',
    },
} as const;

// ============================================================
// HELPER FUNCTIONS
// ============================================================

export type StatusKey = keyof typeof STATUS_CLASSES;
export type LogType = keyof typeof LOG_CLASSES;

/**
 * Get the text color class for a given status.
 * @example getStatusClass('completed') // 'text-success'
 */
export function getStatusClass(status: string): string {
    return STATUS_CLASSES[status as StatusKey] ?? STATUS_CLASSES.pending;
}

/**
 * Get the background class for a given status.
 * @example getStatusBgClass('running') // 'bg-primary/10'
 */
export function getStatusBgClass(status: string): string {
    return STATUS_BG_CLASSES[status as StatusKey] ?? STATUS_BG_CLASSES.pending;
}

/**
 * Get the border class for a given status.
 * @example getStatusBorderClass('failed') // 'border-error'
 */
export function getStatusBorderClass(status: string): string {
    return STATUS_BORDER_CLASSES[status as StatusKey] ?? STATUS_BORDER_CLASSES.pending;
}

/**
 * Get the text color class for a given log type.
 * @example getLogClass('error') // 'text-error'
 */
export function getLogClass(type: string): string {
    return LOG_CLASSES[type as LogType] ?? LOG_CLASSES.info;
}

/**
 * Get agent styling classes for a provider.
 * @example getAgentClasses('anthropic') // { text: 'text-primary', ... }
 */
export function getAgentClasses(provider: string): { text: string; bg: string; border: string } {
    const key = provider.toLowerCase();
    if (key.includes('anthropic') || key.includes('claude')) {
        return AGENT_CLASSES.anthropic;
    }
    if (key.includes('openai') || key.includes('gpt')) {
        return AGENT_CLASSES.openai;
    }
    if (key.includes('google') || key.includes('gemini')) {
        return AGENT_CLASSES.google;
    }
    if (key.includes('openrouter')) {
        return AGENT_CLASSES.openrouter;
    }
    return AGENT_CLASSES.default;
}

// ============================================================
// LEGACY ALIASES (for backwards compatibility)
// ============================================================
// These will be removed in a future version

/** @deprecated Use STATUS_CLASSES instead */
export const STATUS_COLORS = STATUS_CLASSES;
/** @deprecated Use STATUS_BG_CLASSES instead */
export const STATUS_BG = STATUS_BG_CLASSES;
/** @deprecated Use STATUS_BORDER_CLASSES instead */
export const STATUS_BORDER = STATUS_BORDER_CLASSES;
/** @deprecated Use LOG_CLASSES instead */
export const LOG_COLORS = LOG_CLASSES;
/** @deprecated Use getStatusClass instead */
export const getStatusColor = getStatusClass;
/** @deprecated Use getStatusBgClass instead */
export const getStatusBg = getStatusBgClass;
/** @deprecated Use getStatusBorderClass instead */
export const getStatusBorder = getStatusBorderClass;
/** @deprecated Use getLogClass instead */
export const getLogColor = getLogClass;

// ============================================================
// TERMINAL THEME CONFIGURATION
// ============================================================
// xterm.js requires hex color values - these are hardcoded exceptions.
//
// ⚠️ MAINTENANCE NOTE: When changing the shadcn theme, run the color
// extraction script to regenerate these values:
//
//   npx ts-node --esm scripts/extract-terminal-colors.ts
//
// The script reads oklch values from index.css and converts them to hex.
//
// Current theme: darkmatter (from tweakcn.com)
// Last synced: 2026-02-03

/**
 * Terminal themes for xterm.js.
 * These are hardcoded hex values that must match the installed shadcn theme.
 * 
 * @example
 * import { TERMINAL_THEMES, getTerminalTheme } from '@/lib/theme';
 * const theme = getTerminalTheme(); // Auto-detects dark/light mode
 * const term = new Terminal({ theme });
 */
export const TERMINAL_THEMES = {
    /**
     * Dark mode terminal theme.
     * Synced with darkmatter dark theme CSS variables.
     * Run: npx ts-node --esm scripts/extract-terminal-colors.ts to regenerate.
     */
    dark: {
        // Core colors extracted from --background, --foreground oklch values
        background: '#121113',    // oklch(0.1797 0.0043 308.1928)
        foreground: '#c1c1c1',    // oklch(0.8109 0 0)
        cursor: '#c1c1c1',
        cursorAccent: '#121113',
        selectionBackground: '#3d4a5c',  // muted selection

        // ANSI colors - VS Code dark theme style
        black: '#181818',
        red: '#f44747',
        green: '#608b4e',
        yellow: '#dcdcaa',
        blue: '#569cd6',
        magenta: '#c586c0',
        cyan: '#4ec9b0',
        white: '#d4d4d4',

        // Bright variants
        brightBlack: '#6a6a6a',
        brightRed: '#f44747',
        brightGreen: '#608b4e',
        brightYellow: '#dcdcaa',
        brightBlue: '#569cd6',
        brightMagenta: '#c586c0',
        brightCyan: '#4ec9b0',
        brightWhite: '#ffffff',
    },

    /**
     * Light mode terminal theme.
     * Synced with darkmatter light theme CSS variables.
     * Run: npx ts-node --esm scripts/extract-terminal-colors.ts to regenerate.
     */
    light: {
        // Core colors extracted from --background, --foreground oklch values
        background: '#ffffff',    // oklch(1.0000 0 0)
        foreground: '#111827',    // oklch(0.2101 0.0318 264.6645)
        cursor: '#111827',
        cursorAccent: '#ffffff',
        selectionBackground: '#d4e0f0',  // soft blue selection

        // ANSI colors - VS Code light theme style
        black: '#333333',
        red: '#cd3131',
        green: '#14a614',
        yellow: '#b5a000',
        blue: '#0451a5',
        magenta: '#bc05bc',
        cyan: '#0598bc',
        white: '#f8f8f8',

        // Bright variants
        brightBlack: '#666666',
        brightRed: '#cd3131',
        brightGreen: '#14a614',
        brightYellow: '#b5a000',
        brightBlue: '#0451a5',
        brightMagenta: '#bc05bc',
        brightCyan: '#0598bc',
        brightWhite: '#ffffff',
    },
} as const;

/** Terminal theme interface - compatible with xterm.js ITheme */
export interface TerminalTheme {
    background: string;
    foreground: string;
    cursor: string;
    cursorAccent: string;
    selectionBackground: string;
    black: string;
    red: string;
    green: string;
    yellow: string;
    blue: string;
    magenta: string;
    cyan: string;
    white: string;
    brightBlack: string;
    brightRed: string;
    brightGreen: string;
    brightYellow: string;
    brightBlue: string;
    brightMagenta: string;
    brightCyan: string;
    brightWhite: string;
}

/**
 * Get the appropriate terminal theme based on current dark/light mode.
 * Checks for 'dark' class on document.documentElement.
 * 
 * @example
 * const term = new Terminal({ theme: getTerminalTheme() });
 */
export function getTerminalTheme(): TerminalTheme {
    if (typeof document === 'undefined') {
        return TERMINAL_THEMES.dark; // SSR fallback
    }
    const isDark = document.documentElement.classList.contains('dark');
    return isDark ? TERMINAL_THEMES.dark : TERMINAL_THEMES.light;
}

/**
 * Check if current mode is dark.
 */
export function isDarkMode(): boolean {
    if (typeof document === 'undefined') return true;
    return document.documentElement.classList.contains('dark');
}

