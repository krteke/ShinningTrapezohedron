import { afterEach, describe, expect, it, vi } from 'vitest';

import { loadStatus, type DeviceStatus } from './status';

const snapshot: DeviceStatus = {
	revision: 3,
	collectedAtUnixMs: 1234,
	system: {
		uptimeSecs: 3661,
		loadAvg: { oneMinute: 0.1, fiveMinutes: 0.2, fifteenMinutes: 0.3 },
		memory: { totalBytes: 1024, availableBytes: 512, usedBytes: 512 }
	}
};

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
});
