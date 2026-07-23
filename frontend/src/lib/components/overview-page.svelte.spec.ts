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

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('概览页面', () => {
	it('只展示状态快照中的运行指标', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue(
				new Response(JSON.stringify(snapshot), {
					headers: { 'Content-Type': 'application/json' }
				})
			)
		);

		render(OverviewPage);

		await expect.element(page.getByText('1 小时 1 分钟')).toBeInTheDocument();
		await expect.element(page.getByText('50%')).toBeInTheDocument();
		await expect.element(page.getByText('0.12')).toBeInTheDocument();
		await expect.element(page.getByText('Shinning Trapezohedron')).not.toBeInTheDocument();
		await expect.element(page.getByText(/快照/)).not.toBeInTheDocument();
	});
});
