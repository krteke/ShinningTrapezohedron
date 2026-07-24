<script lang="ts">
	import { onMount } from 'svelte';
	import { IconAlertTriangle, IconRefresh } from '@tabler/icons-svelte';

	import { loadStatus, subscribeStatus, type DeviceStatus } from '$lib/api/status';
	import { appendStatusSample, type StatusSample } from '$lib/status-history';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Spinner } from '$lib/components/ui/spinner';

	import StatusCharts from './status-charts.svelte';

	let status = $state<DeviceStatus | null>(null);
	let samples = $state<StatusSample[]>([]);
	let loading = $state(true);
	let refreshing = $state(false);
	let streamError = $state<string | null>(null);
	let requestError = $state<string | null>(null);

	const error = $derived(requestError ?? streamError);

	onMount(() => {
		return subscribeStatus({
			onStatus: (snapshot) => {
				applySnapshot(snapshot);
			},
			onOpen: () => {
				streamError = null;
			},
			onError: (message) => {
				loading = false;
				streamError = message;
			}
		});
	});

	function applySnapshot(snapshot: DeviceStatus) {
		status = snapshot;
		samples = appendStatusSample(samples, snapshot);
		loading = false;
		streamError = null;
		requestError = null;
	}

	async function refresh() {
		if (status === null) loading = true;
		refreshing = true;
		requestError = null;
		try {
			applySnapshot(await loadStatus());
		} catch (cause) {
			requestError = cause instanceof Error ? cause.message : '无法读取设备状态';
		} finally {
			loading = false;
			refreshing = false;
		}
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
</script>

<svelte:head><title>概览</title></svelte:head>

<div class="mx-auto flex max-w-5xl flex-col gap-4">
	<div class="flex items-center justify-end gap-3">
		{#if status?.system}
			<Badge variant="outline">已运行 {formatUptime(status.system.uptimeSecs)}</Badge>
		{/if}
		{#if status}
			<span class="text-xs text-muted-foreground">
				更新于 {formatCollectedAt(status.collectedAtUnixMs)}
			</span>
		{/if}
		<Button variant="outline" size="sm" disabled={refreshing} onclick={() => void refresh()}>
			{#if refreshing}
				<Spinner data-icon="inline-start" aria-label="正在刷新" />
			{:else}
				<IconRefresh data-icon="inline-start" />
			{/if}
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
		<div class="grid gap-4 md:grid-cols-2" aria-label="正在读取设备状态">
			{#each [0, 1] as item (item)}
				<Card.Root>
					<Card.Header class="flex flex-col gap-2">
						<Skeleton class="h-5 w-24" />
						<Skeleton class="h-3 w-40" />
					</Card.Header>
					<Card.Content>
						<Skeleton class="h-64 w-full" />
					</Card.Content>
				</Card.Root>
			{/each}
		</div>
	{:else if status?.system && samples.length > 0}
		<StatusCharts {samples} />
	{:else if !error}
		<Alert.Root>
			<Alert.Title>状态尚未采集</Alert.Title>
		</Alert.Root>
	{/if}
</div>
