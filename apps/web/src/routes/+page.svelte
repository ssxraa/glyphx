<script lang="ts">
	import { resolve } from '$app/paths';
	import FloatingGlyphs from '$lib/FloatingGlyphs.svelte';
	import SiteFooter from '$lib/SiteFooter.svelte';
	import SiteHeader from '$lib/SiteHeader.svelte';
	import { CardStack, CardStackItem, SplitReveal } from '$lib/motion-core';
	import { trackEvent } from '$lib/analytics';
	import { Reveal } from '@glyphx/ui/reveal';
	import {
		IconAlertTriangle,
		IconArrowRight,
		IconBolt,
		IconBrandDropbox,
		IconCheck,
		IconCloud,
		IconCpu,
		IconDownload,
		IconFileText,
		IconFolder,
		IconGitBranch,
		IconKey,
		IconLayoutColumns,
		IconLock,
		IconMinus,
		IconPlayerPlay,
		IconShare,
		IconSparkles,
		IconWifiOff
	} from '@tabler/icons-svelte';

	// Core capabilities, shown as a scroll-driven card stack. Everything here
	// ships today. `visual` selects the small illustration on each card.
	type Visual = 'code' | 'compile' | 'split' | 'git' | 'private';
	type Capability = { icon: typeof IconFileText; title: string; body: string; visual: Visual };
	const capabilities: Capability[] = [
		{
			icon: IconFileText,
			title: 'Real LaTeX, not a lookalike',
			body: 'Full math, figures, BibTeX, and the packages a journal template or thesis class actually needs. What you write is standard .tex, and the PDF you get is the PDF your reviewer gets.',
			visual: 'code'
		},
		{
			icon: IconBolt,
			title: 'Compiles on your machine',
			body: 'The engine runs on your computer. No shared queue, no server, and no timeout the night before a deadline. The preview keeps up while you type.',
			visual: 'compile'
		},
		{
			icon: IconLayoutColumns,
			title: 'Source and page, side by side',
			body: 'Write on the left, watch the document render on the right. Double click a spot in the PDF to jump to the line that made it, and jump the other way too.',
			visual: 'split'
		},
		{
			icon: IconGitBranch,
			title: 'Git built in, no paywall',
			body: 'Stage and commit, read a side by side diff, browse history, clone a repository, and push, pull, or sync with your own remote. Real version control, no subscription tier.',
			visual: 'git'
		},
		{
			icon: IconLock,
			title: 'Private and offline by default',
			body: 'Unpublished results, grant drafts, a thesis under embargo. None of it is uploaded, indexed, or fed to a model. It sits on your disk, and the editor never waits on a connection.',
			visual: 'private'
		}
	];

	// Recurring Overleaf frustrations, paraphrased from what people actually hit.
	// Rendered as a faux "notifications" panel: these are the real messages a
	// cloud editor shows you. `tone` colors the severity; `tag` is the source.
	type Tone = 'red' | 'amber' | 'muted';
	type PainPoint = { quote: string; body: string; tone: Tone; icon: typeof IconLock; tag: string };
	const painPoints: PainPoint[] = [
		{
			quote: 'Compile timed out.',
			body: 'On the free plan a long chapter can stop building right when you need the PDF most.',
			tone: 'red',
			icon: IconAlertTriangle,
			tag: 'free tier'
		},
		{
			quote: 'Upgrade to keep working.',
			body: 'More collaborators, Git, and Dropbox sync all sit behind the paid tier.',
			tone: 'amber',
			icon: IconLock,
			tag: 'premium'
		},
		{
			quote: 'Where did my history go?',
			body: 'Full version history is a premium feature, so a free account only sees so far back.',
			tone: 'amber',
			icon: IconLock,
			tag: 'premium'
		},
		{
			quote: 'Why is the editor lagging?',
			body: 'Long documents get sluggish because every keystroke round trips through a server.',
			tone: 'red',
			icon: IconAlertTriangle,
			tag: 'server round trip'
		},
		{
			quote: 'Is my draft actually private?',
			body: 'Your unpublished work lives on infrastructure you do not own or control.',
			tone: 'amber',
			icon: IconAlertTriangle,
			tag: 'their servers'
		},
		{
			quote: 'I have no signal.',
			body: 'No connection means no editor, even if all you wanted was to fix one line.',
			tone: 'muted',
			icon: IconWifiOff,
			tag: 'offline'
		}
	];

	// Tone -> classes for the notification rows.
	const toneClass: Record<Tone, { chip: string; bar: string }> = {
		red: { chip: 'bg-destructive/10 text-destructive', bar: 'bg-destructive' },
		amber: { chip: 'bg-warning/10 text-warning', bar: 'bg-warning' },
		muted: { chip: 'bg-muted text-muted-foreground', bar: 'bg-muted-foreground/40' }
	};

	// What stays local vs. what you reach through your own accounts.
	const local = [
		{
			icon: IconCpu,
			title: 'The compiler',
			body: 'The LaTeX engine runs on your computer. Nothing is queued on our servers.'
		},
		{
			icon: IconFolder,
			title: 'Your files',
			body: 'Projects are folders on your disk. Opening one reads a directory, saving writes a file.'
		},
		{
			icon: IconGitBranch,
			title: 'Your history',
			body: 'Commits live in your own Git repository, on your machine and on the remote you pick.'
		}
	] as const;

	const connected = [
		{
			icon: IconKey,
			title: 'Your AI key',
			body: 'Connect a key from a provider you trust. Requests go from the app straight to them.',
			tag: 'Planned'
		},
		{
			icon: IconCloud,
			title: 'Your cloud storage',
			body: 'Sync through Dropbox or Google Drive, on the account you already pay for.',
			tag: 'Planned'
		},
		{
			icon: IconShare,
			title: 'Sharing',
			body: 'Hand a project to a collaborator, stored only while shared and only under your name.',
			tag: 'Planned'
		}
	] as const;

	type Cell = boolean | string;
	type Row = { label: string; glyph: Cell; overleaf: Cell; desktop: Cell };
	const comparison: Row[] = [
		{ label: 'Real LaTeX, journal-ready output', glyph: true, overleaf: true, desktop: true },
		{ label: 'Runs fully offline', glyph: true, overleaf: false, desktop: true },
		{ label: 'Nothing uploaded to a server', glyph: true, overleaf: false, desktop: true },
		{ label: 'No compile timeout', glyph: true, overleaf: 'Free limit', desktop: true },
		{ label: 'Git built in', glyph: true, overleaf: 'Paid', desktop: 'Bring your own' },
		{
			label: 'Version history without paying',
			glyph: 'Built in',
			overleaf: 'Paid',
			desktop: 'Your VCS'
		},
		{ label: 'AI help with your own key', glyph: 'Planned', overleaf: 'Paid', desktop: false },
		{
			label: 'Sync through your own cloud',
			glyph: 'Planned',
			overleaf: 'Dropbox, paid',
			desktop: 'Your cloud'
		},
		{ label: 'Free, no account', glyph: true, overleaf: 'Limited', desktop: true }
	];

	const steps = [
		{
			n: '01',
			title: 'Open a folder, or start a blank one',
			body: 'Point GlyphX at an existing project directory or create a new document. There is no upload step and no size to worry about.'
		},
		{
			n: '02',
			title: 'Write LaTeX with the page beside you',
			body: 'Type real LaTeX and watch the document update next to it. Math, figures, and citations render as you go.'
		},
		{
			n: '03',
			title: 'Export the PDF your journal expects',
			body: 'Compile to a clean PDF that matches your template or thesis spec. The file lands in your folder, ready to submit.'
		}
	];

	const faqs = [
		{
			q: 'Is this real LaTeX or a watered-down version?',
			a: 'Real LaTeX. Full math, environments, figures, and BibTeX, with the packages a journal template or thesis class needs. What you write is standard .tex that any LaTeX setup can read.'
		},
		{
			q: 'Do I have to install a TeX distribution?',
			a: 'No. The desktop app ships with the engine built in, and the browser editor compiles in the page. There is no multi gigabyte download and no package manager to fight.'
		},
		{
			q: 'Where do my files live?',
			a: 'On your disk. A project is a normal folder of .tex and .bib files. Nothing is copied to a server, so backups, syncing, and Git are entirely your call.'
		},
		{
			q: 'Can I bring my Overleaf projects over?',
			a: 'Yes. Overleaf projects are plain LaTeX underneath. Download the project folder, drop it into GlyphX, and keep writing.'
		},
		{
			q: 'Does GlyphX have version control?',
			a: 'Yes. The desktop app has a built in Git client: stage and commit, read a side by side diff, browse history, clone a repository, and push, pull, or sync with your own remote. No subscription tier in the way.'
		},
		{
			q: 'Does it use AI, and where would my data go?',
			a: 'AI help is on the roadmap and will be opt in with your own key. You would connect a provider such as OpenAI or another one you trust, and requests go from the app straight to them on your account. We are not in the path, and there is no shared model trained on your writing.'
		},
		{
			q: 'What does it cost?',
			a: 'The browser editor is free with no account. The desktop app is free as well. The source is on GitHub if you want to read it or build it yourself.'
		},
		{
			q: 'Does it really work offline?',
			a: 'On the desktop, yes. The editor and the engine both run on your machine, so a flaky connection or no connection at all does not stop you. The browser editor needs to be online the first time it fetches a package.'
		}
	];
