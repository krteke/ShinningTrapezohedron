export interface DeviceStatus {
	revision: number;
	collectedAtUnixMs: number | null;
	system: SystemStatus | null;
}

export interface SystemStatus {
	uptimeSecs: number;
	loadAvg: {
		oneMinute: number;
		fiveMinutes: number;
		fifteenMinutes: number;
	};
	memory: {
		totalBytes: number;
		availableBytes: number;
		usedBytes: number;
	};
}

export interface StatusStreamHandlers {
	onStatus: (status: DeviceStatus) => void;
	onOpen?: () => void;
	onError?: (message: string) => void;
}

export async function loadStatus(signal?: AbortSignal): Promise<DeviceStatus> {
	const response = await fetch('/api/status', {
		signal,
		headers: { Accept: 'application/json' }
	});
	if (!response.ok) throw new Error('无法读取设备状态');
	return (await response.json()) as DeviceStatus;
}

export function subscribeStatus(handlers: StatusStreamHandlers): () => void {
	const source = new EventSource('/api/status/events');

	source.onopen = () => handlers.onOpen?.();
	source.onerror = () => handlers.onError?.('实时连接已中断，正在自动重连');
	source.addEventListener('status', (event) => {
		try {
			handlers.onStatus(JSON.parse((event as MessageEvent<string>).data) as DeviceStatus);
		} catch {
			handlers.onError?.('收到的实时状态格式无效');
		}
	});

	return () => source.close();
}
