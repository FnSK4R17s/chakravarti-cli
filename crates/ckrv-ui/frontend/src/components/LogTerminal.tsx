import { useEffect, useRef, useCallback } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';

interface LogTerminalProps {
    onMount?: (term: Terminal) => void;
    className?: string;
}

export const LogTerminal: React.FC<LogTerminalProps> = ({ onMount, className }) => {
    const terminalRef = useRef<HTMLDivElement>(null);
    const xtermRef = useRef<Terminal | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    // T030: Track debounce timeout for cleanup (BUG-008)
    const fitDebounceRef = useRef<number | null>(null);
    // T029: Track ResizeObserver for cleanup (BUG-008)
    const resizeObserverRef = useRef<ResizeObserver | null>(null);

    // T030: Debounced fit function to prevent excessive calls during rapid resizes (BUG-008)
    const debouncedFit = useCallback(() => {
        // Clear any pending fit
        if (fitDebounceRef.current) {
            window.clearTimeout(fitDebounceRef.current);
        }

        // Schedule new fit after debounce delay
        fitDebounceRef.current = window.setTimeout(() => {
            fitDebounceRef.current = null;
            if (fitAddonRef.current) {
                try {
                    fitAddonRef.current.fit();
                } catch (e) {
                    console.warn('Terminal fit failed:', e);
                }
            }
        }, 50); // 50ms debounce - responsive but prevents rapid-fire calls
    }, []);

    useEffect(() => {
        if (!terminalRef.current || xtermRef.current) return;

        // T039/T040: Get theme colors from CSS variables (BUG-012)
        const computedStyle = getComputedStyle(document.documentElement);
        const getCSSVar = (name: string, fallback: string): string => {
            const value = computedStyle.getPropertyValue(name).trim();
            return value || fallback;
        };

        // Create xterm instance with theme derived from CSS variables
        const term = new Terminal({
            cursorBlink: true,
            fontSize: 13,
            fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
            convertEol: true, // Handle \n as \r\n
            theme: {
                // Background colors from design system
                background: getCSSVar('--bg-tertiary', '#1e1e1e'),
                foreground: getCSSVar('--text-primary', '#d4d4d4'),
                cursor: getCSSVar('--text-primary', '#d4d4d4'),
                cursorAccent: getCSSVar('--bg-tertiary', '#1e1e1e'),
                selectionBackground: getCSSVar('--accent-cyan-dim', '#264f78'),
                // ANSI colors - using accent colors where applicable
                black: getCSSVar('--bg-tertiary', '#1e1e1e'),
                red: getCSSVar('--accent-red', '#f44747'),
                green: getCSSVar('--accent-green', '#608b4e'),
                yellow: getCSSVar('--accent-amber', '#dcdcaa'),
                blue: '#569cd6', // VS Code blue
                magenta: getCSSVar('--accent-purple', '#c586c0'),
                cyan: getCSSVar('--accent-cyan', '#4ec9b0'),
                white: getCSSVar('--text-primary', '#d4d4d4'),
                brightBlack: getCSSVar('--text-muted', '#808080'),
                brightRed: getCSSVar('--accent-red', '#f44747'),
                brightGreen: getCSSVar('--accent-green', '#608b4e'),
                brightYellow: getCSSVar('--accent-amber', '#dcdcaa'),
                brightBlue: '#569cd6', // VS Code blue
                brightMagenta: getCSSVar('--accent-purple', '#c586c0'),
                brightCyan: getCSSVar('--accent-cyan', '#4ec9b0'),
                brightWhite: '#ffffff',
            },
        });

        const fitAddon = new FitAddon();
        term.loadAddon(fitAddon);

        xtermRef.current = term;
        fitAddonRef.current = fitAddon;

        term.open(terminalRef.current);

        // Initial fit with small delay to ensure DOM is ready
        requestAnimationFrame(() => {
            fitAddon.fit();
        });

        // Enable clipboard copy support
        term.attachCustomKeyEventHandler((event) => {
            // Handle Ctrl+C / Cmd+C for copy if selection exists
            if (event.type === 'keydown' && (event.ctrlKey || event.metaKey) && event.key === 'c' && term.hasSelection()) {
                const selection = term.getSelection();
                navigator.clipboard.writeText(selection).catch(() => { });
                return false;
            }
            return true;
        });

        // Notify parent
        if (onMount) {
            onMount(term);
        }

        // T029: Use ResizeObserver for container-level resize detection (BUG-008)
        // This is more reliable than window resize for layout changes
        const resizeObserver = new ResizeObserver(() => {
            debouncedFit();
        });

        if (terminalRef.current) {
            resizeObserver.observe(terminalRef.current);
        }
        resizeObserverRef.current = resizeObserver;

        // Also handle window resize as fallback
        const handleWindowResize = () => {
            debouncedFit();
        };
        window.addEventListener('resize', handleWindowResize);

        return () => {
            // Cleanup debounce timeout
            if (fitDebounceRef.current) {
                window.clearTimeout(fitDebounceRef.current);
                fitDebounceRef.current = null;
            }
            // Cleanup ResizeObserver
            if (resizeObserverRef.current) {
                resizeObserverRef.current.disconnect();
                resizeObserverRef.current = null;
            }
            // Cleanup window listener
            window.removeEventListener('resize', handleWindowResize);
            // Dispose terminal
            term.dispose();
            xtermRef.current = null;
        };
    }, [onMount, debouncedFit]);

    // T029: Fixed - removed useEffect with no dependency array (BUG-008)
    // Initial fit is now handled in the main useEffect with requestAnimationFrame

    return (
        <div
            ref={terminalRef}
            className={`w-full h-full overflow-hidden ${className || ''}`}
            style={{ background: '#1e1e1e' }}
            data-testid="log-terminal"
        />
    );
};