</script>

<svelte:head>
	<title>GlyphX: the LaTeX editor Overleaf should have been</title>
	<meta
		name="description"
		content="GlyphX is a local-first LaTeX editor for researchers, PhD students, and mathematicians. Write papers and theses in real LaTeX, compile on your own machine, use built-in Git, and keep your drafts off other people's servers. Free in your browser, plus a native desktop app."
	/>
</svelte:head>

<div class="bg-canvas text-foreground min-h-dvh">
	<SiteHeader />

	<!-- ============================================================
	     Hero: centered headline + a full-width product window beneath
	     ============================================================ -->
	<section class="relative overflow-hidden">
		<FloatingGlyphs />
		<!-- soft brand glow behind the headline -->
		<div
			class="pointer-events-none absolute top-[-12%] left-1/2 -z-0 h-[460px] w-[820px] max-w-[92vw] -translate-x-1/2 rounded-full opacity-60 blur-[130px]"
			style="background: radial-gradient(closest-side, var(--brand-subtle), transparent);"
			aria-hidden="true"
		></div>

		<div class="relative mx-auto max-w-[1140px] px-5 sm:px-6">
			<div class="flex flex-col items-center pt-24 text-center sm:pt-32">
				<h1
					class="font-display max-w-4xl text-[2.6rem] leading-[1.04] tracking-[-0.035em] sm:text-[4.25rem]"
				>
					<SplitReveal mode="lines" class="block">
						The LaTeX editor Overleaf should have been.
					</SplitReveal>
				</h1>
				<Reveal variant="up" delay={250} class="flex flex-col items-center">
					<p class="text-muted-foreground mt-7 max-w-[40rem] text-lg leading-relaxed">
						Write papers, proofs, and theses in real LaTeX. The desktop app runs the editor, the
						compiler, your files, and Git on your own computer. Sync and AI connect through accounts
						you already own. Nothing has to pass through our servers.
					</p>
					<div class="mt-9 flex flex-wrap items-center justify-center gap-3">
						<a
							href={resolve('/download')}
							onclick={() => trackEvent('cta_download_click', { location: 'hero' })}
							class="bg-primary text-primary-foreground group inline-flex h-12 items-center gap-2 rounded-lg px-6 text-sm font-semibold transition-all hover:opacity-90 active:scale-[0.98]"
						>
							<IconDownload class="size-4" /> Get the desktop app
						</a>
						<a
							href={resolve('/editor')}
							class="border-hairline bg-card text-foreground hover:bg-muted group inline-flex h-12 items-center gap-2 rounded-lg border px-6 text-sm font-semibold transition-colors"
						>
							<IconPlayerPlay class="size-4" /> Try it in the browser
							<IconArrowRight class="size-4 transition-transform group-hover:translate-x-0.5" />
						</a>
					</div>
					<p class="text-muted-foreground/80 mt-4 font-mono text-xs">
						free · no account · the desktop app keeps it all on your machine
					</p>
				</Reveal>
			</div>

			<!-- Full-width product window -->
			<Reveal variant="morph" delay={140} class="mt-14 sm:mt-16">
				<div
					class="border-hairline bg-card shadow-craft-floating overflow-hidden rounded-2xl border"
				>
					<div class="border-hairline flex h-11 items-center gap-2 border-b px-4">
						<span class="bg-muted-foreground/30 size-2.5 rounded-full"></span>
						<span class="bg-muted-foreground/30 size-2.5 rounded-full"></span>
						<span class="bg-muted-foreground/30 size-2.5 rounded-full"></span>
						<span class="text-muted-foreground ml-3 font-mono text-xs">thesis.tex</span>
						<span
							class="text-brand border-brand/30 bg-brand-subtle ml-auto rounded-full border px-2 py-0.5 font-mono text-[10px] font-semibold tracking-wider uppercase"
							>LaTeX</span
						>
					</div>
					<div class="grid grid-cols-1 lg:grid-cols-2">
						<pre
							class="text-muted-foreground border-hairline overflow-hidden border-b p-5 font-mono text-[12px] leading-[1.85] sm:p-6 lg:border-r lg:border-b-0"><span
								class="text-brand">\documentclass</span
							>&#123;article&#125;
