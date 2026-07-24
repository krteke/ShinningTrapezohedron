import type { DeviceStatus } from '$lib/api/status';

// 这是浏览器内存保护上限，不是设备采样配置；页面重载后会重新积累。
export const MAX_HISTORY_SAMPLES = 60;

/** 图表使用的扁平采样点，避免组件反复解析完整状态快照。 */
export interface StatusSample {
	revision: number;
	collectedAtUnixMs: number;
	memoryPercent: number;
	memoryUsedMiB: number;
	memoryTotalMiB: number;
	loadOne: number;
	loadFive: number;
	loadFifteen: number;
}

export function appendStatusSample(
	history: StatusSample[],
	status: DeviceStatus,
	limit = MAX_HISTORY_SAMPLES
): StatusSample[] {
	const system = status.system;
	if (!system || status.collectedAtUnixMs === null || limit <= 0) return history;

	// 手动刷新可能与 SSE 收到同一快照，旧快照也不能倒插到趋势末尾。
	const latest = history.at(-1);
	if (latest?.revision === status.revision) return history;
	if (
		latest &&
		status.revision < latest.revision &&
		status.collectedAtUnixMs <= latest.collectedAtUnixMs
	) {
		return history;
	}

	const memoryRatio =
		system.memory.totalBytes > 0 ? system.memory.usedBytes / system.memory.totalBytes : 0;
	const bytesPerMiB = 1024 ** 2;
	const sample: StatusSample = {
		revision: status.revision,
		collectedAtUnixMs: status.collectedAtUnixMs,
		memoryPercent: Math.round(Math.min(1, Math.max(0, memoryRatio)) * 1000) / 10,
		memoryUsedMiB: system.memory.usedBytes / bytesPerMiB,
		memoryTotalMiB: system.memory.totalBytes / bytesPerMiB,
		loadOne: system.loadAvg.oneMinute,
		loadFive: system.loadAvg.fiveMinutes,
		loadFifteen: system.loadAvg.fifteenMinutes
	};

	// 守护进程重启或系统时钟回拨后重新起一段趋势，避免横轴折返。
	const shouldReset =
		latest &&
		(status.revision < latest.revision || status.collectedAtUnixMs <= latest.collectedAtUnixMs);
	return [...(shouldReset ? [] : history), sample].slice(-limit);
}
