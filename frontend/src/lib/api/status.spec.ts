import { afterEach, describe, expect, it, vi } from 'vitest';

import { loadStatus, subscribeStatus, type DeviceStatus } from './status';

const snapshot: DeviceStatus = {
	revision: 3,
	collectedAtUnixMs: 1234,
	system: {
		uptimeSecs: 3661,
		loadAvg: { oneMinute: 0.1, fiveMinutes: 0.2, fifteenMinutes: 0.3 },
		memory: { totalBytes: 1024, availableBytes: 512, usedBytes: 512 }
	}
};

class MockEventSource {
	static current: MockEventSource;

	readonly close = vi.fn();
	onopen: (() => void) | null = null;
	onerror: (() => void) | null = null;
	private statusListener?: (event: MessageEvent<string>) => void;

	constructor(readonly url: string) {
		MockEventSource.current = this;
	}

	addEventListener(type: string, listener: EventListener) {
		if (type === 'status') {
			this.statusListener = listener as (event: MessageEvent<string>) => void;
		}
	}

	emitStatus(status: DeviceStatus) {
		this.statusListener?.({ data: JSON.stringify(status) } as MessageEvent<string>);
	}
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('状态 API 客户端', () => {
	it('读取完整状态快照', async () => {
		const fetchMock = vi.fn().mockResolvedValue(
			new Response(JSON.stringify(snapshot), {
				headers: { 'Content-Type': 'application/json' }
			})
		);
		vi.stubGlobal('fetch', fetchMock);

		await expect(loadStatus()).resolves.toEqual(snapshot);
		expect(fetchMock).toHaveBeenCalledWith('/api/status', {
			signal: undefined,
			headers: { Accept: 'application/json' }
		});
	});

	it('订阅实时快照并在销毁时关闭连接', () => {
		vi.stubGlobal('EventSource', MockEventSource);
		const onStatus = vi.fn();
		const close = subscribeStatus({ onStatus });

		expect(MockEventSource.current.url).toBe('/api/status/events');
		MockEventSource.current.emitStatus(snapshot);
		expect(onStatus).toHaveBeenCalledWith(snapshot);

		close();
		expect(MockEventSource.current.close).toHaveBeenCalledOnce();
	});

	it('连接中断时交由浏览器自动重连', () => {
		vi.stubGlobal('EventSource', MockEventSource);
		const onError = vi.fn();
		subscribeStatus({ onStatus: vi.fn(), onError });

		MockEventSource.current.onerror?.();

		expect(onError).toHaveBeenCalledWith('实时连接已中断，正在自动重连');
		expect(MockEventSource.current.close).not.toHaveBeenCalled();
	});
});
