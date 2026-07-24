<script lang="ts">
	import { AreaChart, LineChart } from 'layerchart';

	import type { StatusSample } from '$lib/status-history';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Chart from '$lib/components/ui/chart';

	let { samples }: { samples: StatusSample[] } = $props();

	const latest = $derived(samples.at(-1));
	const memoryCapacityMiB = $derived(
		Math.max(1, ...samples.map((sample) => sample.memoryTotalMiB))
	);
	const timeFormatter = new Intl.DateTimeFormat('zh-CN', {
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit'
	});
	const percentFormatter = new Intl.NumberFormat('zh-CN', { maximumFractionDigits: 1 });
	const memoryFormatter = new Intl.NumberFormat('zh-CN', { maximumFractionDigits: 2 });

	const memoryConfig = {
		memoryUsedMiB: {
			label: '已用内存',
			theme: { light: 'var(--chart-3)', dark: 'var(--chart-1)' }
		}
	} satisfies Chart.ChartConfig;
	const loadConfig = {
		loadOne: {
			label: '1 分钟',
			theme: { light: 'var(--chart-2)', dark: 'var(--chart-1)' }
		},
		loadFive: {
			label: '5 分钟',
			theme: { light: 'var(--chart-4)', dark: 'var(--chart-2)' }
		},
		loadFifteen: {
			label: '15 分钟',
			theme: { light: 'var(--chart-5)', dark: 'var(--chart-3)' }
		}
	} satisfies Chart.ChartConfig;
	const memorySeries = [
		{
			key: 'memoryUsedMiB',
			label: memoryConfig.memoryUsedMiB.label,
			color: 'var(--color-memoryUsedMiB)'
		}
	];
	const loadSeries = [
		{ key: 'loadOne', label: loadConfig.loadOne.label, color: 'var(--color-loadOne)' },
		{ key: 'loadFive', label: loadConfig.loadFive.label, color: 'var(--color-loadFive)' },
		{
			key: 'loadFifteen',
			label: loadConfig.loadFifteen.label,
			color: 'var(--color-loadFifteen)'
		}
	];

	function formatTime(value: unknown): string {
		return timeFormatter.format(Number(value));
	}

	function formatPercent(value: number): string {
		return `${percentFormatter.format(value)}%`;
	}

	function formatMiB(value: number): string {
		return `${memoryFormatter.format(value)} MiB`;
	}
</script>

<div class="grid gap-4 xl:grid-cols-5">
	<Card.Root class="xl:col-span-3">
		<Card.Header>
			<Card.Title>内存占用</Card.Title>
			<Card.Description>实际已用容量趋势</Card.Description>
			<Card.Action>
				<Badge variant="secondary">
					{latest
						? `${formatMiB(latest.memoryUsedMiB)} / ${formatMiB(latest.memoryTotalMiB)} · ${formatPercent(latest.memoryPercent)}`
						: '—'}
				</Badge>
			</Card.Action>
		</Card.Header>
		<Card.Content>
			<Chart.Container
				config={memoryConfig}
				class="h-64 w-full"
				role="img"
				aria-label={`内存占用趋势，共 ${samples.length} 个采样点，当前 ${latest ? `${formatMiB(latest.memoryUsedMiB)}，占 ${formatPercent(latest.memoryPercent)}` : '未知'}`}
			>
				<AreaChart
					data={samples}
					x="collectedAtUnixMs"
					series={memorySeries}
					yDomain={[0, memoryCapacityMiB]}
					padding={{ top: 8, right: 12, bottom: 28, left: 104 }}
					points={samples.length < 3}
					props={{
						area: { fillOpacity: 0.2, line: { strokeWidth: 2 } },
						xAxis: { format: formatTime, tickSpacing: 80 },
						yAxis: { format: (value) => formatMiB(Number(value)), ticks: 5 }
					}}
				>
					{#snippet tooltip()}
						<Chart.Tooltip labelFormatter={formatTime}>
							{#snippet formatter({ value, name })}
								<div class="flex flex-1 items-center justify-between gap-4">
									<span class="text-muted-foreground">{name}</span>
									<span class="font-mono font-medium text-foreground tabular-nums">
										{formatMiB(Number(value))}
									</span>
								</div>
							{/snippet}
						</Chart.Tooltip>
					{/snippet}
				</AreaChart>
			</Chart.Container>
		</Card.Content>
	</Card.Root>

	<Card.Root class="xl:col-span-2">
		<Card.Header>
			<Card.Title>系统负载</Card.Title>
			<Card.Description>系统平均负载</Card.Description>
			<Card.Action>
				<Badge variant="secondary">
					1 分钟 {latest ? latest.loadOne.toFixed(2) : '—'}
				</Badge>
			</Card.Action>
		</Card.Header>
		<Card.Content>
			<Chart.Container
				config={loadConfig}
				class="h-64 w-full"
				role="img"
				aria-label={`系统负载趋势，共 ${samples.length} 个采样点，当前一分钟负载 ${latest ? latest.loadOne.toFixed(2) : '未知'}`}
			>
				<LineChart
					data={samples}
					x="collectedAtUnixMs"
					series={loadSeries}
					yDomain={[0, null]}
					yNice
					points={samples.length < 3}
					legend
					props={{
						spline: { strokeWidth: 2 },
						xAxis: { format: formatTime, tickSpacing: 80 },
						yAxis: { format: (value) => Number(value).toFixed(1), ticks: 5 }
					}}
				>
					{#snippet tooltip()}
						<Chart.Tooltip labelFormatter={formatTime} />
					{/snippet}
				</LineChart>
			</Chart.Container>
		</Card.Content>
	</Card.Root>
</div>
