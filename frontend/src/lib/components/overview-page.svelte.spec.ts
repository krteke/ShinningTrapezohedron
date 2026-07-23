import { page } from 'vitest/browser';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';

import type { DeviceStatus } from '$lib/api/status';

import OverviewPage from './overview-page.svelte';

const snapshot: DeviceStatus = {
	revision: 8,
	collectedAtUnixMs: 1_722_470_400_000,
	system: {
		uptimeSecs: 3661,
		loadAvg: { oneMinute: 0.12, fiveMinutes: 0.34, fifteenMinutes: 0.56 },
		memory: {
			totalBytes: 1024 ** 3,
			availableBytes: 512 * 1024 ** 2,
			usedBytes: 512 * 1024 ** 2
		}
	}
};

class MockEventSource {
	static current: MockEventSource;

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

	close() {}
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('概览页面', () => {
	it('只展示状态快照中的运行指标', async () => {
		vi.stubGlobal('EventSource', MockEventSource);

		render(OverviewPage);
		MockEventSource.current.emitStatus(snapshot);

		await expect.element(page.getByText('1 小时 1 分钟')).toBeInTheDocument();
		await expect.element(page.getByText('50%')).toBeInTheDocument();
		await expect.element(page.getByText('0.12')).toBeInTheDocument();
		await expect.element(page.getByText('Shinning Trapezohedron')).not.toBeInTheDocument();
		await expect.element(page.getByText(/快照/)).not.toBeInTheDocument();
	});

	it('连接中断时提示自动重连，恢复后清除提示', async () => {
		vi.stubGlobal('EventSource', MockEventSource);
		render(OverviewPage);

		MockEventSource.current.onerror?.();
		await expect.element(page.getByText('实时连接已中断，正在自动重连')).toBeInTheDocument();

		MockEventSource.current.onopen?.();
		await expect.element(page.getByText('实时连接已中断，正在自动重连')).not.toBeInTheDocument();
	});

	it('新实时快照会清除手动刷新错误', async () => {
		vi.stubGlobal('EventSource', MockEventSource);
		vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(null, { status: 503 })));
		render(OverviewPage);
		MockEventSource.current.emitStatus(snapshot);

		await page.getByRole('button', { name: '刷新' }).click();
		await expect.element(page.getByText('无法读取设备状态')).toBeInTheDocument();

		MockEventSource.current.emitStatus({ ...snapshot, revision: 9 });
		await expect.element(page.getByText('无法读取设备状态')).not.toBeInTheDocument();
	});
});
