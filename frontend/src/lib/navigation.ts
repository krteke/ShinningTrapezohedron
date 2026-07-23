import { IconActivity, IconBrowser, IconFileText, IconGauge } from '@tabler/icons-svelte';

export type AppPath = '/' | '/settings/web' | '/settings/status' | '/settings/logging';

export interface NavigationItem {
	href: AppPath;
	label: string;
	icon: typeof IconGauge;
}

export interface NavigationGroup {
	label: string;
	items: NavigationItem[];
}

export const navigationGroups: NavigationGroup[] = [
	{
		label: '状态',
		items: [{ href: '/', label: '概览', icon: IconGauge }]
	},
	{
		label: '设置',
		items: [
			{ href: '/settings/web', label: 'Web 服务', icon: IconBrowser },
			{ href: '/settings/status', label: '状态采样', icon: IconActivity },
			{ href: '/settings/logging', label: '日志', icon: IconFileText }
		]
	}
];

export function pageTitle(pathname: string): string {
	return (
		navigationGroups.flatMap((group) => group.items).find((item) => item.href === pathname)
			?.label ?? '控制台'
	);
}
