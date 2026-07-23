<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import {
		IconAlertTriangle,
		IconCheck,
		IconDeviceFloppy,
		IconRefresh
	} from '@tabler/icons-svelte';

	import { ConfigApiError, loadConfig, replaceConfig, type DeviceConfig } from '$lib/api/config';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Spinner } from '$lib/components/ui/spinner';

	type Feedback = { kind: 'success' | 'error'; message: string };

	let {
		title,
		fieldPaths,
		validate,
		children
	}: {
		title: string;
		fieldPaths: string[];
		validate?: (config: DeviceConfig) => string | null;
		children: Snippet<[DeviceConfig, boolean, string | null]>;
	} = $props();

	let config = $state<DeviceConfig | null>(null);
	let hotAppliedFields = $state<string[]>([]);
	let restartRequiredFields = $state<string[]>([]);
	let loading = $state(true);
	let saving = $state(false);
	let feedback = $state<Feedback | null>(null);

	const validationMessage = $derived(config && validate ? validate(config) : null);
	const requiresRestart = $derived(fieldPaths.some((path) => restartRequiredFields.includes(path)));
	const appliesImmediately = $derived(
		fieldPaths.some((path) => hotAppliedFields.includes(path)) && !requiresRestart
	);

	onMount(() => {
		const controller = new AbortController();
		void refresh(controller.signal);
		return () => controller.abort();
	});

	// 避免把 Svelte 深层状态代理直接作为请求载荷或服务端快照继续使用。
	function copyConfig(value: DeviceConfig): DeviceConfig {
		return {
			web: { ...value.web },
			status: { ...value.status },
			logging: { ...value.logging }
		};
	}

	function errorMessage(error: unknown): string {
		if (error instanceof ConfigApiError) return error.message;
		if (error instanceof Error) return error.message;
		return '操作失败';
	}

	async function refresh(signal?: AbortSignal) {
		loading = true;
		feedback = null;
		try {
			const response = await loadConfig(signal);
			config = copyConfig(response.config);
			hotAppliedFields = response.hotAppliedFields;
			restartRequiredFields = response.restartRequiredFields;
		} catch (error) {
			if (error instanceof DOMException && error.name === 'AbortError') return;
			feedback = { kind: 'error', message: errorMessage(error) };
		} finally {
			if (!signal?.aborted) loading = false;
		}
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		const form = event.currentTarget as HTMLFormElement;
		if (!config || saving || validationMessage || !form.reportValidity()) return;

		saving = true;
		feedback = null;
		try {
			const response = await replaceConfig(copyConfig(config));
			config = copyConfig(response.config);
			hotAppliedFields = response.hotAppliedFields;
			restartRequiredFields = response.restartRequiredFields;
			feedback = {
				kind: 'success',
				message: requiresRestart ? '已保存，重启后生效' : '已保存并生效'
			};
		} catch (error) {
			feedback = { kind: 'error', message: errorMessage(error) };
		} finally {
			saving = false;
		}
	}
</script>

<div class="mx-auto flex max-w-2xl flex-col gap-4">
	{#if feedback}
		<Alert.Root variant={feedback.kind === 'error' ? 'destructive' : 'default'}>
			{#if feedback.kind === 'error'}
				<IconAlertTriangle />
			{:else}
				<IconCheck />
			{/if}
			<Alert.Title>{feedback.message}</Alert.Title>
			{#if feedback.kind === 'error' && !config}
				<Alert.Action>
					<Button variant="outline" size="sm" disabled={loading} onclick={() => void refresh()}>
						<IconRefresh data-icon="inline-start" />
						重试
					</Button>
				</Alert.Action>
			{/if}
		</Alert.Root>
	{/if}

	{#if loading}
		<Card.Root aria-label="正在读取配置">
			<Card.Header><Skeleton class="h-4 w-24" /></Card.Header>
			<Card.Content class="flex flex-col gap-3">
				<Skeleton class="h-3 w-20" />
				<Skeleton class="h-8 w-full" />
			</Card.Content>
			<Card.Footer><Skeleton class="h-7 w-28" /></Card.Footer>
		</Card.Root>
	{:else if config}
		<form onsubmit={save}>
			<Card.Root>
				<Card.Header>
					<Card.Title>{title}</Card.Title>
					<Card.Action>
						{#if requiresRestart}
							<Badge variant="outline">重启后生效</Badge>
						{:else if appliesImmediately}
							<Badge variant="secondary">立即生效</Badge>
						{/if}
					</Card.Action>
				</Card.Header>
				<Card.Content>
					{@render children(config, saving, validationMessage)}
				</Card.Content>
				<Card.Footer class="justify-end gap-2">
					<Button type="button" variant="outline" disabled={saving} onclick={() => void refresh()}>
						<IconRefresh data-icon="inline-start" />
						重新读取
					</Button>
					<Button type="submit" disabled={saving || validationMessage !== null}>
						{#if saving}
							<Spinner data-icon="inline-start" aria-label="正在保存" />
						{:else}
							<IconDeviceFloppy data-icon="inline-start" />
						{/if}
						保存
					</Button>
				</Card.Footer>
			</Card.Root>
		</form>
	{/if}
</div>
