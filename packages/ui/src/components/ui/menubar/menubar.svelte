<script lang="ts">
	import { Menubar as MenubarPrimitive } from "bits-ui";
	import { cn } from "@glyph/ui/utils";

	let {
		ref = $bindable(null),
		value = $bindable(),
		class: className,
		...restProps
	}: MenubarPrimitive.RootProps = $props();
</script>

<MenubarPrimitive.Root
	bind:ref
	bind:value
	data-slot="menubar"
	class={cn("flex items-center gap-0.5", className)}
	{...restProps}
/>
