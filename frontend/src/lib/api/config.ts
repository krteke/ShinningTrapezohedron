export interface DeviceConfig {
  web: {
    listen_address: string;
  };
  status: {
    sample_interval_seconds: number;
  };
  logging: {
    filter: string;
    ansi: boolean;
  };
}

export interface ConfigEnvelope {
  config: DeviceConfig;
  /** 后端明确返回字段的生效方式 */
  hotAppliedFields: string[];
  restartRequiredFields: string[];
}

interface ApiErrorBody {
  code?: string;
  message?: string;
}

export class ConfigApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string
  ) {
    super(message);
    this.name = 'ConfigApiError';
  }
}

async function requestConfig(init?: RequestInit): Promise<ConfigEnvelope> {
  const response = await fetch('/api/config', {
    ...init,
    headers: {
      Accept: 'application/json',
      ...(init?.body ? { 'Content-Type': 'application/json' } : {})
    }
  });

  let body: unknown;
  try {
    body = await response.json();
  } catch {
    throw new ConfigApiError(response.status, 'invalid_response', '服务返回了无法解析的响应');
  }

  if (!response.ok) {
    const error = body as ApiErrorBody;
    throw new ConfigApiError(
      response.status,
      error.code ?? 'request_failed',
      error.message ?? '配置请求失败'
    );
  }

  return body as ConfigEnvelope;
}

export function loadConfig(signal?: AbortSignal): Promise<ConfigEnvelope> {
  return requestConfig({ signal });
}

export function replaceConfig(config: DeviceConfig): Promise<ConfigEnvelope> {
  return requestConfig({
    method: 'PUT',
    body: JSON.stringify(config)
  });
}
