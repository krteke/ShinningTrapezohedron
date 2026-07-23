import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  ConfigApiError,
  loadConfig,
  replaceConfig,
  type ConfigEnvelope,
  type DeviceConfig
} from './config';

const config: DeviceConfig = {
  web: { listen_address: '127.0.0.1:3000' },
  status: { sample_interval_seconds: 2 },
  logging: { filter: 'info', ansi: false }
};

const envelope: ConfigEnvelope = {
  config,
  hotAppliedFields: ['status.sample_interval_seconds'],
  restartRequiredFields: ['web.listen_address', 'logging.filter', 'logging.ansi']
};

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' }
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('配置 API 客户端', () => {
  it('读取完整配置快照', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(envelope));
    vi.stubGlobal('fetch', fetchMock);

    await expect(loadConfig()).resolves.toEqual(envelope);
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/config',
      expect.objectContaining({ headers: { Accept: 'application/json' } })
    );
  });

  it('提交完整候选配置', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(envelope));
    vi.stubGlobal('fetch', fetchMock);

    await replaceConfig(config);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/config',
      expect.objectContaining({
        method: 'PUT',
        body: JSON.stringify(config),
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json'
        }
      })
    );
  });

  it('保留后端错误码和消息', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(jsonResponse({ code: 'invalid_config', message: '候选配置无效' }, 422))
    );

    await expect(replaceConfig(config)).rejects.toEqual(
      new ConfigApiError(422, 'invalid_config', '候选配置无效')
    );
  });
});
