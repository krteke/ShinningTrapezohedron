<script lang="ts">
	import ConfigEditor from '$lib/components/config-editor.svelte';
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { Switch } from '$lib/components/ui/switch';
</script>

<svelte:head><title>日志</title></svelte:head>

<ConfigEditor title="日志输出" fieldPaths={['logging.filter', 'logging.ansi']}>
	{#snippet children(config, saving)}
		<Field.FieldGroup>
			<Field.Field data-disabled={saving ? true : undefined}>
				<Field.FieldLabel for="logging-filter">过滤规则</Field.FieldLabel>
				<Input
					id="logging-filter"
					bind:value={config.logging.filter}
					disabled={saving}
					placeholder="info,tower_http=warn"
					autocomplete="off"
					spellcheck={false}
				/>
				<Field.FieldDescription>使用 tracing EnvFilter 语法。</Field.FieldDescription>
			</Field.Field>

			<Field.Field orientation="horizontal" data-disabled={saving ? true : undefined}>
				<Field.FieldContent>
					<Field.FieldLabel for="logging-ansi">ANSI 控制字符</Field.FieldLabel>
					<Field.FieldDescription>写入 journal 时建议关闭。</Field.FieldDescription>
				</Field.FieldContent>
				<Switch
					id="logging-ansi"
					bind:checked={config.logging.ansi}
					disabled={saving}
					aria-label="ANSI 控制字符"
				/>
			</Field.Field>
		</Field.FieldGroup>
	{/snippet}
</ConfigEditor>
