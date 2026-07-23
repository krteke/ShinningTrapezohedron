<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

	import { navigationGroups } from '$lib/navigation';
	import * as Sidebar from '$lib/components/ui/sidebar';

	const sidebar = Sidebar.useSidebar();

	function closeMobileNavigation() {
		if (sidebar.isMobile) sidebar.setOpenMobile(false);
	}
</script>

<Sidebar.Root collapsible="icon" aria-label="主导航">
	<Sidebar.Content class="pt-2">
		{#each navigationGroups as group (group.label)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as item (item.href)}
							{@const active = page.url.pathname === item.href}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton isActive={active} tooltipContent={item.label}>
									{#snippet child({ props })}
										<a
											{...props}
											href={resolve(item.href)}
											aria-current={active ? 'page' : undefined}
											onclick={closeMobileNavigation}
										>
											<item.icon />
											<span>{item.label}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>
	<Sidebar.Rail />
</Sidebar.Root>
