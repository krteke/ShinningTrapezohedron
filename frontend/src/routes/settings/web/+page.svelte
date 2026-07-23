<script lang="ts">
	import ConfigEditor from '$lib/components/config-editor.svelte';
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
</script>

<svelte:head><title>Web 服务</title></svelte:head>

<ConfigEditor title="监听" fieldPaths={['web.listen_address']}>
	{#snippet children(config, saving)}
		<Field.FieldGroup>
			<Field.Field data-disabled={saving ? true : undefined}>
				<Field.FieldLabel for="web-listen-address">监听地址</Field.FieldLabel>
				<Input
					id="web-listen-address"
					bind:value={config.web.listen_address}
					disabled={saving}
					placeholder="0.0.0.0:3000"
					autocomplete="off"
					spellcheck={false}
					required
				/>
				<Field.FieldDescription>使用 IP:端口 格式。</Field.FieldDescription>
			</Field.Field>
		</Field.FieldGroup>
	{/snippet}
</ConfigEditor>