<span class="text-brand">\usepackage</span>&#123;amsmath&#125;

<span class="text-brand">\title</span>&#123;On Local-First Typesetting&#125;
<span class="text-brand">\author</span>&#123;A. Researcher&#125;

<span class="text-brand">\begin</span>&#123;document&#125;
<span class="text-brand">\maketitle</span>

We observe that the estimator <span class="text-foreground">$\hat&#123;\theta&#125;$</span>
is consistent, with <span class="text-foreground">$\alpha$</span> scaling as <span
								class="text-foreground">$\beta^2$</span
							>:

<span class="text-brand">\begin</span>&#123;equation&#125;
  E = m c^2
<span class="text-brand">\end</span>&#123;equation&#125;

See <span class="text-brand">\cite</span>&#123;einstein1905&#125;.<span
								class="text-brand inline-block h-[1em] w-[2px] translate-y-[2px] align-middle [animation:blink_1.05s_steps(1)_infinite]"
							></span>
<span class="text-brand">\end</span>&#123;document&#125;</pre>
						<div class="bg-canvas/50 p-7 sm:p-9">
							<div
								class="text-muted-foreground/60 mb-4 font-mono text-[11px] tracking-widest uppercase"
							>
								Live preview
							</div>
							<h3 class="font-display mb-1 text-2xl">On Local-First Typesetting</h3>
							<p class="text-muted-foreground mb-5 text-sm">A. Researcher</p>
							<p class="text-muted-foreground leading-relaxed">
								We observe that the estimator θ̂ is consistent, with α scaling as β²:
							</p>
							<div class="text-foreground my-4 text-center text-2xl italic">E = mc²</div>
							<p class="text-muted-foreground/70 text-sm">See [1].</p>
						</div>
					</div>
					<div
						class="border-hairline text-muted-foreground flex h-9 items-center gap-4 border-t px-4 font-mono text-[11px]"
					>
						<span class="text-brand flex items-center gap-1.5">
							<span class="bg-brand size-1.5 rounded-full"></span> compiled
						</span>
						<span>on your device</span>
						<span class="text-muted-foreground/50 ml-auto tabular-nums">offline · Ln 14</span>
					</div>
				</div>
			</Reveal>
		</div>
	</section>

	<!-- Trust strip -->
	<Reveal as="section" variant="up" class="mt-16 sm:mt-20">
		<div class="border-hairline mx-auto max-w-[1140px] border-y px-5 sm:px-6">
			<div
				class="text-muted-foreground grid grid-cols-2 gap-y-4 py-6 font-mono text-xs sm:grid-cols-4"
			>
				<span class="flex items-center gap-2"
					><span class="bg-brand size-1.5 rounded-full"></span> Nothing uploaded</span
				>
				<span class="flex items-center gap-2"
					><span class="bg-brand size-1.5 rounded-full"></span> Compiles locally</span
				>
				<span class="flex items-center gap-2"
					><span class="bg-brand size-1.5 rounded-full"></span> No account</span
				>
				<span class="flex items-center gap-2"
					><span class="bg-brand size-1.5 rounded-full"></span> Free and open source</span
				>
			</div>
		</div>
	</Reveal>

	<!-- ============================================================
	     The problem
	     ============================================================ -->
	<section class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
		<div class="grid items-start gap-12 lg:grid-cols-[0.85fr_1.15fr]">
			<Reveal variant="up" class="lg:sticky lg:top-28">
				<span
					class="text-muted-foreground font-mono text-[11px] font-semibold tracking-[0.2em] uppercase"
				>
					Sound familiar
				</span>
				<h2 class="font-display mt-4 text-3xl tracking-tight sm:text-[2.6rem] sm:leading-[1.08]">
					<SplitReveal mode="lines" triggerOnScroll class="block">
						If you have lived in Overleaf, you know these.
					</SplitReveal>
				</h2>
				<p class="text-muted-foreground mt-4 text-lg leading-relaxed">
					Overleaf put LaTeX one click away, and that got a lot of people writing. Then the browser
					started getting in the way. The same handful of messages kept coming back.
				</p>
				<p class="text-foreground mt-6 text-base leading-relaxed font-medium">
					None of these are LaTeX problems. They are cloud problems. GlyphX has no cloud, so it has
					none of them.
				</p>
			</Reveal>

			<!-- Faux notifications panel: the actual messages a cloud editor shows you -->
			<Reveal variant="up" delay={80}>
				<div class="border-hairline bg-card shadow-craft-lg overflow-hidden rounded-2xl border">
					<div class="border-hairline flex items-center gap-2 border-b px-4 py-3">
						<span class="bg-destructive/60 size-2.5 rounded-full"></span>
						<span class="bg-warning/60 size-2.5 rounded-full"></span>
						<span class="bg-muted-foreground/40 size-2.5 rounded-full"></span>
						<span class="text-muted-foreground ml-2 font-mono text-xs">overleaf.com</span>
						<span class="text-muted-foreground/60 ml-auto font-mono text-[11px]">
							{painPoints.length} notifications
						</span>
					</div>

					{#each painPoints as p, i (p.quote)}
						{@const Icon = p.icon}
						<Reveal
							as="div"
							variant="up"
							delay={i * 45}
							class="border-hairline hover:bg-muted/40 flex items-stretch gap-4 border-b px-4 py-4 transition-colors last:border-b-0 sm:px-5"
						>
							<span class="w-[3px] shrink-0 rounded-full {toneClass[p.tone].bar}"></span>
							<span
								class="grid size-9 shrink-0 place-items-center rounded-lg {toneClass[p.tone].chip}"
							>
								<Icon class="size-[18px]" />
							</span>
							<div class="min-w-0 flex-1">
								<div class="flex items-center gap-3">
									<p class="text-foreground text-[15px] font-semibold">{p.quote}</p>
									<span
										class="border-hairline text-muted-foreground/70 ml-auto hidden shrink-0 rounded-full border px-2 py-0.5 font-mono text-[10px] tracking-wider uppercase sm:block"
									>
										{p.tag}
									</span>
								</div>
								<p class="text-muted-foreground mt-1 text-sm leading-relaxed">{p.body}</p>
							</div>
						</Reveal>
					{/each}
				</div>
			</Reveal>
		</div>
	</section>

	<!-- ============================================================
	     Capabilities: a scroll-driven card stack (Motion Core).
	     Each card sticks and scales as the next slides over it.
	     ============================================================ -->
	<section id="features" class="border-hairline border-t">
		<div class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
			<Reveal variant="up" class="max-w-2xl">
				<span
					class="text-muted-foreground font-mono text-[11px] font-semibold tracking-[0.2em] uppercase"
				>
					What you get
				</span>
				<h2 class="font-display mt-4 text-3xl tracking-tight sm:text-[2.6rem] sm:leading-[1.08]">
					<SplitReveal mode="lines" triggerOnScroll class="block">
						Built for the people who write the LaTeX, not the people who run the servers.
					</SplitReveal>
				</h2>
				<p class="text-muted-foreground mt-4 text-lg leading-relaxed">
					Five things GlyphX does today. Scroll through them.
				</p>
			</Reveal>

			<div class="mx-auto mt-14 max-w-[1000px]">
				<CardStack topOffset={108} offset={16} scaleFactor={0.035}>
					{#each capabilities as f, i (f.title)}
						{@const Icon = f.icon}
						<CardStackItem
							class="border-hairline bg-card shadow-craft-lg mb-6 grid min-h-[420px] gap-8 overflow-hidden rounded-3xl border p-8 sm:min-h-[440px] sm:p-12 lg:grid-cols-[1.05fr_0.95fr] lg:items-center"
						>
							<div>
								<div class="flex items-center gap-3">
									<span
										class="border-hairline bg-canvas text-brand grid size-11 place-items-center rounded-xl border"
									>
										<Icon class="size-5" />
									</span>
									<span class="text-muted-foreground/70 font-mono text-xs tracking-widest">
										{String(i + 1).padStart(2, '0')} / {String(capabilities.length).padStart(
											2,
											'0'
										)}
									</span>
								</div>
								<h3 class="font-display mt-6 text-2xl tracking-tight sm:text-3xl">{f.title}</h3>
								<p class="text-muted-foreground mt-3 max-w-md text-base leading-relaxed">
									{f.body}
								</p>
							</div>

							<!-- Per-card visual -->
							<div
								class="border-hairline bg-canvas/60 relative hidden overflow-hidden rounded-2xl border p-6 lg:block"
							>
								{#if f.visual === 'code'}
									<pre class="text-muted-foreground font-mono text-[12px] leading-[1.8]"><span
											class="text-brand">\section</span
										>&#123;Results&#125;

We estimate <span class="text-foreground">$\hat&#123;\theta&#125;$</span> with

<span class="text-brand">\begin</span>&#123;equation&#125;
  E = m c^2
<span class="text-brand">\end</span>&#123;equation&#125;</pre>
								{:else if f.visual === 'compile'}
									<div class="flex flex-col gap-3 font-mono text-xs">
										<div class="text-muted-foreground flex items-center gap-2">
											<IconBolt class="text-brand size-4" /> tectonic · local engine
										</div>
										<div class="bg-brand/10 h-1.5 w-full overflow-hidden rounded-full">
											<div class="bg-brand h-full w-2/3 rounded-full"></div>
										</div>
										<div class="text-brand flex items-center gap-2">
											<IconCheck class="size-4" /> compiled in 0.41s
										</div>
										<div class="text-muted-foreground/70">no queue · no server · offline</div>
									</div>
								{:else if f.visual === 'split'}
									<div class="grid grid-cols-2 gap-3 text-xs">
										<pre
											class="text-muted-foreground border-hairline rounded-lg border p-3 font-mono leading-relaxed"><span
												class="text-brand">\hat</span
											>&#123;\theta&#125; \to \theta</pre>
										<div class="border-hairline grid place-items-center rounded-lg border p-3">
											<span class="text-foreground text-lg italic">θ̂ → θ</span>
										</div>
									</div>
								{:else if f.visual === 'git'}
									<div class="flex flex-col gap-2 font-mono text-xs">
										<div class="text-muted-foreground flex items-center gap-2">
											<IconGitBranch class="text-brand size-4" /> main · 2 changes
										</div>
										<div class="text-muted-foreground flex items-center gap-2">
											<span class="text-warning">M</span> chapter3.tex
										</div>
										<div class="text-muted-foreground flex items-center gap-2">
											<span class="text-success">+</span> figures/plot.pdf
										</div>
										<div
											class="bg-primary text-primary-foreground mt-1 w-fit rounded-md px-3 py-1 text-[11px] font-semibold"
										>
											Commit
										</div>
									</div>
								{:else}
									<div class="flex flex-col gap-3">
										<div class="text-foreground flex items-center gap-2 text-sm font-medium">
											<IconLock class="text-brand size-4" /> Nothing uploaded
										</div>
										<div class="text-foreground flex items-center gap-2 text-sm font-medium">
											<IconWifiOff class="text-brand size-4" /> Works offline
										</div>
										<div class="text-foreground flex items-center gap-2 text-sm font-medium">
											<IconFolder class="text-brand size-4" /> Your folder, your disk
										</div>
										<p class="text-muted-foreground/70 mt-1 text-xs leading-relaxed">
											It sits on your machine and nowhere else.
										</p>
									</div>
								{/if}
							</div>
						</CardStackItem>
					{/each}
				</CardStack>
			</div>
		</div>
	</section>

	<!-- ============================================================
	     Where your data goes: local core vs. your accounts
	     ============================================================ -->
	<section class="border-hairline border-t">
		<div class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
			<Reveal variant="up" class="max-w-2xl">
				<span
					class="text-muted-foreground inline-flex items-center gap-2 font-mono text-[11px] font-semibold tracking-[0.18em] uppercase"
				>
					Where your data goes
				</span>
				<h2 class="font-display mt-5 text-3xl tracking-tight sm:text-4xl">
					Local-first does not mean cut off from the cloud.
				</h2>
				<p class="text-muted-foreground mt-4 text-lg leading-relaxed">
					The private, heavy work stays on your machine. When you want sync, history, or a hand from
					a model, you reach the cloud through accounts you already have. The compiler is never a
					server, and the keys are always yours.
				</p>
			</Reveal>

			<div class="mt-12 grid gap-4 lg:grid-cols-2">
				<!-- Stays local -->
				<Reveal variant="up" class="border-hairline bg-card flex flex-col rounded-2xl border p-7">
					<div class="mb-6 flex items-center gap-2">
						<span class="bg-brand size-1.5 rounded-full"></span>
						<span
							class="text-foreground font-mono text-[11px] font-semibold tracking-[0.16em] uppercase"
							>On your machine</span
						>
					</div>
					<div class="flex flex-col gap-5">
						{#each local as m (m.title)}
							{@const Icon = m.icon}
							<div class="flex items-start gap-3">
								<span
									class="border-hairline bg-canvas text-brand grid size-9 shrink-0 place-items-center rounded-lg border"
								>
									<Icon class="size-[18px]" />
								</span>
								<div>
									<h3 class="text-sm font-semibold">{m.title}</h3>
									<p class="text-muted-foreground mt-0.5 text-sm leading-relaxed">{m.body}</p>
								</div>
							</div>
						{/each}
					</div>
				</Reveal>

				<!-- Through your accounts -->
				<Reveal
					variant="up"
					delay={90}
					class="border-hairline bg-card/60 flex flex-col rounded-2xl border p-7"
				>
					<div class="mb-6 flex items-center gap-2">
						<span class="bg-muted-foreground/40 size-1.5 rounded-full"></span>
						<span
							class="text-muted-foreground font-mono text-[11px] font-semibold tracking-[0.16em] uppercase"
							>Through your accounts</span
						>
					</div>
					<div class="flex flex-col gap-5">
						{#each connected as m (m.title)}
							{@const Icon = m.icon}
							<div class="flex items-start gap-3">
								<span
									class="border-hairline bg-canvas text-muted-foreground grid size-9 shrink-0 place-items-center rounded-lg border"
								>
									<Icon class="size-[18px]" />
								</span>
								<div class="flex-1">
									<div class="flex items-center gap-2">
										<h3 class="text-sm font-semibold">{m.title}</h3>
										<span
											class="border-hairline text-muted-foreground rounded-full border px-2 py-0.5 font-mono text-[9px] font-semibold tracking-wider uppercase"
											>{m.tag}</span
										>
									</div>
									<p class="text-muted-foreground mt-0.5 text-sm leading-relaxed">{m.body}</p>
								</div>
							</div>
						{/each}
					</div>
				</Reveal>
			</div>

			<Reveal variant="up" delay={120}>
				<div
					class="border-hairline bg-brand-subtle mt-4 flex items-start gap-3 rounded-2xl border p-6"
				>
					<IconLock class="text-brand mt-0.5 size-5 shrink-0" />
					<p class="text-foreground/90 text-base leading-relaxed">
						We only ever hold what you choose to share, tied to your personal account. Click share
						and a project goes out under your name. Never click it, and your work never leaves your
						machine.
					</p>
				</div>
			</Reveal>
		</div>
	</section>

	<!-- ============================================================
	     Comparison
	     ============================================================ -->
	<section id="compare" class="border-hairline border-t">
		<div class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
			<Reveal variant="up">
				<span
					class="text-muted-foreground inline-flex items-center gap-2 font-mono text-[11px] font-semibold tracking-[0.18em] uppercase"
				>
					Compare
				</span>
				<h2 class="font-display mt-5 max-w-2xl text-3xl tracking-tight sm:text-4xl">
					Private like your laptop. Easy like the cloud was supposed to be.
				</h2>
			</Reveal>

			<Reveal variant="up" delay={80} class="mt-10">
				<div class="border-hairline overflow-x-auto rounded-2xl border">
					<div class="grid min-w-[640px] grid-cols-[1.8fr_1fr_1fr_1fr] text-sm">
						<div
							class="text-muted-foreground bg-card px-5 py-4 font-mono text-[11px] tracking-widest uppercase"
						>
							&nbsp;
						</div>
						<div class="bg-card text-foreground px-3 py-4 text-center font-semibold">GlyphX</div>
						<div class="bg-card text-muted-foreground px-3 py-4 text-center font-medium">
							Overleaf free
						</div>
						<div class="bg-card text-muted-foreground px-3 py-4 text-center font-medium">
							Desktop&nbsp;TeX
						</div>

						{#each comparison as row (row.label)}
							<div class="border-hairline text-foreground/90 border-t px-5 py-4">{row.label}</div>
							<div class="border-hairline bg-brand-subtle border-t px-3 py-4 text-center">
								{#if row.glyph === true}
									<IconCheck class="text-brand mx-auto size-5" />
								{:else}
									<span class="text-brand font-mono text-xs font-semibold">{row.glyph}</span>
								{/if}
							</div>
							<div class="border-hairline text-muted-foreground border-t px-3 py-4 text-center">
								{#if row.overleaf === true}
									<IconCheck class="mx-auto size-5 opacity-70" />
								{:else if row.overleaf}
									<span class="font-mono text-xs">{row.overleaf}</span>
								{:else}
									<IconMinus class="text-muted-foreground/40 mx-auto size-4" />
								{/if}
							</div>
							<div class="border-hairline text-muted-foreground border-t px-3 py-4 text-center">
								{#if row.desktop === true}
									<IconCheck class="mx-auto size-5 opacity-70" />
								{:else if row.desktop}
									<span class="font-mono text-xs">{row.desktop}</span>
								{:else}
									<IconMinus class="text-muted-foreground/40 mx-auto size-4" />
								{/if}
							</div>
						{/each}
					</div>
				</div>
				<p class="text-muted-foreground/70 mt-3 font-mono text-[11px]">
					Overleaf paid tiers lift some of these limits. The point is that GlyphX does not put them
					there to begin with.
				</p>
			</Reveal>
		</div>
	</section>

	<!-- ============================================================
	     How it works
	     ============================================================ -->
	<section class="border-hairline border-t">
		<div class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
			<Reveal variant="up">
				<span
					class="text-muted-foreground inline-flex items-center gap-2 font-mono text-[11px] font-semibold tracking-[0.18em] uppercase"
				>
					How it works
				</span>
				<h2 class="font-display mt-5 max-w-2xl text-3xl tracking-tight sm:text-4xl">
					Three steps, none of which involve a login.
				</h2>
			</Reveal>

			<div
				class="bg-hairline border-hairline mt-12 grid gap-px overflow-hidden rounded-2xl border sm:grid-cols-3"
			>
				{#each steps as s, i (s.n)}
					<Reveal as="article" variant="up" delay={i * 80} class="bg-card flex flex-col p-7">
						<span class="text-brand font-mono text-sm font-semibold">{s.n}</span>
						<h3 class="mt-4 text-base font-semibold">{s.title}</h3>
						<p class="text-muted-foreground mt-2 text-sm leading-relaxed">{s.body}</p>
					</Reveal>
				{/each}
			</div>
		</div>
	</section>

	<!-- ============================================================
	     Roadmap note (honest about what is not built yet)
	     ============================================================ -->
	<section class="border-hairline border-t">
		<div class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
			<Reveal
				variant="up"
				class="border-hairline bg-card flex flex-col gap-4 rounded-2xl border p-8 sm:flex-row sm:items-start sm:gap-6 sm:p-10"
			>
				<span
					class="border-hairline bg-canvas text-foreground grid size-11 shrink-0 place-items-center rounded-xl border"
				>
					<IconSparkles class="size-5" />
				</span>
				<div class="max-w-2xl">
					<h2 class="font-display text-2xl tracking-tight sm:text-3xl">Still to come.</h2>
					<p class="text-muted-foreground mt-3 text-base leading-relaxed">
						A bring-your-own AI key, Dropbox and Google Drive sync, and project sharing are the next
						things we are building. They are not in the app yet, and we would rather say so than
						pretend otherwise. The rule behind all of them stays the same: your providers, your
						keys, your data.
					</p>
					<div class="mt-5 flex flex-wrap gap-2">
						{#each [{ icon: IconKey, label: 'Your AI key' }, { icon: IconBrandDropbox, label: 'Cloud sync' }, { icon: IconShare, label: 'Sharing' }] as r (r.label)}
							{@const Icon = r.icon}
							<span
								class="border-hairline text-muted-foreground inline-flex items-center gap-1.5 rounded-full border px-3 py-1 font-mono text-[11px] font-medium"
							>
								<Icon class="size-3.5" />
								{r.label}
								<span class="text-muted-foreground/50">· planned</span>
							</span>
						{/each}
					</div>
				</div>
			</Reveal>
		</div>
	</section>

	<!-- ============================================================
	     FAQ
	     ============================================================ -->
	<section id="faq" class="border-hairline border-t">
		<div class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-28">
			<div class="grid gap-12 lg:grid-cols-[0.8fr_1.2fr]">
				<Reveal variant="up">
					<span
						class="text-muted-foreground inline-flex items-center gap-2 font-mono text-[11px] font-semibold tracking-[0.18em] uppercase"
					>
						FAQ
					</span>
					<h2 class="font-display mt-5 max-w-sm text-3xl tracking-tight sm:text-4xl">
						The questions people ask first.
					</h2>
				</Reveal>

				<div class="flex flex-col">
					{#each faqs as f, i (f.q)}
						<Reveal
							as="div"
							variant="up"
							delay={i * 40}
							class="border-hairline border-t py-6 first:border-t-0 first:pt-0"
						>
							<h3 class="text-foreground text-base font-semibold">{f.q}</h3>
							<p class="text-muted-foreground mt-2 text-sm leading-relaxed">{f.a}</p>
						</Reveal>
					{/each}
				</div>
			</div>
		</div>
	</section>

	<!-- ============================================================
	     CTA band
	     ============================================================ -->
	<section class="mx-auto max-w-[1140px] px-5 py-20 sm:px-6 sm:py-24">
		<Reveal variant="scale">
			<div
				class="bg-primary text-primary-foreground relative overflow-hidden rounded-3xl px-8 py-16 text-center sm:px-16 sm:py-20"
			>
				<div
					class="pointer-events-none absolute inset-0 opacity-10"
					style="background-image: radial-gradient(circle at 1px 1px, currentColor 1px, transparent 0); background-size: 22px 22px;"
					aria-hidden="true"
				></div>
				<div class="relative mx-auto flex max-w-xl flex-col items-center">
					<h2 class="font-display text-3xl tracking-tight sm:text-[2.75rem] sm:leading-[1.05]">
						Keep your research on your own machine.
					</h2>
					<p class="text-primary-foreground/65 mt-4 max-w-md text-base leading-relaxed">
						Start in the browser for free, or get the desktop app and work fully offline. No
						account, no upload, no waiting on a server.
					</p>
					<div class="mt-9 flex flex-wrap justify-center gap-3">
						<a
							href={resolve('/download')}
							onclick={() => trackEvent('cta_download_click', { location: 'footer_cta' })}
							class="bg-card text-foreground group inline-flex h-12 items-center gap-2 rounded-lg px-6 text-sm font-semibold transition-all hover:opacity-90 active:scale-[0.98]"
						>
							<IconDownload class="size-4" /> Download GlyphX
						</a>
						<a
							href={resolve('/editor')}
							class="border-primary-foreground/25 text-primary-foreground hover:bg-primary-foreground/10 inline-flex h-12 items-center gap-2 rounded-lg border px-6 text-sm font-semibold transition-colors"
						>
							Open the editor <IconArrowRight class="size-4" />
						</a>
					</div>
				</div>
			</div>
		</Reveal>
	</section>

	<SiteFooter />
</div>

<style>
	@keyframes blink {
		50% {
			opacity: 0;
		}
	}
</style>
