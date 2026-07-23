<script lang="ts">
	import type { DeviceConfig } from '$lib/api/config';
	import ConfigEditor from '$lib/components/config-editor.svelte';
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';

	function validate(config: DeviceConfig): string | null {
		const interval = config.status.sample_interval_seconds;
		return Number.isInteger(interval) && interval > 0 ? null : '请输入大于零的整数';
	}
</script>

<svelte:head><title>状态采样</title></svelte:head>

<ConfigEditor title="采样周期" fieldPaths={['status.sample_interval_seconds']} {validate}>
	{#snippet children(config, saving, validationMessage)}
		<Field.FieldGroup>
			<Field.Field
				data-invalid={validationMessage ? true : undefined}
				data-disabled={saving ? true : undefined}
			>
				<Field.FieldLabel for="status-sample-interval">周期（秒）</Field.FieldLabel>
				<Input
					id="status-sample-interval"
					class="max-w-40"
					type="number"
					min="1"
					step="1"
					bind:value={config.status.sample_interval_seconds}
					disabled={saving}
					aria-invalid={validationMessage !== null}
					required
				/>
				{#if validationMessage}
					<Field.FieldError>{validationMessage}</Field.FieldError>
				{/if}
			</Field.Field>
		</Field.FieldGroup>
	{/snippet}
</ConfigEditor>
