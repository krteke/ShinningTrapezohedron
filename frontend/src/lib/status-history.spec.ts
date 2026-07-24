import { describe, expect, it } from 'vitest';

import type { DeviceStatus } from '$lib/api/status';

import { appendStatusSample } from './status-history';

const MIB = 1024 ** 2;

function createSnapshot(revision: number, usedMiB = 512): DeviceStatus {
	return {
		revision,
		collectedAtUnixMs: 1_722_470_400_000 + revision * 2_000,
		system: {
			uptimeSecs: revision * 2,
			loadAvg: {
				oneMinute: revision / 10,
				fiveMinutes: revision / 20,
				fifteenMinutes: revision / 40
			},
			memory: {
				totalBytes: 1024 * MIB,
				availableBytes: (1024 - usedMiB) * MIB,
				usedBytes: usedMiB * MIB
			}
		}
	};
}

describe('状态趋势历史', () => {
	it('把完整快照转换为图表采样点', () => {
		const history = appendStatusSample([], createSnapshot(3));

		expect(history).toEqual([
			{
				revision: 3,
				collectedAtUnixMs: 1_722_470_406_000,
				memoryPercent: 50,
				memoryUsedMiB: 512,
				memoryTotalMiB: 1024,
				loadOne: 0.3,
				loadFive: 0.15,
				loadFifteen: 0.075
			}
		]);
	});

	it('忽略重复或更旧的快照，并限制保留数量', () => {
		let history = appendStatusSample([], createSnapshot(1), 3);
		history = appendStatusSample(history, createSnapshot(2), 3);
		history = appendStatusSample(history, createSnapshot(3), 3);
		history = appendStatusSample(history, createSnapshot(4), 3);
		const unchanged = appendStatusSample(history, createSnapshot(3), 3);

		expect(history.map((sample) => sample.revision)).toEqual([2, 3, 4]);
		expect(unchanged).toBe(history);
	});

	it('守护进程重启后清空旧趋势并接受新的 revision', () => {
		const previous = appendStatusSample([], createSnapshot(20));
		const restarted = {
			...createSnapshot(1),
			collectedAtUnixMs: previous[0].collectedAtUnixMs + 2_000
		};

		expect(appendStatusSample(previous, restarted).map((sample) => sample.revision)).toEqual([1]);
	});

	it('不把尚未采集的状态加入趋势', () => {
		const snapshot = { ...createSnapshot(1), system: null };

		expect(appendStatusSample([], snapshot)).toEqual([]);
	});
});
