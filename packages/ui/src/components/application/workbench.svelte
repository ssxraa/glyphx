<script lang="ts" module>
  export type ViewMode = "editor" | "split" | "preview";

  const SAMPLE_LATEX = String.raw`% Glyph — LaTeX document
\documentclass{article}
\usepackage{amsmath}

\title{Hello from Glyph}
\author{}
\date{}

\begin{document}
\maketitle

Glyph compiles \LaTeX{} entirely on your machine with Tectonic.
Nothing is uploaded. Nothing leaves this device.

\begin{equation}
  E = m c^2
\end{equation}

\end{document}
`;

  const SAMPLE_BIB = String.raw`@article{glyph2026,
  title   = {Local-first Typesetting},
  author  = {Glyph},
  journal = {Journal of Private Research},
  year    = {2026}
}
`;
</script>

<script lang="ts">
  import { Button } from "@glyph/ui/button";
  import { ButtonGroup } from "@glyph/ui/button-group";
  import {
    DropdownMenu,
    DropdownMenuCheckboxItem,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuShortcut,
    DropdownMenuTrigger,
  } from "@glyph/ui/dropdown-menu";
  import {
    parseLatexLog,
    parseSyncTex,
    summarizeProblems,
    type SyncTexLocation,
    type SyncTexMap,
  } from "@glyph/ui/editor";
  import { Logo } from "@glyph/ui/logo";
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
  } from "@glyph/ui/select";
  import { COMPILE_DEBOUNCE_MS, settings } from "@glyph/ui/settings";
  import { Toaster, toast } from "@glyph/ui/sonner";
  import { Spinner } from "@glyph/ui/spinner";
  import {
    IconAlertTriangle,
    IconChevronDown,
    IconCurrentLocation,
    IconDownload,
    IconEye,
    IconLayoutColumns,
    IconLoader2,
    IconMinus,
    IconPencil,
    IconPlayerPlayFilled,
    IconPlus,
    IconRefresh,
    IconSearch,
  } from "@tabler/icons-svelte";
  import { onDestroy, onMount } from "svelte";

  import ActivityBar, { type ActivityView } from "./activity-bar.svelte";
  import CodeEditor from "./code-editor.svelte";
  import CommandPalette from "./command-palette.svelte";
  import type { EngineManager } from "./engine-settings.svelte";
  import ExportMenu from "./export-menu.svelte";
  import FormatToolbar from "./format-toolbar.svelte";
  import MenuBar, { type Menu } from "./menu-bar.svelte";
  import PdfView from "./pdf-view.svelte";
  import ProblemsPanel from "./problems-panel.svelte";
  import type { ProjectHost } from "./project";
  import SidePanel from "./side-panel.svelte";

  /**
   * Workbench — the full editor experience. Top bar (logo · editable title ·
   * view-modes / theme / export), a left rail + side panel, and the editor /
   * preview area. Formatting lives in the editor panel header. Calm,
   * semantic-token-only chrome; the editor keeps its JetBrains identity.
   */
  // `path`/`loaded` are set for disk-backed project files: `path` is the
  // absolute on-disk path (also the stable `id`); `loaded` tracks whether the
  // content has been read from disk yet (lazy-loaded on first open).
  type GlyphFile = {
    id: string;
    name: string;
    content: string;
    path?: string;
    loaded?: boolean;
  };

  // Fallback document used when the host doesn't inject a project (e.g. the web
  // /editor route, or a quick demo).
  const DEMO_FILES: GlyphFile[] = [
    { id: "main", name: "main.tex", content: SAMPLE_LATEX },
    {
      id: "intro",
      name: "sections/introduction.tex",
      content: String.raw`\section{Introduction}

Local-first typesetting keeps your unpublished work on your own machine.
This section motivates the approach.
`,
    },
    {
      id: "results",
      name: "sections/results.tex",
      content: String.raw`\section{Results}

We observe that $\hat{\theta}$ is consistent, with $\alpha$ scaling as $\beta^2$.
`,
    },
    { id: "refs", name: "references.bib", content: SAMPLE_BIB },
  ];

  let {
    platform = "web" as "web" | "desktop",
    compile,
    compileProject,
    engine,
    saveFile,
    project,
    openPathOnMount,
    projectName = "glyph-project",
    initialFiles,
    onpersist,
  }: {
    platform?: "web" | "desktop";
    compile?: (source: string) => Promise<{
      pdf?: string;
      log?: string;
      error?: string;
      synctex?: string;
    }>;
    /**
     * Compile a multi-file project on disk (desktop). Given the project root and
     * the main file's path relative to it, runs the engine so `\input` /
     * `\includegraphics` / `\bibliography` resolve against the real folder. Used
     * whenever a project folder is open.
     */
    compileProject?: (
      root: string,
      mainRel: string,
    ) => Promise<{
      pdf?: string;
      log?: string;
      error?: string;
      synctex?: string;
    }>;
    engine?: EngineManager;
    /**
     * Host-injected file save (desktop = Tauri dialog + fs). Resolves `true`
     * when written, `false` when the user cancels; throws on failure. When
     * absent (web), the workbench falls back to a browser download.
     */
    saveFile?: (
      bytes: Uint8Array,
      opts: { filename: string; extensions?: string[] },
    ) => Promise<boolean>;
    /** Folder-based project bridge (desktop = Tauri fs / zip). Absent on web. */
    project?: ProjectHost;
    /**
     * A folder / `.tex` / `.glyx` path to open on mount (file-association launch,
     * routed by the app shell). Requires `project`.
     */
    openPathOnMount?: string;
    /** Workspace name shown in the command centre / explorer. */
    projectName?: string;
    /** Files to open with (a project's files). Defaults to a demo document. */
    initialFiles?: GlyphFile[];
    /** Called (debounced) whenever files change, so the host can persist. */
    onpersist?: (files: GlyphFile[]) => void;
  } = $props();

  const seedFiles =
    initialFiles && initialFiles.length ? initialFiles : DEMO_FILES;

  let files = $state<GlyphFile[]>(seedFiles.map((f) => ({ ...f })));
  let activeId = $state(seedFiles[0]?.id ?? "main");
  let source = $state(seedFiles[0]?.content ?? "");
  let untitledCount = $state(0);

  // --- Folder-based project (desktop) ---------------------------------------
  // When a folder is open, `projectRoot` is its absolute path, files are backed
  // by disk (lazy-loaded / saved through `project`), and `mainId` is the compile
  // target. Without a folder we stay on the in-memory demo / web document.
  let projectRoot = $state<string | null>(null);
  let mainId = $state<string | null>(null);

  const activeFile = $derived(files.find((f) => f.id === activeId) ?? files[0]);
  // Project name shown in chrome: the open folder's name, else the prop default.
  const displayName = $derived(
    projectRoot ? baseName(projectRoot) : projectName,
  );

  // --- Path helpers (OS-agnostic string ops; separator inferred from root) ---
  function sepOf(p: string): string {
    return p.includes("\\") ? "\\" : "/";
  }
  function baseName(p: string): string {
    const s = p.replace(/[\\/]+$/, "");
    const i = Math.max(s.lastIndexOf("/"), s.lastIndexOf("\\"));
    return i >= 0 ? s.slice(i + 1) : s;
  }
  function parentDir(p: string): string {
    const s = p.replace(/[\\/]+$/, "");
    const i = Math.max(s.lastIndexOf("/"), s.lastIndexOf("\\"));
    return i >= 0 ? s.slice(0, i) : s;
  }
  function joinPath(root: string, rel: string): string {
    const sep = sepOf(root);
    return root.replace(/[\\/]+$/, "") + sep + rel.split("/").join(sep);
  }
  function samePath(a?: string, b?: string): boolean {
    if (!a || !b) return false;
    const n = (s: string) => s.replace(/[\\/]+/g, "/").toLowerCase();
    return n(a) === n(b);
  }

  /** Write the active buffer back to memory (and disk, for project files). */
  async function flushActive(): Promise<void> {
    const f = files.find((x) => x.id === activeId);
    if (!f) return;
    f.content = source;
    if (project && f.path) {
      try {
        await project.writeFile(f.path, source);
      } catch (e) {
        toast.error(`Could not save ${f.name} — ${e}`);
      }
    }
  }

  async function openFile(id: string, force = false): Promise<void> {
    if (!force && id === activeId) return;
    await flushActive();
    activeId = id;
    const f = files.find((x) => x.id === id);
    if (f && project && f.path && !f.loaded) {
      try {
        f.content = await project.readFile(f.path);
      } catch (e) {
        f.content = `% Could not open this file — it may be binary or unreadable.\n% ${e}\n`;
      }
      f.loaded = true;
    }
    source = f?.content ?? "";
  }

  async function newFile(): Promise<void> {
    await flushActive();
    if (project && projectRoot) {
      untitledCount += 1;
      let rel = `untitled-${untitledCount}.tex`;
      let abs = joinPath(projectRoot, rel);
      // Avoid clobbering an existing file.
      while (await project.exists(abs)) {
        untitledCount += 1;
        rel = `untitled-${untitledCount}.tex`;
        abs = joinPath(projectRoot, rel);
      }
      try {
        await project.createEntry(abs, false);
      } catch (e) {
        toast.error(`Could not create file — ${e}`);
        return;
      }
      files = [
        ...files,
        { id: abs, name: rel, content: "", path: abs, loaded: true },
      ];
      activeId = abs;
      source = "";
      return;
    }
    untitledCount += 1;
    const id = `untitled-${untitledCount}`;
    files = [
      ...files,
      { id, name: `untitled-${untitledCount}.tex`, content: "" },
    ];
    activeId = id;
    source = "";
  }

  // Rename a file from the Explorer. The tree edits the *leaf* name; keep the
  // folder prefix so a nested file stays in its folder.
  async function renameFile(id: string, newLeaf: string): Promise<void> {
    const target = files.find((f) => f.id === id);
    const leaf = newLeaf.trim();
    if (!target || !leaf || leaf === baseName(target.name)) return;
    const slash = target.name.lastIndexOf("/");
    const dir = slash === -1 ? "" : target.name.slice(0, slash + 1);
    const newRel = dir + leaf;
    if (project && projectRoot && target.path) {
      const newAbs = joinPath(projectRoot, newRel);
      try {
        await project.rename(target.path, newAbs);
      } catch (e) {
        toast.error(`Rename failed — ${e}`);
        return;
      }
      const wasMain = id === mainId;
      files = files.map((f) =>
        f.id === id ? { ...f, id: newAbs, name: newRel, path: newAbs } : f,
      );
      if (id === activeId) activeId = newAbs;
      if (wasMain) {
        mainId = newAbs;
        void writeManifest();
      }
      return;
    }
    files = files.map((f) => (f.id === id ? { ...f, name: newRel } : f));
  }

  async function deleteFile(id: string): Promise<void> {
    const remaining = files.filter((f) => f.id !== id);
    if (remaining.length === 0 && !project) {
      toast.error("Can't delete the last file in a project.");
      return;
    }
    if (project) {
      const t = files.find((f) => f.id === id);
      if (t?.path) {
        try {
          await project.remove(t.path);
        } catch (e) {
          toast.error(`Delete failed — ${e}`);
          return;
        }
      }
    }
    const wasActive = id === activeId;
    files = remaining;
    if (id === mainId) {
      mainId = null;
      void writeManifest();
    }
    if (wasActive) {
      if (remaining[0]) await openFile(remaining[0].id, true);
      else {
        activeId = "";
        source = "";
      }
    }
    toast.success("Deleted");
  }

  /** Mark a file as the compile target and remember it in the `.glyx` manifest. */
  async function setMain(id: string): Promise<void> {
    mainId = id;
    await writeManifest();
    toast.success(
      `${baseName(files.find((f) => f.id === id)?.name ?? "")} is now the main file`,
    );
  }

  // The `.glyx` manifest (project root, named after the folder) records the main
  // file so reopening the folder — or double-clicking the .glyx — restores it.
  function manifestPath(): string | null {
    if (!projectRoot) return null;
    return joinPath(projectRoot, `${baseName(projectRoot)}.glyx`);
  }
  async function writeManifest(): Promise<void> {
    if (!project || !projectRoot) return;
    const mp = manifestPath();
    if (!mp) return;
    const mainRel = files.find((f) => f.id === mainId)?.name ?? null;
    const data = JSON.stringify(
      { glyph: 1, main: mainRel, name: baseName(projectRoot) },
      null,
      2,
    );
    try {
      await project.writeFile(mp, data);
    } catch {
      /* best-effort */
    }
  }

  // --- Open / import / export a project -------------------------------------
  /** Pick the main file from a manifest, else a sensible heuristic. */
  async function resolveMain(
    list: { abs: string; rel: string }[],
  ): Promise<string | null> {
    if (!project) return null;
    const glyx = list.find((f) => f.rel.toLowerCase().endsWith(".glyx"));
    if (glyx) {
      try {
        const m = JSON.parse(await project.readFile(glyx.abs));
        if (m && typeof m.main === "string") {
          const hit = list.find((f) => f.rel === m.main);
          if (hit) return hit.abs;
        }
      } catch {
        /* fall through to heuristics */
      }
    }
    const mainTex = list.find((f) => f.rel.toLowerCase() === "main.tex");
    if (mainTex) return mainTex.abs;
    const rootTex = list.find((f) => /^[^/]+\.tex$/i.test(f.rel));
    if (rootTex) return rootTex.abs;
    const anyTex = list.find((f) => f.rel.toLowerCase().endsWith(".tex"));
    return anyTex?.abs ?? list[0]?.abs ?? null;
  }

  async function loadProject(root: string, focusPath?: string): Promise<void> {
    if (!project) return;
    try {
      const list = await project.readFiles(root);
      projectRoot = root;
      // `.glyx` manifest is project metadata — keep it on disk, hide from the tree.
      const visible = list.filter(
        (f) => !f.rel.toLowerCase().endsWith(".glyx"),
      );
      files = visible.map((f) => ({
        id: f.abs,
        name: f.rel,
        content: "",
        path: f.abs,
        loaded: false,
      }));
      mainId = await resolveMain(list);
      untitledCount = 0;
      const focusId = focusPath
        ? files.find((f) => samePath(f.path, focusPath))?.id
        : undefined;
      const first = focusId ?? mainId ?? files[0]?.id;
      activeId = "";
      if (first) await openFile(first, true);
      else source = "";
      toast.success(`Opened ${baseName(root)}`);
    } catch (e) {
      toast.error(`Could not open project — ${e}`);
    }
  }

  /** Resolve a launched / picked path (folder, `.glyx`, or `.tex`) → open it. */
  async function openPath(p: string): Promise<void> {
    if (!project) return;
    const lower = p.toLowerCase();
    if (lower.endsWith(".glyx")) {
      await loadProject(parentDir(p));
    } else if (/\.[a-z0-9]+$/i.test(lower)) {
      await loadProject(parentDir(p), p);
    } else {
      await loadProject(p);
    }
  }

  async function openFolder(): Promise<void> {
    if (!project) return;
    const root = await project.pickFolder("Open LaTeX project folder");
    if (root) await loadProject(root);
  }

  async function importProject(): Promise<void> {
    if (!project) return;
    const file = await project.pickImportFile();
    if (!file) return;
    if (/\.zip$/i.test(file)) {
      const root = await project.importZip(file);
      if (root) await loadProject(root);
    } else {
      await openPath(file);
    }
  }

  async function exportProject(): Promise<void> {
    if (!project || !projectRoot) {
      toast.info("Open a project folder first to export it.");
      return;
    }
    await flushActive();
    try {
      const ok = await project.exportZip(
        projectRoot,
        `${baseName(projectRoot)}.zip`,
      );
      if (ok) toast.success("Exported project as ZIP");
    } catch (e) {
      toast.error(`Export failed — ${e}`);
    }
  }

  /** Register the OS "Open with Glyph" folder integration (desktop, Windows). */
  async function registerShell(): Promise<void> {
    if (!project?.registerShellIntegration) return;
    try {
      toast.success(await project.registerShellIntegration());
    } catch (e) {
      toast.error(`Could not register shell integration — ${e}`);
    }
  }

  // Open a folder / file the app shell routed us to (file-association launch),
  // and listen for later "Open with Glyph" launches forwarded while we're open.
  onMount(() => {
    if (!project) return;
    let unlisten: (() => void) | undefined;
    void (async () => {
      if (openPathOnMount) {
        try {
          await openPath(openPathOnMount);
        } catch {
          /* ignore — bad launch path */
        }
      }
      try {
        unlisten = await project.onOpenPath?.((path) => void openPath(path));
      } catch {
        /* event bridge unavailable */
      }
    })();
    return () => unlisten?.();
  });

  type SearchOptions = {
    query: string;
    replace?: string;
    caseSensitive?: boolean;
    wholeWord?: boolean;
    regexp?: boolean;
  };
  type SearchMatch = {
    from: number;
    to: number;
    line: number;
    column: number;
    text: string;
  };

  type EditorApi = {
    wrapSelection: (before: string, after?: string) => void;
    insertText: (text: string) => void;
    focusEditor: () => void;
    undo: () => void;
    redo: () => void;
    goToLine: (line: number) => void;
    findAll: (o: SearchOptions) => SearchMatch[];
    selectRange: (from: number, to: number) => void;
    replaceRange: (from: number, to: number, insert: string) => void;
    replaceAllMatches: (o: SearchOptions, replacement: string) => number;
    clearSearch: () => void;
  };
  let editor = $state<EditorApi>();

  let activeView = $state<ActivityView>("files");
  let panelCollapsed = $state(false);
  let viewMode = $state<ViewMode>("split");

  // Top bar: quick-open palette (the VS Code command centre). `projectName`
  // is a prop now (the open project's name).
  let paletteOpen = $state(false);

  // Snapshot of the current files (active file's live source merged in).
  function snapshotFiles(): GlyphFile[] {
    return files.map((f) =>
      f.id === activeId ? { ...f, content: source } : f,
    );
  }

  // Persist back to the host (project store) on any edit, debounced. Disabled
  // when a real folder is open — disk-backed projects save per-file via
  // `flushActive` instead of round-tripping the whole set.
  $effect(() => {
    void source; // track edits
    void files; // track add/remove/rename
    if (!onpersist || projectRoot) return;
    const t = setTimeout(() => onpersist(snapshotFiles()), 500);
    return () => clearTimeout(t);
  });

  // View-mode toggles — three plain icon buttons (editor / split / preview).
  const viewOptions = [
    { value: "editor" as const, icon: IconPencil, title: "Editor only" },
    { value: "split" as const, icon: IconLayoutColumns, title: "Split view" },
    { value: "preview" as const, icon: IconEye, title: "Preview only" },
  ];

  let cursor = $state({ line: 1, column: 1 });

  const lineCount = $derived(source.split("\n").length);
  const charCount = $derived(source.length);
  const wordCount = $derived(
    source.trim() ? source.trim().split(/\s+/).length : 0,
  );

  // --- Resizable split -------------------------------------------------------
  let splitPct = $state(52);
  let dragging = $state(false);
  let bodyEl = $state<HTMLElement>();

  // --- Resizable sidebar (drag the edge; capped at 20% of the shell width) ---
  let shellEl = $state<HTMLElement>();
  let shellW = $state(1280);
  let sidebarW = $state(240);
  let resizingSidebar = $state(false);
  const ACTIVITY_BAR_PX = 48; // the w-12 rail left of the panel
  const maxSidebar = $derived(Math.max(180, Math.round(shellW * 0.2)));
  const sidebarWidth = $derived(Math.min(sidebarW, maxSidebar));

  $effect(() => {
    if (!shellEl || typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(() => {
      if (shellEl) shellW = shellEl.getBoundingClientRect().width;
    });
    ro.observe(shellEl);
    return () => ro.disconnect();
  });

  function startResize() {
    if (viewMode === "split") dragging = true;
  }
  function startSidebarResize() {
    resizingSidebar = true;
  }
  function onPointerMove(e: PointerEvent) {
    if (resizingSidebar && shellEl) {
      const rect = shellEl.getBoundingClientRect();
      const w = e.clientX - rect.left - ACTIVITY_BAR_PX;
      sidebarW = Math.min(maxSidebar, Math.max(180, w));
      return;
    }
    if (!dragging || viewMode !== "split" || !bodyEl) return;
    const rect = bodyEl.getBoundingClientRect();
    const pct = ((e.clientX - rect.left) / rect.width) * 100;
    splitPct = Math.min(72, Math.max(28, pct));
  }
  function stopResize() {
    dragging = false;
    resizingSidebar = false;
  }

  function selectView(view: ActivityView) {
    if (view === activeView) panelCollapsed = !panelCollapsed;
    else {
      activeView = view;
      panelCollapsed = false;
    }
  }

  function cycleGrammar() {
    settings.grammar = settings.grammar === "legacy" ? "lezer" : "legacy";
  }

  // --- Find / replace (drives the side-panel Search view) -------------------
  let searchOpts = $state<SearchOptions>({
    query: "",
    replace: "",
    caseSensitive: false,
    wholeWord: false,
    regexp: false,
  });
  let searchResults = $state<SearchMatch[]>([]);
  let searchActive = $state(0);

  function runSearch(o: SearchOptions) {
    searchOpts = o;
    searchResults = o.query ? (editor?.findAll(o) ?? []) : [];
    searchActive = 0;
    if (!o.query) editor?.clearSearch();
  }
  function gotoResult(i: number) {
    if (!searchResults.length) return;
    const n = searchResults.length;
    searchActive = ((i % n) + n) % n;
    const m = searchResults[searchActive];
    editor?.selectRange(m.from, m.to);
  }
  function searchNext() {
    gotoResult(searchActive + 1);
  }
  function searchPrev() {
    gotoResult(searchActive - 1);
  }
  function replaceCurrent(replace: string) {
    const m = searchResults[searchActive];
    if (!m) return;
    let insert = replace;
    if (searchOpts.regexp) {
      try {
        let pat = searchOpts.query;
        if (searchOpts.wholeWord) pat = `\\b(?:${pat})\\b`;
        const single = new RegExp(pat, searchOpts.caseSensitive ? "" : "i");
        insert = source.slice(m.from, m.to).replace(single, replace);
      } catch {
        /* fall back to literal */
      }
    }
    editor?.replaceRange(m.from, m.to, insert);
    runSearch({ ...searchOpts, replace });
  }
  function replaceAll(replace: string) {
    const n = editor?.replaceAllMatches(searchOpts, replace) ?? 0;
    runSearch({ ...searchOpts, replace });
    toast.success(`Replaced ${n} ${n === 1 ? "match" : "matches"}`);
  }

  // Clear the editor highlight when the Search view isn't showing.
  $effect(() => {
    if (activeView !== "search" || panelCollapsed) editor?.clearSearch();
  });

  // --- Compilation (Tectonic on desktop; placeholder elsewhere) -------------
  // Live, debounced recompile-as-you-type. Tectonic is a heavier subprocess
  // than a WASM engine, so compiles are *coalesced*: while one is running we
  // never start a second — we mark the doc dirty and recompile once it lands.
  type CompileStatus = "idle" | "compiling" | "success" | "error";
  let compiling = $state(false);
  let compileStatus = $state<CompileStatus>("idle");
  let lastCompileMs = $state<number | null>(null);
  let pendingRecompile = false;
  let lastCompiledSource: string | null = null;
  let pdfUrl = $state<string | undefined>(undefined);
  let pdfBytes = $state<Uint8Array | undefined>(undefined);
  let synctex = $state<SyncTexMap | undefined>(undefined);
  let compileError = $state<string | undefined>(undefined);
  let compileLog = $state("");

  // Parsed diagnostics for the Problems panel; recomputed on each compile.
  const problems = $derived(parseLatexLog(compileLog, compileError));
  const problemSummary = $derived(summarizeProblems(problems));
  let showProblems = $state(false);

  const projectCompile = $derived(
    Boolean(compileProject && projectRoot && mainId),
  );
  const canCompile = $derived(Boolean(compile) || projectCompile);

  function base64ToBytes(b64: string): Uint8Array {
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return bytes;
  }

  /** Double-click in the PDF → jump the editor to the matching source line. */
  function onReverse(loc: { page: number; x: number; y: number }) {
    const hit = synctex?.locate(loc.page, loc.x, loc.y);
    if (hit) editor?.goToLine(hit.line);
  }

  let pdfView = $state<{
    revealLocation: (loc: SyncTexLocation) => void;
    zoomIn: () => void;
    zoomOut: () => void;
    setZoomPct: (pct: number) => void;
    fitWidth: () => void;
    openFind: () => void;
  }>();

  // PDF preview zoom/page state — bound from PdfView, driven by the header.
  let pdfScalePct = $state(100);
  let pdfFitMode = $state(true);
  let pdfNumPages = $state(0);
  const ZOOM_PRESETS = [50, 75, 100, 125, 150, 200];

  /**
   * Save the compiled PDF from the preview toolbar. On desktop the host injects
   * `saveFile` (Tauri's dialog + fs plugins → a real native "Save As"); on the
   * web we fall back to a normal browser download.
   */
  async function downloadPdf() {
    if (!pdfBytes) return;
    const srcName =
      files.find((f) => f.id === mainId)?.name ??
      activeFile?.name ??
      "document";
    const base = baseName(srcName).replace(/\.[^./\\]+$/, "") || "document";
    const filename = `${base}.pdf`;
    try {
      if (saveFile) {
        const saved = await saveFile(pdfBytes, {
          filename,
          extensions: ["pdf"],
        });
        if (saved) toast.success(`Saved ${filename}`);
        return; // false = user cancelled the dialog → stay quiet
      }
      if (!pdfUrl) return;
      const a = document.createElement("a");
      a.href = pdfUrl;
      a.download = filename;
      a.click();
      toast.success(`Downloaded ${filename}`);
    } catch (e) {
      toast.error(`Could not save the PDF — ${e}`);
    }
  }

  /** Forward sync: caret line → scroll/flash the matching region in the PDF. */
  function syncToPdf() {
    if (!synctex) {
      toast.info("Compile first to sync to the PDF.");
      return;
    }
    const loc = synctex.forward(cursor.line);
    if (!loc) {
      toast.info(`No PDF location for line ${cursor.line}.`);
      return;
    }
    if (viewMode === "editor") viewMode = "split";
    pdfView?.revealLocation(loc);
  }

  async function runCompile(manual = false) {
    if (!canCompile) {
      compileError = "Compilation runs in the Glyph desktop app.";
      compileStatus = "error";
      if (manual && viewMode === "editor") viewMode = "split";
      return;
    }
    // Coalesce: a compile is already in flight — queue exactly one rerun.
    if (compiling) {
      pendingRecompile = true;
      return;
    }
    if (manual && viewMode === "editor") viewMode = "split";
    compiling = true;
    try {
      do {
        pendingRecompile = false;
        // Flush the active buffer (to disk for project files) so the engine
        // sees the latest source before it runs.
        await flushActive();
        const snapshot = source;
        const useProject = projectCompile;
        // Skip redundant auto-recompiles of already-rendered content. Only in
        // single-file mode — in a project, an edit to a non-active file (saved
        // on switch) wouldn't change `source`, so we always recompile there.
        if (!manual && !useProject && snapshot === lastCompiledSource) break;
        compileStatus = "compiling";
        const started = performance.now();
        const out =
          useProject && compileProject && projectRoot
            ? await compileProject(
                projectRoot,
                files.find((f) => f.id === mainId)?.name ?? "",
              )
            : compile
              ? await compile(snapshot)
              : { error: "No compiler available." };
        lastCompileMs = Math.round(performance.now() - started);
        compileLog = out.log ?? "";
        // Mirror to the devtools console so logs are readable/debuggable.
        if (out.error) {
          console.error(
            `[Glyph] LaTeX compilation failed (${lastCompileMs}ms): ${out.error}`,
          );
          if (out.log) console.error(out.log);
        } else {
          console.info(`[Glyph] compiled in ${lastCompileMs}ms`);
          if (out.log?.trim()) console.debug(out.log);
        }
        if (out.pdf) {
          if (pdfUrl) URL.revokeObjectURL(pdfUrl);
          const bytes = base64ToBytes(out.pdf);
          pdfBytes = bytes;
          pdfUrl = URL.createObjectURL(
            new Blob([bytes as BlobPart], { type: "application/pdf" }),
          );
          synctex = out.synctex ? parseSyncTex(out.synctex) : undefined;
          compileError = undefined;
          compileStatus = "success";
          lastCompiledSource = snapshot;
          // A best-effort PDF can still carry errors (e.g. an undefined macro
          // that the engine recovered from). Surface them like Overleaf does.
          if (problemSummary.errors > 0) showProblems = true;
        } else {
          compileError = out.error ?? "Compilation failed.";
          compileStatus = "error";
          showProblems = true; // surface failures immediately
        }
      } while (pendingRecompile);
    } catch (e) {
      compileError = String(e);
      compileStatus = "error";
      showProblems = true;
      console.error("[Glyph] compile threw:", e);
    } finally {
      compiling = false;
    }
  }

  // Debounced auto-compile: re-arm on every edit (and on file switch, since
  // `source` changes). Disabled when the user turns auto-compile off or no
  // engine is wired (web build leaves the prop unset when desktop-only).
  $effect(() => {
    void source; // track edits + file switches
    void mainId; // recompile when the main file changes
    const auto = settings.autoCompile;
    if (!canCompile || !auto) return;
    const timer = setTimeout(() => runCompile(false), COMPILE_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  const compileLabel = $derived(
    !canCompile
      ? "Desktop only"
      : compileStatus === "compiling"
        ? "Compiling…"
        : compileStatus === "error"
          ? "Compile error"
          : compileStatus === "success"
            ? lastCompileMs != null
              ? `Compiled in ${(lastCompileMs / 1000).toFixed(1)}s`
              : "Compiled"
            : "Ready",
  );

  // VS Code-style application menu. `$derived` so checkmarks (view mode, live
  // compile, open panel) stay in sync with state. Actions reuse the same
  // handlers the rest of the chrome calls.
  const menus = $derived<Menu[]>([
    {
      label: "File",
      items: [
        { label: "New File", run: () => newFile() },
        {
          label: "Open File…",
          shortcut: "⌘P",
          run: () => (paletteOpen = true),
        },
        {
          label: "Open Folder…",
          shortcut: "⌘O",
          disabled: !project,
          run: () => openFolder(),
        },
        { type: "separator" },
        {
          label: "Import Project…",
          disabled: !project,
          run: () => importProject(),
        },
        {
          label: "Export as Zip",
          disabled: !project || !projectRoot,
          run: () => exportProject(),
        },
        { type: "separator" },
        {
          label: "Compile",
          shortcut: "⌘S",
          disabled: !canCompile,
          run: () => runCompile(true),
        },
      ],
    },
    {
      label: "Edit",
      items: [
        { label: "Undo", shortcut: "⌘Z", run: () => editor?.undo() },
        { label: "Redo", shortcut: "⌘⇧Z", run: () => editor?.redo() },
        { type: "separator" },
        { label: "Bold", run: () => editor?.wrapSelection("\\textbf{", "}") },
        { label: "Italic", run: () => editor?.wrapSelection("\\textit{", "}") },
        { type: "separator" },
        {
          label: "Insert Section",
          run: () => editor?.insertText("\\section{}\n"),
        },
        {
          label: "Insert List",
          run: () =>
            editor?.insertText("\\begin{itemize}\n  \\item \n\\end{itemize}\n"),
        },
        {
          label: "Insert Equation",
          run: () =>
            editor?.insertText("\\begin{equation}\n  \n\\end{equation}\n"),
        },
        { type: "separator" },
        {
          label: "Find in File",
          shortcut: "⌘F",
          run: () => {
            activeView = "search";
            panelCollapsed = false;
          },
        },
      ],
    },
    {
      label: "View",
      items: [
        {
          label: "Explorer",
          checked: !panelCollapsed && activeView === "files",
          run: () => selectView("files"),
        },
        {
          label: "Search",
          checked: !panelCollapsed && activeView === "search",
          run: () => selectView("search"),
        },
        {
          label: "Source Control",
          checked: !panelCollapsed && activeView === "git",
          run: () => selectView("git"),
        },
        { type: "separator" },
        {
          label: "Editor",
          checked: viewMode === "editor",
          run: () => (viewMode = "editor"),
        },
        {
          label: "Split",
          checked: viewMode === "split",
          run: () => (viewMode = "split"),
        },
        {
          label: "Preview",
          checked: viewMode === "preview",
          run: () => (viewMode = "preview"),
        },
        { type: "separator" },
        {
          label: "Toggle Sidebar",
          checked: !panelCollapsed,
          run: () => (panelCollapsed = !panelCollapsed),
        },
        {
          label: "Problems",
          checked: showProblems,
          run: () => (showProblems = !showProblems),
        },
      ],
    },
    {
      label: "Go",
      items: [
        {
          label: "Go to File…",
          shortcut: "⌘P",
          run: () => (paletteOpen = true),
        },
        { type: "separator" },
        { label: "Sync to PDF", shortcut: "⌘J", run: () => syncToPdf() },
      ],
    },
    {
      label: "Run",
      items: [
        {
          label: "Compile",
          shortcut: "⌘S",
          disabled: !canCompile,
          run: () => runCompile(true),
        },
        { type: "separator" },
        {
          label: "Live Compile",
          checked: settings.autoCompile,
          run: () => (settings.autoCompile = !settings.autoCompile),
        },
        { type: "separator" },
        { label: "Sync to PDF", shortcut: "⌘J", run: () => syncToPdf() },
      ],
    },
    {
      label: "Help",
      items: [
        { label: "Keyboard Shortcuts", disabled: true },
        { type: "separator" },
        {
          label: "About Glyph",
          run: () => toast.message(`Glyph — local-first LaTeX (${platform})`),
        },
      ],
    },
  ]);

  function onKeydown(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey)) return;
    // Ctrl/Cmd+S or Ctrl/Cmd+Enter forces an immediate compile.
    if (e.key === "s" || e.key === "Enter") {
      e.preventDefault();
      runCompile(true);
    }
    // Ctrl/Cmd+J jumps the PDF to the caret's line (forward sync).
    if (e.key === "j" || e.key === "J") {
      e.preventDefault();
      syncToPdf();
    }
    // Ctrl/Cmd+P opens the quick-open palette.
    if (e.key === "p" || e.key === "P") {
      e.preventDefault();
      paletteOpen = true;
    }
    // Ctrl/Cmd+O opens a project folder (desktop).
    if ((e.key === "o" || e.key === "O") && project) {
      e.preventDefault();
      void openFolder();
    }
    // Ctrl/Cmd+F opens the Search view in the side panel.
    if (e.key === "f" || e.key === "F") {
      e.preventDefault();
      activeView = "search";
      panelCollapsed = false;
    }
  }

  onDestroy(() => {
    if (pdfUrl) URL.revokeObjectURL(pdfUrl);
  });
</script>

<svelte:window
  onpointermove={onPointerMove}
  onpointerup={stopResize}
  onkeydown={onKeydown}
/>

<div class="bg-background text-foreground flex h-dvh flex-col overflow-hidden">
  <!-- Top bar — VS Code-style: logo + menu (left) · workspace (centre) ·
	     view toggles + export + compile (right). Theme & settings live in the
	     Settings panel, not here. -->
  <header
    class="border-border bg-card flex h-11 shrink-0 items-center gap-2 border-b px-2.5"
  >
    <!-- Left: logo + application menu -->
    <div class="flex shrink-0 items-center gap-1.5">
      <Logo
        href="/"
        text={false}
        size="md"
        viewTransitionName="app-logo"
        class="pr-1"
      />
      <MenuBar {menus} />
    </div>

    <!-- Centre: workspace name / quick-open (⌘P) -->
    <div class="flex flex-1 justify-center">
      <CommandPalette
        bind:open={paletteOpen}
        {files}
        {activeId}
        projectName={displayName}
        onopen={openFile}
      />
    </div>

    <!-- Right: view toggles · export · compile -->
    <div class="inline-flex shrink-0 items-center gap-2">
      <Select bind:value={viewMode} type="single" name="viewMode">
        <SelectTrigger size="sm" class="w-auto min-w-0 border-0" aria-label="Select view mode">
          {@const Icon = viewOptions.find((o) => o.value === viewMode)?.icon}
          {#if Icon}
            <Icon class="inline-block" />
          {/if}
          {viewMode === "editor"
            ? "Editor only"
            : viewMode === "split"
              ? "Split view"
              : "Preview only"}
        </SelectTrigger>
        <SelectContent>
          {#each viewOptions as option (option.value)}
            <SelectItem value={option.value}>{option.title}</SelectItem>
          {/each}
        </SelectContent>
      </Select>

      <ExportMenu
        {source}
        filename={activeFile?.name ?? "document.tex"}
        {pdfBytes}
        {saveFile}
        size="default"
      />

      <!-- Compile split-button: run + a ▾ menu for live-compile / sync. -->
      <ButtonGroup>
        <Button onclick={() => runCompile(true)} disabled={compiling} size="xs">
          {#if compiling}
            <IconLoader2 class="animate-spin" />
          {:else}
            <IconPlayerPlayFilled />
          {/if}
          {compiling ? "Compiling…" : "Compile"}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger>
            {#snippet child({ props })}
              <Button
                {...props}
                size="icon-xs"
                title="Compile options"
                aria-label="Compile options"
              >
                <IconChevronDown />
              </Button>
            {/snippet}
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" class="w-52">
            <DropdownMenuCheckboxItem
              checked={settings.autoCompile}
              onCheckedChange={(v) => (settings.autoCompile = v)}
            >
              Live compile
            </DropdownMenuCheckboxItem>
            <DropdownMenuItem
              disabled={!canCompile}
              onclick={() => runCompile(true)}
            >
              Compile once
              <DropdownMenuShortcut>⌘S</DropdownMenuShortcut>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onclick={() => syncToPdf()}>
              Sync to PDF
              <DropdownMenuShortcut>⌘J</DropdownMenuShortcut>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </ButtonGroup>
    </div>
  </header>

  <!-- Body -->
  <div bind:this={shellEl} class="flex min-h-0 flex-1">
    <ActivityBar active={activeView} onselect={selectView} />

    <!-- Smooth collapse + drag-to-resize (panel stays mounted; capped at 20%). -->
    <div
      class="shrink-0 overflow-hidden {resizingSidebar
        ? ''
        : 'transition-[width] duration-300 ease-[cubic-bezier(0.625,0.05,0,1)]'} {panelCollapsed
        ? 'pointer-events-none'
        : ''}"
      style:width={panelCollapsed ? "0px" : `${sidebarWidth}px`}
      aria-hidden={panelCollapsed}
    >
      <SidePanel
        view={activeView}
        {files}
        {activeId}
        {mainId}
        projectName={displayName}
        hasProject={Boolean(project)}
        {engine}
        widthPx={sidebarWidth}
        onopen={openFile}
        onnew={newFile}
        onopenfolder={openFolder}
        onrenamefile={renameFile}
        ondeletefile={deleteFile}
        onsetmain={setMain}
        onregistershell={project?.registerShellIntegration
          ? registerShell
          : undefined}
        {searchResults}
        {searchActive}
        onsearch={runSearch}
        ongotoresult={gotoResult}
        onsearchnext={searchNext}
        onsearchprev={searchPrev}
        onreplacecurrent={replaceCurrent}
        onreplaceall={replaceAll}
      />
    </div>

    {#if !panelCollapsed}
      <div
        class="group relative z-10 flex w-1 shrink-0 cursor-col-resize touch-none items-center justify-center"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize sidebar"
        tabindex="-1"
        onpointerdown={startSidebarResize}
      >
        <span
          class="h-10 w-0.5 rounded-full transition-colors {resizingSidebar
            ? 'bg-primary'
            : 'bg-border group-hover:bg-primary/60'}"
        ></span>
      </div>
    {/if}

    <!-- min-w-0 lets these flex children shrink below their content's intrinsic
		     width, so a wide PDF page / long log line can't push the layout past the
		     window edge (which would hide the preview toolbar + log copy button). -->
    <main class="flex min-h-0 min-w-0 flex-1 flex-col">
      <div bind:this={bodyEl} class="flex min-h-0 min-w-0 flex-1">
        {#if viewMode !== "preview"}
          <section
            class="flex min-h-0 min-w-0 shrink-0 flex-col overflow-hidden {viewMode ===
            'split'
              ? 'border-border border-r'
              : ''}"
            style={viewMode === "split" ? `width:${splitPct}%` : "width:100%"}
          >
            <div
              class="text-muted-foreground border-border flex h-9 shrink-0 items-center gap-2 border-b px-2 text-xs"
            >
              <span class="px-1 font-medium">Source</span>
              <span class="text-muted-foreground/60">{lineCount} lines</span>
              <div class="ml-auto flex items-center gap-1">
                <button
                  class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
                  title="Sync to PDF (⌘/Ctrl+J)"
                  aria-label="Sync to PDF"
                  onclick={syncToPdf}
                >
                  <IconCurrentLocation size={15} />
                </button>
                <FormatToolbar
                  wrap={(b, a) => editor?.wrapSelection(b, a)}
                  insert={(t) => editor?.insertText(t)}
                />
              </div>
            </div>
            <div class="min-h-0 flex-1">
              <CodeEditor
                bind:this={editor}
                bind:value={source}
                theme={settings.resolved}
                grammar={settings.grammar}
                fontSize={settings.fontSize}
                fontFamily={settings.fontStack}
                lineWrapping={settings.lineWrapping}
                oncursor={(p) => (cursor = p)}
              />
            </div>
          </section>
        {/if}

        {#if viewMode === "split"}
          <div
            class="group relative z-10 flex w-1 shrink-0 cursor-col-resize touch-none items-center justify-center"
            role="separator"
            aria-orientation="vertical"
            aria-valuenow={Math.round(splitPct)}
            tabindex="-1"
            onpointerdown={startResize}
          >
            <span
              class="h-10 w-0.5 rounded-full transition-colors {dragging
                ? 'bg-primary'
                : 'bg-border group-hover:bg-primary/60'}"
            ></span>
          </div>
        {/if}

        {#if viewMode !== "editor"}
          <section class="bg-muted/40 flex min-h-0 min-w-0 flex-1 flex-col">
            <div
              class="text-muted-foreground border-border flex h-9 shrink-0 items-center gap-1.5 border-b px-2 text-xs"
            >
              <!-- Status + recompile -->
              <span
                class="inline-flex min-w-0 items-center gap-1.5 truncate pl-1 {compileStatus ===
                'error'
                  ? 'text-destructive'
                  : 'text-muted-foreground/80'}"
              >
                {#if compileStatus === "compiling"}
                  <IconLoader2 size={12} class="animate-spin" />
                {:else if compileStatus === "error"}
                  <IconAlertTriangle size={12} />
                {:else}
                  <span
                    class="size-1.5 rounded-full {compileStatus === 'success'
                      ? 'bg-success'
                      : 'bg-muted-foreground/40'}"
                  ></span>
                {/if}
                {compileLabel}
              </span>
              <button
                class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors disabled:opacity-40 disabled:hover:bg-transparent"
                title="Recompile (⌘/Ctrl+S)"
                aria-label="Recompile"
                disabled={!canCompile || compiling}
                onclick={() => runCompile(true)}
              >
                <IconRefresh
                  size={13}
                  class={compiling ? "animate-spin" : ""}
                />
              </button>

              {#if pdfBytes}
                <!-- Find + page count + zoom + download -->
                <div class="ml-auto flex items-center gap-1">
                  <button
                    class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
                    title="Find in PDF (Ctrl/Cmd+F)"
                    aria-label="Find in PDF"
                    onclick={() => pdfView?.openFind()}
                  >
                    <IconSearch size={14} />
                  </button>
                  <span class="bg-border mx-1 h-4 w-px"></span>
                  <span class="text-muted-foreground/70 tabular-nums">
                    {pdfNumPages || 1} page{(pdfNumPages || 1) === 1 ? "" : "s"}
                  </span>
                  <span class="bg-border mx-1 h-4 w-px"></span>
                  <button
                    class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
                    title="Zoom out"
                    aria-label="Zoom out"
                    onclick={() => pdfView?.zoomOut()}
                  >
                    <IconMinus size={14} />
                  </button>
                  <DropdownMenu>
                    <DropdownMenuTrigger>
                      {#snippet child({ props })}
                        <button
                          {...props}
                          class="hover:bg-muted hover:text-foreground flex h-6 items-center gap-1 rounded px-1.5 tabular-nums transition-colors"
                          title="Zoom level"
                        >
                          {pdfFitMode ? "Fit" : `${pdfScalePct}%`}
                          <IconChevronDown size={12} class="opacity-60" />
                        </button>
                      {/snippet}
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" class="w-36">
                      <DropdownMenuItem onclick={() => pdfView?.fitWidth()}>
                        Fit width
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      {#each ZOOM_PRESETS as pct (pct)}
                        <DropdownMenuItem
                          onclick={() => pdfView?.setZoomPct(pct)}
                        >
                          {pct}%
                        </DropdownMenuItem>
                      {/each}
                    </DropdownMenuContent>
                  </DropdownMenu>
                  <button
                    class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
                    title="Zoom in"
                    aria-label="Zoom in"
                    onclick={() => pdfView?.zoomIn()}
                  >
                    <IconPlus size={14} />
                  </button>
                  <span class="bg-border mx-1 h-4 w-px"></span>
                  <button
                    class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
                    title="Download PDF"
                    aria-label="Download PDF"
                    onclick={downloadPdf}
                  >
                    <IconDownload size={14} />
                  </button>
                </div>
              {/if}
            </div>
            <div class="min-h-0 flex-1">
              {#if pdfBytes}
                <PdfView
                  bind:this={pdfView}
                  data={pdfBytes}
                  onreverse={onReverse}
                  bind:scalePct={pdfScalePct}
                  bind:fitMode={pdfFitMode}
                  bind:numPages={pdfNumPages}
                />
              {:else}
                <div class="h-full overflow-auto p-6">
                  {#if compileError}
                    <div
                      class="border-destructive/30 bg-destructive/5 mx-auto max-w-prose rounded-lg border p-4"
                    >
                      <p class="text-destructive text-sm font-medium">
                        {compileError}
                      </p>
                      {#if compileLog}
                        <pre
                          class="text-muted-foreground mt-3 max-h-72 overflow-auto font-mono text-[11px] whitespace-pre-wrap">{compileLog}</pre>
                      {/if}
                    </div>
                  {:else}
                    <div
                      class="glyph-print-area flex h-full flex-col items-center justify-center gap-6 text-center"
                    >
                      <Logo text={false} badge size={64} class="opacity-95" />
                      {#if compileStatus === "compiling"}
                        <div
                          class="text-muted-foreground flex items-center gap-2.5 text-sm"
                        >
                          <Spinner class="size-4" />
                          <span>Rendering your document…</span>
                        </div>
                      {:else if canCompile}
                        <div class="flex flex-col items-center gap-1.5">
                          <p class="text-foreground text-sm font-medium">
                            Nothing to preview yet
                          </p>
                          <p
                            class="text-muted-foreground max-w-[18rem] text-xs leading-relaxed"
                          >
                            {settings.autoCompile
                              ? "Start typing — Glyph renders live, entirely on your device."
                              : "Press Compile (⌘/Ctrl+S) to render — entirely on your device."}
                          </p>
                        </div>
                      {:else}
                        <div class="flex flex-col items-center gap-1.5">
                          <p class="text-foreground text-sm font-medium">
                            Preview is desktop-only
                          </p>
                          <p
                            class="text-muted-foreground max-w-[18rem] text-xs leading-relaxed"
                          >
                            Compilation runs in the Glyph desktop app — fully
                            offline.
                          </p>
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          </section>
        {/if}
      </div>

      {#if showProblems}
        <ProblemsPanel
          {problems}
          log={compileLog}
          ongoto={(l) => {
            editor?.goToLine(l);
            if (viewMode === "preview") viewMode = "split";
          }}
          onclose={() => (showProblems = false)}
        />
      {/if}

      <!-- Status bar -->
      <footer
        class="border-border bg-card text-muted-foreground flex h-6 shrink-0 items-center gap-3 border-t px-3 text-[11px]"
      >
        <span
          class="flex items-center gap-1.5 {compileStatus === 'error'
            ? 'text-destructive'
            : ''}"
        >
          <span
            class="size-1.5 rounded-full {compileStatus === 'compiling'
              ? 'bg-primary animate-pulse'
              : compileStatus === 'error'
                ? 'bg-destructive'
                : 'bg-success'}"
          ></span>
          {compileLabel}
        </span>
        <span class="text-muted-foreground/50">·</span>
        <span>{settings.autoCompile ? "Auto" : "Manual"}</span>
        <span>LaTeX · Tectonic</span>
        <button
          class="hover:text-foreground transition-colors"
          title="Toggle LaTeX grammar"
          onclick={cycleGrammar}
        >
          {settings.grammar === "legacy" ? "stex" : "lezer"}
        </button>
        <button
          class="inline-flex items-center gap-1 transition-colors hover:text-foreground {problemSummary.errors
            ? 'text-destructive'
            : problemSummary.warnings
              ? 'text-warning'
              : ''}"
          title="Toggle problems / log"
          aria-pressed={showProblems}
          onclick={() => (showProblems = !showProblems)}
        >
          <IconAlertTriangle size={12} />
          <span class="tabular-nums"
            >{problemSummary.errors}/{problemSummary.warnings}</span
          >
        </button>
        <span class="text-muted-foreground/50 ml-auto"
          >Ln {cursor.line}, Col {cursor.column}</span
        >
        <span class="text-muted-foreground/50"
          >{wordCount} words · {charCount} chars</span
        >
        <span class="text-muted-foreground/50 capitalize">{platform}</span>
      </footer>
    </main>
  </div>
</div>

<!-- Toast feedback (bottom-right; matches the app's corner-notification language). -->
<Toaster />

<style>
  /* PDF export = print the preview page only, until Tectonic compiles for real. */
  @media print {
    :global(body *) {
      visibility: hidden !important;
    }
    :global(.glyph-print-area),
    :global(.glyph-print-area *) {
      visibility: visible !important;
    }
    :global(.glyph-print-area) {
      position: fixed;
      inset: 0;
      max-width: none;
      border: none;
      overflow: visible;
    }
  }
</style>
