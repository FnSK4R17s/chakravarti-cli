/**
 * useWebSocketReconnect Hook (T011)
 * 
 * Custom hook for managing WebSocket connections with automatic reconnection.
 * Implements exponential backoff with visible countdown.
 * 
 * Addresses BUG-002: WebSocket Connection Not Reconnected on Network Failure
 */

import { useRef, useState, useCallback, useEffect } from 'react';

export interface WebSocketState {
    status: 'disconnected' | 'connecting' | 'connected' | 'reconnecting';
    retryCount: number;
    retryCountdown: number;
    lastError?: string;
}

export interface UseWebSocketReconnectOptions {
    /** Maximum number of retry attempts (default: 3) */
    maxRetries?: number;
    /** Initial delay in ms before first retry (default: 5000) */
    initialDelay?: number;
    /** Multiplier for exponential backoff (default: 2) */
    backoffMultiplier?: number;
    /** Callback when message received */
    onMessage?: (event: MessageEvent) => void;
    /** Callback when connection opens */
    onOpen?: () => void;
    /** Callback when connection closes (all retries exhausted) */
    onClose?: () => void;
    /** Callback when error occurs */
    onError?: (error: Event) => void;
}

export interface UseWebSocketReconnectReturn {
    /** Current WebSocket state */
    state: WebSocketState;
    /** Connect to WebSocket URL */
    connect: (url: string) => void;
    /** Disconnect and cleanup */
    disconnect: () => void;
    /** Manually trigger reconnection */
    reconnect: () => void;
    /** Send message through WebSocket */
    send: (data: string | ArrayBufferLike | Blob | ArrayBufferView) => void;
}

export function useWebSocketReconnect(
    options: UseWebSocketReconnectOptions = {}
): UseWebSocketReconnectReturn {
    const {
        maxRetries = 3,
        initialDelay = 5000,
        backoffMultiplier = 2,
        onMessage,
        onOpen,
        onClose,
        onError,
    } = options;

    const wsRef = useRef<WebSocket | null>(null);
    const urlRef = useRef<string>('');
    const retryTimeoutRef = useRef<number | null>(null);
    const countdownIntervalRef = useRef<number | null>(null);

    const [state, setState] = useState<WebSocketState>({
        status: 'disconnected',
        retryCount: 0,
        retryCountdown: 0,
    });

    // Cleanup function
    const cleanup = useCallback(() => {
        if (retryTimeoutRef.current) {
            clearTimeout(retryTimeoutRef.current);
            retryTimeoutRef.current = null;
        }
        if (countdownIntervalRef.current) {
            clearInterval(countdownIntervalRef.current);
            countdownIntervalRef.current = null;
        }
        if (wsRef.current) {
            wsRef.current.onopen = null;
            wsRef.current.onmessage = null;
            wsRef.current.onerror = null;
            wsRef.current.onclose = null;
            wsRef.current.close();
            wsRef.current = null;
        }
    }, []);

    // Create WebSocket connection
    const createConnection = useCallback((url: string, retryCount: number = 0) => {
        cleanup();
        urlRef.current = url;

        setState(prev => ({
            ...prev,
            status: retryCount > 0 ? 'reconnecting' : 'connecting',
            retryCount,
        }));

        const ws = new WebSocket(url);
        wsRef.current = ws;

        ws.onopen = () => {
            setState({
                status: 'connected',
                retryCount: 0,
                retryCountdown: 0,
            });
            onOpen?.();
        };

        ws.onmessage = (event) => {
            onMessage?.(event);
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            setState(prev => ({
                ...prev,
                lastError: 'WebSocket connection error',
            }));
            onError?.(error);
        };

        ws.onclose = () => {
            if (retryCount < maxRetries) {
                // Calculate delay with exponential backoff
                const delay = initialDelay * Math.pow(backoffMultiplier, retryCount);
                const delaySeconds = Math.ceil(delay / 1000);

                setState(prev => ({
                    ...prev,
                    status: 'reconnecting',
                    retryCount: retryCount + 1,
                    retryCountdown: delaySeconds,
                }));

                // Start countdown
                let countdown = delaySeconds;
                countdownIntervalRef.current = window.setInterval(() => {
                    countdown -= 1;
                    setState(prev => ({
                        ...prev,
                        retryCountdown: countdown,
                    }));
                    if (countdown <= 0) {
                        clearInterval(countdownIntervalRef.current!);
                        countdownIntervalRef.current = null;
                    }
                }, 1000);

                // Schedule retry
                retryTimeoutRef.current = window.setTimeout(() => {
                    createConnection(url, retryCount + 1);
                }, delay);
            } else {
                // All retries exhausted
                setState({
                    status: 'disconnected',
                    retryCount: 0,
                    retryCountdown: 0,
                    lastError: 'Connection failed after maximum retry attempts',
                });
                onClose?.();
            }
        };
    }, [cleanup, maxRetries, initialDelay, backoffMultiplier, onMessage, onOpen, onClose, onError]);

    // Public connect function
    const connect = useCallback((url: string) => {
        createConnection(url, 0);
    }, [createConnection]);

    // Public disconnect function
    const disconnect = useCallback(() => {
        cleanup();
        setState({
            status: 'disconnected',
            retryCount: 0,
            retryCountdown: 0,
        });
    }, [cleanup]);

    // Manual reconnect
    const reconnect = useCallback(() => {
        if (urlRef.current) {
            createConnection(urlRef.current, 0);
        }
    }, [createConnection]);

    // Send message
    const send = useCallback((data: string | ArrayBufferLike | Blob | ArrayBufferView) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(data);
        } else {
            console.warn('WebSocket not connected, cannot send message');
        }
    }, []);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            cleanup();
        };
    }, [cleanup]);

    return {
        state,
        connect,
        disconnect,
        reconnect,
        send,
    };
}

export default useWebSocketReconnect;
