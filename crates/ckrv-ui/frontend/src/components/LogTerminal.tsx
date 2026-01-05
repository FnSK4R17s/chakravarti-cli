import React, { useEffect, useRef } from 'react';
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

    useEffect(() => {
        if (!terminalRef.current || xtermRef.current) return;

        // Create xterm instance with VS Code-like theme
        const term = new Terminal({
            cursorBlink: true,
            fontSize: 13,
            fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
            convertEol: true, // Handle \n as \r\n
            theme: {
                background: '#1e1e1e',
                foreground: '#d4d4d4',
                cursor: '#d4d4d4',
                cursorAccent: '#1e1e1e',
                selectionBackground: '#264f78',
                black: '#1e1e1e',
                red: '#f44747',
                green: '#608b4e',
                yellow: '#dcdcaa',
                blue: '#569cd6',
                magenta: '#c586c0',
                cyan: '#4ec9b0',
                white: '#d4d4d4',
                brightBlack: '#808080',
                brightRed: '#f44747',
                brightGreen: '#608b4e',
                brightYellow: '#dcdcaa',
                brightBlue: '#569cd6',
                brightMagenta: '#c586c0',
                brightCyan: '#4ec9b0',
                brightWhite: '#ffffff',
            },
        });

        const fitAddon = new FitAddon();
        term.loadAddon(fitAddon);

        xtermRef.current = term;
        fitAddonRef.current = fitAddon;

        term.open(terminalRef.current);
        fitAddon.fit();

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

        // Handle resize
        const handleResize = () => {
            fitAddon.fit();
        };
        window.addEventListener('resize', handleResize);

        return () => {
            window.removeEventListener('resize', handleResize);
            term.dispose();
            xtermRef.current = null;
        };
    }, [onMount]);

    // Handle initial fit after layout
    useEffect(() => {
        const timer = setTimeout(() => {
            fitAddonRef.current?.fit();
        }, 100);
        return () => clearTimeout(timer);
    });

    return (
        <div
            ref={terminalRef}
            className={`w-full h-full overflow-hidden ${className || ''}`}
            style={{ background: '#1e1e1e' }}
        />
    );
};
