#!/usr/bin/env npx ts-node
/**
 * @file extract-terminal-colors.ts
 * @description
 * Extracts color values from index.css and converts them to hex format
 * suitable for xterm.js terminal themes.
 *
 * Usage:
 *   npx ts-node scripts/extract-terminal-colors.ts
 *
 * This script:
 * 1. Reads index.css to find --background and --foreground variables
 * 2. Converts oklch() values to hex using the culori library
 * 3. Outputs terminal theme config that can be copied to theme.ts
 *
 * @example Output:
 *   Dark Mode Terminal Theme:
 *     background: '#181818'
 *     foreground: '#e8e8e8'
 *
 *   Light Mode Terminal Theme:
 *     background: '#fafafa'
 *     foreground: '#333333'
 */

import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// oklch to hex conversion (simplified approximation)
// For more accurate conversion, install culori: npm install culori
function oklchToHex(l: number, c: number, h: number): string {
    // oklch -> oklab -> linear sRGB -> sRGB -> hex
    // This is a simplified conversion; for production use 'culori' library

    // Convert oklch to oklab
    const hRad = (h * Math.PI) / 180;
    const a = c * Math.cos(hRad);
    const b = c * Math.sin(hRad);

    // oklab to linear sRGB (approximate)
    const L = l;
    const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = L - 0.0894841775 * a - 1.291485548 * b;

    const l3 = l_ * l_ * l_;
    const m3 = m_ * m_ * m_;
    const s3 = s_ * s_ * s_;

    let r = +4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let bVal = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.707614701 * s3;

    // Linear sRGB to sRGB (gamma correction)
    const toSrgb = (x: number): number => {
        if (x <= 0.0031308) return 12.92 * x;
        return 1.055 * Math.pow(x, 1 / 2.4) - 0.055;
    };

    r = Math.max(0, Math.min(1, toSrgb(r)));
    g = Math.max(0, Math.min(1, toSrgb(g)));
    bVal = Math.max(0, Math.min(1, toSrgb(bVal)));

    // Convert to hex
    const toHex = (v: number): string => {
        const hex = Math.round(v * 255).toString(16);
        return hex.length === 1 ? '0' + hex : hex;
    };

    return `#${toHex(r)}${toHex(g)}${toHex(bVal)}`;
}

// Parse oklch() string and convert to hex
function parseOklch(value: string): string | null {
    // Match: oklch(0.182 0 0) or oklch(0.182 0.05 120)
    const match = value.match(/oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*\)/);
    if (!match) return null;

    const l = parseFloat(match[1]);
    const c = parseFloat(match[2]);
    const h = parseFloat(match[3]) || 0;

    return oklchToHex(l, c, h);
}

// Extract CSS variables from a section
function extractVariables(css: string, section: 'root' | 'dark'): Record<string, string> {
    const vars: Record<string, string> = {};

    // Find the appropriate section
    let sectionContent: string;
    if (section === 'root') {
        const rootMatch = css.match(/:root\s*\{([^}]+)\}/s);
        sectionContent = rootMatch?.[1] || '';
    } else {
        const darkMatch = css.match(/\.dark\s*\{([^}]+)\}/s);
        sectionContent = darkMatch?.[1] || '';
    }

    // Extract variable definitions
    const varRegex = /--([\w-]+):\s*([^;]+);/g;
    let match;
    while ((match = varRegex.exec(sectionContent)) !== null) {
        const name = match[1];
        const value = match[2].trim();
        vars[name] = value;
    }

    return vars;
}

// Main function
function main() {
    const cssPath = join(__dirname, '../src/index.css');
    let css: string;

    try {
        css = readFileSync(cssPath, 'utf-8');
    } catch (error) {
        console.error(`Error reading ${cssPath}:`, error);
        process.exit(1);
    }

    // Extract variables from both sections
    const rootVars = extractVariables(css, 'root');
    const darkVars = extractVariables(css, 'dark');

    // Key variables to extract for terminal themes
    const terminalVars = ['background', 'foreground', 'muted', 'muted-foreground', 'primary'];

    console.log('='.repeat(60));
    console.log('TERMINAL THEME COLORS');
    console.log('Extracted from index.css and converted to hex for xterm.js');
    console.log('='.repeat(60));
    console.log('');

    // Light mode (from :root)
    console.log('📌 LIGHT MODE (from :root):');
    console.log('─'.repeat(40));
    for (const varName of terminalVars) {
        const value = rootVars[varName];
        if (value) {
            const hex = parseOklch(value);
            if (hex) {
                console.log(`  --${varName}: ${value}`);
                console.log(`    → ${hex}`);
            }
        }
    }
    console.log('');

    // Dark mode (from .dark)
    console.log('🌙 DARK MODE (from .dark):');
    console.log('─'.repeat(40));
    for (const varName of terminalVars) {
        const value = darkVars[varName];
        if (value) {
            const hex = parseOklch(value);
            if (hex) {
                console.log(`  --${varName}: ${value}`);
                console.log(`    → ${hex}`);
            }
        }
    }
    console.log('');

    // Generate copy-paste ready config
    console.log('📋 COPY-PASTE CONFIG FOR theme.ts:');
    console.log('─'.repeat(40));

    const darkBg = parseOklch(darkVars['background'] || '') || '#181818';
    const darkFg = parseOklch(darkVars['foreground'] || '') || '#e8e8e8';
    const lightBg = parseOklch(rootVars['background'] || '') || '#fafafa';
    const lightFg = parseOklch(rootVars['foreground'] || '') || '#333333';

    console.log(`
dark: {
    background: '${darkBg}',
    foreground: '${darkFg}',
    cursor: '${darkFg}',
    cursorAccent: '${darkBg}',
    // ... ANSI colors remain the same
},

light: {
    background: '${lightBg}',
    foreground: '${lightFg}',
    cursor: '${lightFg}',
    cursorAccent: '${lightBg}',
    // ... ANSI colors remain the same
},
`);

    console.log('='.repeat(60));
    console.log('Done! Update TERMINAL_THEMES in src/lib/theme.ts with these values.');
    console.log('='.repeat(60));
}

main();
