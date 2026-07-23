import { page } from 'vitest/browser';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';

import type { ConfigEnvelope, DeviceConfig } from '$lib/api/config';

import StatusSettingsPage from '../../routes/settings/status/+page.svelte';

const config: DeviceConfig = {
	web: { listen_address: '0.0.0.0:3000' },
	status: { sample_interval_seconds: 2 },
	logging: { filter: 'info', ansi: false }
};

const envelope: ConfigEnvelope = {
	config,
	hotAppliedFields: ['status.sample_interval_seconds'],
	restartRequiredFields: ['web.listen_address', 'logging.filter', 'logging.ansi']
};

function jsonResponse(body: unknown): Response {
	return new Response(JSON.stringify(body), {
		headers: { 'Content-Type': 'application/json' }
	});
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe('状态采样设置页', () => {
	it('只显示本页控制项并提交完整配置快照', async () => {
		const fetchMock = vi
			.fn()
			.mockResolvedValueOnce(jsonResponse(envelope))
			.mockImplementationOnce(async (_input: RequestInfo | URL, init?: RequestInit) => {
				const candidate = JSON.parse(String(init?.body)) as DeviceConfig;
				return jsonResponse({ ...envelope, config: candidate });
			});
		vi.stubGlobal('fetch', fetchMock);

		render(StatusSettingsPage);

		const interval = page.getByLabelText('周期（秒）');
		await expect.element(interval).toHaveValue(2);
		await expect.element(page.getByLabelText('监听地址')).not.toBeInTheDocument();
		await expect.element(page.getByLabelText('过滤规则')).not.toBeInTheDocument();

		await interval.fill('5');
		await page.getByRole('button', { name: '保存' }).click();

		await expect.element(page.getByText('已保存并生效')).toBeInTheDocument();
		expect(JSON.parse(String(fetchMock.mock.calls[1]?.[1]?.body))).toMatchObject({
			status: { sample_interval_seconds: 5 }
		});
	});
});
