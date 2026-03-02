/**
 * @module test/mocks/websocket
 * @description
 * Mock WebSocket class for testing hooks that use WebSocket connections.
 * Simulates open, message, error, and close events.
 *
 * @context
 * Used by useExecutionStream and useWebSocketReconnect tests to simulate
 * WebSocket behavior without requiring a real server.
 *
 * @dependencies
 * - None (standalone mock)
 */

type WebSocketEventHandler = ((event: Event) => void) | null;
type MessageHandler = ((event: MessageEvent) => void) | null;

export class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSING = 2;
  readonly CLOSED = 3;

  url: string;
  readyState: number = MockWebSocket.CONNECTING;
  protocol = '';
  extensions = '';
  bufferedAmount = 0;
  binaryType: BinaryType = 'blob';

  onopen: WebSocketEventHandler = null;
  onmessage: MessageHandler = null;
  onerror: WebSocketEventHandler = null;
  onclose: ((event: CloseEvent) => void) | null = null;

  /** Track all created instances for test assertions */
  static instances: MockWebSocket[] = [];

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  constructor(url: string, protocols?: string | string[]) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  /** Simulate the connection opening */
  simulateOpen(): void {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.(new Event('open'));
  }

  /** Simulate receiving a message */
  simulateMessage(data: string): void {
    this.onmessage?.(new MessageEvent('message', { data }));
  }

  /** Simulate an error */
  simulateError(): void {
    this.onerror?.(new Event('error'));
  }

  /** Simulate the connection closing */
  simulateClose(code = 1000, reason = ''): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close', { code, reason }));
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
    if (this.readyState !== MockWebSocket.OPEN) {
      throw new DOMException('WebSocket is not open');
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  close(code?: number, reason?: string): void {
    this.readyState = MockWebSocket.CLOSED;
  }

  addEventListener() {}
  removeEventListener() {}
  dispatchEvent(): boolean { return false; }

  /** Reset all tracked instances between tests */
  static reset(): void {
    MockWebSocket.instances = [];
  }
}
