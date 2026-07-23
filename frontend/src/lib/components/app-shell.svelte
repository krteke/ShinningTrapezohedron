<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';

	import { pageTitle } from '$lib/navigation';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import ThemeMenu from '$lib/components/theme-menu.svelte';
	import { Separator } from '$lib/components/ui/separator';
	import * as Sidebar from '$lib/components/ui/sidebar';

	let { children }: { children: Snippet } = $props();
	const title = $derived(pageTitle(page.url.pathname));
</script>

<Sidebar.Provider style="--sidebar-width: 13.5rem;">
	<AppSidebar />
	<Sidebar.Inset class="min-w-0 bg-muted/20">
		<header class="sticky top-0 flex h-12 shrink-0 items-center gap-2 border-b bg-background px-3">
			<Sidebar.Trigger />
			<Separator orientation="vertical" class="h-4" />
			<h1 class="truncate text-sm font-medium">{title}</h1>
			<div class="ml-auto">
				<ThemeMenu />
			</div>
		</header>
		<div class="flex-1 p-4 md:p-6">
			{@render children()}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
