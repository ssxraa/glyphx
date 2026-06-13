<script lang="ts">
  import { Button, type ButtonSize } from "@glyphx/ui/button";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuShortcut,
    DropdownMenuTrigger
  } from "@glyphx/ui/dropdown-menu";
  import { toast } from "@glyphx/ui/sonner";
  import {
    IconCheck,
    IconChevronDown,
    IconCode,
    IconCopy,
    IconDownload,
    IconFileText,
    IconFileTypePdf
  } from "@tabler/icons-svelte";
  /**
   * ExportMenu — Export / Share. Every export goes through the host-injected
   * `saveFile` (desktop = Tauri's native "Save As") when present, else a plain
   * browser download. A compiled PDF is saved from its bytes; with no PDF yet,
   * PDF export falls back to print-to-PDF of the preview.
   */
  let {
    source = "",
    filename = "main.tex",
    pdfBytes,
    saveFile,
    size = "sm",
  }: {
    source?: string;
    filename?: string;
    pdfBytes?: Uint8Array;
    saveFile?: (
      bytes: Uint8Array,
      opts: { filename: string; extensions?: string[] },
    ) => Promise<boolean>;
    size?: ButtonSize;
  } = $props();

  let open = $state(false);
  let root = $state<HTMLDivElement>();

  const baseName = $derived(filename.replace(/\.[^./\\]+$/, "") || "document");

  // Save via the host's native dialog when available (desktop = Tauri), else a
  // plain browser download. Stays quiet if the user cancels the dialog.
  async function saveOrDownload(
    bytes: Uint8Array,
    name: string,
    extensions: string[],
  ) {
    open = false;
    try {
      if (saveFile) {
        const saved = await saveFile(bytes, { filename: name, extensions });
        if (saved) toast.success(`Saved ${name}`);
        return;
      }
      const blob = new Blob([bytes as BlobPart]);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = name;
      a.click();
      URL.revokeObjectURL(url);
      toast.success(`Exported ${name}`);
    } catch (e) {
      toast.error(`Could not save ${name} — ${e}`);
    }
  }

  let copied = $state(false);

  function exportPdf() {
    if (pdfBytes) {
      saveOrDownload(pdfBytes, `${baseName}.pdf`, ["pdf"]);
      return;
    }
    // No compiled PDF yet — fall back to print-to-PDF of the preview.
    open = false;
    toast.message("Opening the print dialog…");
    requestAnimationFrame(() => window.print());
  }
  const exportTex = () =>
    saveOrDownload(new TextEncoder().encode(source), `${baseName}.tex`, [
      "tex",
    ]);
  const exportTxt = () =>
    saveOrDownload(new TextEncoder().encode(source), `${baseName}.txt`, [
      "txt",
    ]);

  async function copySource() {
    try {
      await navigator.clipboard.writeText(source);
      copied = true;
      toast.success("Source copied to clipboard");
      setTimeout(() => (copied = false), 1500);
    } catch {
      toast.error("Could not copy — clipboard blocked");
    }
  }

  type Item = {
    icon: typeof IconDownload;
    label: string;
    hint?: string;
    run: () => void;
  };
  const items = $derived<Item[]>([
    {
      icon: IconFileTypePdf,
      label: "Export PDF",
      hint: pdfBytes ? "Save" : "Print",
      run: exportPdf,
    },
    { icon: IconCode, label: "Export .tex", run: exportTex },
    { icon: IconFileText, label: "Export plain text", run: exportTxt },
    {
      icon: copied ? IconCheck : IconCopy,
      label: copied ? "Copied" : "Copy source",
      run: copySource,
    },
  ]);

  $effect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (root && !root.contains(e.target as Node)) open = false;
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") open = false;
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  });
</script>

<DropdownMenu>
  <DropdownMenuTrigger>
    {#snippet child({ props })}
      <Button
        {...props}
        size="xs"
        variant="default_soft"
		
      >
        <IconDownload  />
        Export
        <IconChevronDown />
      </Button>
    {/snippet}
  </DropdownMenuTrigger>
  <DropdownMenuContent align="end" class="w-52">
   {#each items as item (item.label)}
      {@const Icon = item.icon}
	  <DropdownMenuItem onclick={item.run}>
		<Icon size={16} class="text-muted-foreground" />
		<span class="flex-1">{item.label}</span>
		{#if item.hint}
		  <DropdownMenuShortcut>{item.hint}</DropdownMenuShortcut>
		{/if}
	  </DropdownMenuItem>
	{/each}
  </DropdownMenuContent>
</DropdownMenu>
