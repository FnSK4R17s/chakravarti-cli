/**
 * @module logService.test
 * @description
 * Tests for the log service API functions. Validates correct URL construction,
 * query parameter handling, error throwing, and response parsing.
 *
 * @context
 * Uses MSW for HTTP mocking. Straight async function tests without renderHook
 * or QueryClient.
 *
 * @dependencies
 * - vitest: describe, it, expect
 * - msw: http, HttpResponse for per-test handler overrides
 * - @/test/mocks/server: MSW server instance
 * - @/services/logService: Functions under test
 */

import { describe, it, expect } from 'vitest';
import { http, HttpResponse } from 'msw';
import { server } from '@/test/mocks/server';
import { fetchLogs, fetchTailLogs, deleteLogs, fetchLogsSince } from '@/services/logService';

const EXECUTION_ID = 'exec-001';

describe('fetchLogs', () => {
    it('fetches logs from GET /api/execution/:id/logs', async () => {
        server.use(
            http.get('/api/execution/:id/logs', ({ params }) => {
                expect(params.id).toBe(EXECUTION_ID);
                return HttpResponse.json({
                    execution_id: EXECUTION_ID,
                    logs: [{ timestamp: '2026-01-15T10:00:00Z', level: 'info', message: 'test' }],
                    total: 1,
                    offset: 0,
                    limit: 100,
                });
            })
        );

        const result = await fetchLogs(EXECUTION_ID);
        expect(result.execution_id).toBe(EXECUTION_ID);
        expect(result.logs).toHaveLength(1);
    });

    it('passes offset and limit as query params', async () => {
        server.use(
            http.get('/api/execution/:id/logs', ({ request }) => {
                const url = new URL(request.url);
                expect(url.searchParams.get('offset')).toBe('10');
                expect(url.searchParams.get('limit')).toBe('50');
                return HttpResponse.json({
                    execution_id: EXECUTION_ID,
                    logs: [],
                    total: 0,
                    offset: 10,
                    limit: 50,
                });
            })
        );

        await fetchLogs(EXECUTION_ID, { offset: 10, limit: 50 });
    });

    it('passes since as query param', async () => {
        server.use(
            http.get('/api/execution/:id/logs', ({ request }) => {
                const url = new URL(request.url);
                expect(url.searchParams.get('since')).toBe('2026-01-15T10:00:00Z');
                return HttpResponse.json({
                    execution_id: EXECUTION_ID,
                    logs: [],
                    total: 0,
                    offset: 0,
                    limit: 100,
                });
            })
        );

        await fetchLogs(EXECUTION_ID, { since: '2026-01-15T10:00:00Z' });
    });

    it('throws on non-ok response', async () => {
        server.use(
            http.get('/api/execution/:id/logs', () => new HttpResponse(null, { status: 500 }))
        );

        await expect(fetchLogs(EXECUTION_ID)).rejects.toThrow('Failed to fetch logs');
    });
});

describe('fetchTailLogs', () => {
    it('defaults to count=10', async () => {
        server.use(
            http.get('/api/execution/:id/logs/tail', ({ request }) => {
                const url = new URL(request.url);
                expect(url.searchParams.get('count')).toBe('10');
                return HttpResponse.json({
                    execution_id: EXECUTION_ID,
                    logs: [],
                    count: 10,
                });
            })
        );

        await fetchTailLogs(EXECUTION_ID);
    });

    it('uses custom count', async () => {
        server.use(
            http.get('/api/execution/:id/logs/tail', ({ request }) => {
                const url = new URL(request.url);
                expect(url.searchParams.get('count')).toBe('50');
                return HttpResponse.json({
                    execution_id: EXECUTION_ID,
                    logs: [],
                    count: 50,
                });
            })
        );

        await fetchTailLogs(EXECUTION_ID, 50);
    });
});

describe('deleteLogs', () => {
    it('sends DELETE request', async () => {
        server.use(
            http.delete('/api/execution/:id/logs', ({ params }) => {
                expect(params.id).toBe(EXECUTION_ID);
                return HttpResponse.json({
                    success: true,
                    execution_id: EXECUTION_ID,
                    deleted_count: 5,
                });
            })
        );

        const result = await deleteLogs(EXECUTION_ID);
        expect(result.success).toBe(true);
    });
});

describe('fetchLogsSince', () => {
    it('delegates to fetchLogs with since option', async () => {
        server.use(
            http.get('/api/execution/:id/logs', ({ request }) => {
                const url = new URL(request.url);
                expect(url.searchParams.get('since')).toBe('2026-01-15T10:00:00Z');
                return HttpResponse.json({
                    execution_id: EXECUTION_ID,
                    logs: [],
                    total: 0,
                    offset: 0,
                    limit: 100,
                });
            })
        );

        await fetchLogsSince(EXECUTION_ID, '2026-01-15T10:00:00Z');
    });
});
