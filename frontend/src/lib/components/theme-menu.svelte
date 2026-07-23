<script lang="ts">
	import { IconDeviceDesktop, IconMoon, IconSun } from '@tabler/icons-svelte';
	import { setMode, userPrefersMode } from 'mode-watcher';

	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';

	type ThemeMode = 'system' | 'light' | 'dark';

	const selectedMode = $derived(userPrefersMode.current);
	const CurrentIcon = $derived.by(() => {
		if (selectedMode === 'light') return IconSun;
		if (selectedMode === 'dark') return IconMoon;
		return IconDeviceDesktop;
	});

	function selectMode(value: string) {
		if (value === 'system' || value === 'light' || value === 'dark') {
			setMode(value satisfies ThemeMode);
		}
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon-sm" aria-label="切换主题">
				<CurrentIcon />
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end" class="w-36">
		<DropdownMenu.Group>
			<DropdownMenu.Label>主题</DropdownMenu.Label>
			<DropdownMenu.RadioGroup value={selectedMode} onValueChange={selectMode}>
				<DropdownMenu.RadioItem value="system">
					<IconDeviceDesktop />
					System
				</DropdownMenu.RadioItem>
				<DropdownMenu.RadioItem value="light">
					<IconSun />
					Light
				</DropdownMenu.RadioItem>
				<DropdownMenu.RadioItem value="dark">
					<IconMoon />
					Dark
				</DropdownMenu.RadioItem>
			</DropdownMenu.RadioGroup>
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
