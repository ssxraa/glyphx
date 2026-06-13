<script lang="ts" module>
	import type { Snippet } from 'svelte';

	export interface SettingsFieldProps {
		/** Field label. */
		label: string;
		/** Optional helper text, shown under the label in both layouts. */
		description?: string;
		/** `row` = label left / control right (forms, wide settings page).
		 *  `stack` = control below the label (full-width controls, narrow panels). */
		layout?: 'row' | 'stack';
		/** `md` = settings page; `sm` = compact (editor sidebar panel). */
		size?: 'sm' | 'md';
		/** `for` of the control, so clicking the label focuses it. */
		for?: string;
		class?: string;
		/** The control (Segmented, Switch, stepper, …). */
		children?: Snippet;
	}
</script>

<script lang="ts">
	import { cn } from '@glyph/ui/utils';
	import { Label } from '@glyph/ui/label';

	let {
		label,
		description,
		layout = 'stack',
		size = 'md',
		for: htmlFor,
		class: className,
		children
	}: SettingsFieldProps = $props();

	const labelCls = size === 'sm' ? 'text-[13px]' : 'text-sm';
	const descCls = size === 'sm' ? 'text-muted-foreground text-xs' : 'text-muted-foreground text-xs';
</script>

{#if layout === 'row'}
	<div class={cn('flex items-center justify-between gap-3', className)} data-slot="settings-field">
		<div class="flex min-w-0 flex-col gap-0.5">
			<Label for={htmlFor} class={labelCls}>{label}</Label>
			{#if description}<span class={descCls}>{description}</span>{/if}
		</div>
		<div class="shrink-0">{@render children?.()}</div>
	</div>
{:else}
	<div class={cn('flex flex-col gap-2', className)} data-slot="settings-field">
		<div class="flex flex-col gap-0.5">
			<Label for={htmlFor} class={labelCls}>{label}</Label>
			{#if description}<span class={descCls}>{description}</span>{/if}
		</div>
		{@render children?.()}
	</div>
{/if}
