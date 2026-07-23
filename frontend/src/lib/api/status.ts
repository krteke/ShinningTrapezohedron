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

export async function loadStatus(signal?: AbortSignal): Promise<DeviceStatus> {
	const response = await fetch('/api/status', {
		signal,
		headers: { Accept: 'application/json' }
	});
	if (!response.ok) throw new Error('无法读取设备状态');
	return (await response.json()) as DeviceStatus;
}
