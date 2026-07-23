<script lang="ts">
	import { onMount } from 'svelte';
	import {
		IconAlertTriangle,
		IconClock,
		IconCpu,
		IconRefresh,
		IconServer
	} from '@tabler/icons-svelte';

	import { loadStatus, type DeviceStatus } from '$lib/api/status';
	import * as Alert from '$lib/components/ui/alert';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Skeleton } from '$lib/components/ui/skeleton';

	let status = $state<DeviceStatus | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const memoryPercent = $derived.by(() => {
		const memory = status?.system?.memory;
		if (!memory || memory.totalBytes === 0) return 0;
		return Math.round((memory.usedBytes / memory.totalBytes) * 100);
	});

	onMount(() => {
		const controller = new AbortController();
		void refresh(controller.signal);
		return () => controller.abort();
	});

	async function refresh(signal?: AbortSignal) {
		loading = true;
		error = null;
		try {
			status = await loadStatus(signal);
		} catch (cause) {
			if (cause instanceof DOMException && cause.name === 'AbortError') return;
			error = cause instanceof Error ? cause.message : '无法读取设备状态';
		} finally {
			if (!signal?.aborted) loading = false;
		}
	}

	function formatBytes(bytes: number): string {
		const gibibyte = 1024 ** 3;
		const mebibyte = 1024 ** 2;
		if (bytes >= gibibyte) return `${(bytes / gibibyte).toFixed(1)} GiB`;
		return `${Math.round(bytes / mebibyte)} MiB`;
	}

	function formatUptime(seconds: number): string {
		const days = Math.floor(seconds / 86_400);
		const hours = Math.floor((seconds % 86_400) / 3_600);
		const minutes = Math.floor((seconds % 3_600) / 60);
		if (days > 0) return `${days} 天 ${hours} 小时`;
		if (hours > 0) return `${hours} 小时 ${minutes} 分钟`;
		return `${minutes} 分钟`;
	}

	function formatCollectedAt(timestamp: number | null): string {
		if (timestamp === null) return '尚未采集';
		return new Intl.DateTimeFormat('zh-CN', {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		}).format(timestamp);
	}

	function formatBootAt(collectedAt: number | null, uptimeSeconds: number): string | null {
		if (collectedAt === null) return null;
		return new Intl.DateTimeFormat('zh-CN', {
			month: 'numeric',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		}).format(collectedAt - uptimeSeconds * 1000);
	}
</script>

<svelte:head><title>概览</title></svelte:head>

<div class="mx-auto flex max-w-5xl flex-col gap-4">
	<div class="flex items-center justify-end gap-3">
		{#if status}
			<span class="text-xs text-muted-foreground">
				更新于 {formatCollectedAt(status.collectedAtUnixMs)}
			</span>
		{/if}
		<Button variant="outline" size="sm" disabled={loading} onclick={() => void refresh()}>
			<IconRefresh data-icon="inline-start" />
			刷新
		</Button>
	</div>

	{#if error}
		<Alert.Root variant="destructive">
			<IconAlertTriangle />
			<Alert.Title>{error}</Alert.Title>
		</Alert.Root>
	{/if}

	{#if loading}
		<div class="grid gap-4 md:grid-cols-3" aria-label="正在读取设备状态">
			{#each [0, 1, 2] as item (item)}
				<Card.Root>
					<Card.Header><Skeleton class="h-4 w-20" /></Card.Header>
					<Card.Content class="flex flex-col gap-2">
						<Skeleton class="h-8 w-32" />
						<Skeleton class="h-3 w-24" />
					</Card.Content>
				</Card.Root>
			{/each}
		</div>
	{:else if status?.system}
		<div class="grid gap-4 md:grid-cols-3">
			<Card.Root>
				<Card.Header>
					<Card.Title>运行时间</Card.Title>
					<Card.Action class="text-muted-foreground"><IconClock class="size-4" /></Card.Action>
				</Card.Header>
				<Card.Content class="flex flex-col gap-1.5">
					<p class="text-2xl font-medium tabular-nums">
						{formatUptime(status.system.uptimeSecs)}
					</p>
					{@const bootAt = formatBootAt(status.collectedAtUnixMs, status.system.uptimeSecs)}
					{#if bootAt}
						<p class="text-xs text-muted-foreground">启动于 {bootAt}</p>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title>内存</Card.Title>
					<Card.Action class="text-muted-foreground"><IconServer class="size-4" /></Card.Action>
				</Card.Header>
				<Card.Content class="flex flex-col gap-1.5">
					<p class="text-2xl font-medium tabular-nums">{memoryPercent}%</p>
					<p class="text-xs text-muted-foreground">
						{formatBytes(status.system.memory.usedBytes)} / {formatBytes(
							status.system.memory.totalBytes
						)}
					</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title>系统负载</Card.Title>
					<Card.Action class="text-muted-foreground"><IconCpu class="size-4" /></Card.Action>
				</Card.Header>
				<Card.Content>
					<div class="grid grid-cols-3 gap-3">
						<div>
							<p class="text-xl font-medium tabular-nums">
								{status.system.loadAvg.oneMinute.toFixed(2)}
							</p>
							<p class="text-xs text-muted-foreground">1 分钟</p>
						</div>
						<div>
							<p class="text-xl font-medium tabular-nums">
								{status.system.loadAvg.fiveMinutes.toFixed(2)}
							</p>
							<p class="text-xs text-muted-foreground">5 分钟</p>
						</div>
						<div>
							<p class="text-xl font-medium tabular-nums">
								{status.system.loadAvg.fifteenMinutes.toFixed(2)}
							</p>
							<p class="text-xs text-muted-foreground">15 分钟</p>
						</div>
					</div>
				</Card.Content>
			</Card.Root>
		</div>
	{:else}
		<Alert.Root>
			<Alert.Title>状态尚未采集</Alert.Title>
		</Alert.Root>
	{/if}
</div>
