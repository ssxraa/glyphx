# GlyphX — Design System

> A calm, near-monochrome instrument for writing LaTeX. The document is the
> hero; the interface gets out of the way. Restraint is the premium signal.
>
> Source of truth for tokens: [`packages/ui/src/app.css`](packages/ui/src/app.css).
> This file is the human-readable spec; the CSS is what ships.

## Identity

GlyphX is a local-first, privacy-first **LaTeX** editor (Tectonic on-device).
The visual language is **editorial monochrome**: clean near-white in light, a
blue-near-black night in dark, ink as the primary voice, and a single rationed
**emerald** accent for live/active signals. Monospace (Geist Mono) carries the
"technical voice" — labels, code, metrics — which is honest for a typesetting
tool. Direction reference: the disciplined, near-accentless end of modern
product design (keyfold.io, OpenAI, Vercel) — not colourful AI marketing.

## Principles

1. **Greyscale first, accent last.** Design in ink-on-paper; add emerald only
   where it *means* something (compiled, live, active, a link). Target ≤ ~10%
   of any surface carrying colour.
2. **`--primary` is ink, not brand.** Near-black drives CTAs and emphasis. The
   emerald lives in a separate `--brand` token so it never leaks into every
   button-hover (which uses the neutral shadcn `--accent`).
3. **Hairlines are translucent.** Borders are translucent ink (`rgb(9 9 11 /
   .08)`) / translucent white in dark — finer than solid greys, and they adapt
   to any surface.
4. **Motion settles, never slides cheaply.** One shared easing
   (`--ease-craft`, `cubic-bezier(.625,.05,0,1)`); reveals fade + rise / blur /
   scale / morph into place. Always `motion-reduce` safe.
5. **Type does the hierarchy.** Geist (display + UI) with tight negative
   tracking on display; Geist Mono for the technical register. No third family.

## Colour tokens

Semantic tokens drive the whole component library + editor chrome and stay
monochrome. `--brand` is the only chromatic accent.

| Token | Light | Dark | Role |
|---|---|---|---|
| `--background` / `--canvas` | `#fafafa` | `#0a0a0c` | Page floor (near-white / blue-near-black). |
| `--card` | `#ffffff` | `#141418` | Raised surface. |
| `--foreground` | `#18181b` | `#ebebeb` | Default text (ink). |
| `--muted-foreground` | `#6b6b75` | `#9a9aa3` | Secondary text (cool grey). |
| `--primary` | `#0a0a0c` | `#f4f4f5` | Ink CTA / emphasis. **Not** the brand colour. |
| `--accent` | `#ece9e2` | `#1f1f25` | Neutral hover/surface (shadcn) — never brand. |
| `--border` / `--hairline` | `rgb(9 9 11 / .08)` | `rgb(255 255 255 / .08)` | Translucent hairline. |
| `--brand` | `#0d9373` | `#34d399` | The single emerald accent (live/active/links). |
| `--brand-subtle` | `rgb(13 147 115 / .10)` | `rgb(52 211 153 / .12)` | Faint brand wash. |
| `--success` | `#0d9373` | `#34d399` | Status (shares the emerald family). |
| `--destructive` | `#d92d20` | `#f0564b` | Error — deliberately distinct from brand. |

Tailwind utilities follow the token names: `bg-canvas`, `text-brand`,
`bg-brand-subtle`, `border-hairline`, `text-ink-muted`, etc.

## Typography

Self-hosted variable fonts (`@fontsource-variable/*`) — bundled, fully offline.

- **Display + UI — Geist** (`--font-sans` / `--font-display`). Display: weight
  600, letter-spacing `-0.025em` (tighter at large sizes). Body: 400/500.
- **Technical voice — Geist Mono** (`--font-mono`). Eyebrows (uppercase,
  tracked ~`0.16em`), file names, engine badges, status bars, metrics. Pair
  with `tabular-nums` wherever numbers carry meaning.
- **Editor surface — JetBrains Mono** is retained for the CodeMirror code pane.

## Motion

Driven by the `Reveal` component (`@glyphx/ui/reveal`) — an IntersectionObserver
toggling Tailwind classes through `--ease-craft`. Variants: `up` / `down` /
`left` / `right` (directional fade), `blur` (focus-pull), `scale`, and `morph`
(blur + scale + slide — the signature "settle into place"). Stagger lists with
`delay={i * 60}`. Hero artifacts use `morph` / `blur`; sections use `up`.
Prefer Svelte's native `transition:`/`in:`/`out:` + Tailwind; reach for a motion
library only when something genuinely needs orchestration (rare).

## Shape & elevation

- `--radius` = `0.625rem` (10px) base; scale derives `sm/md/lg/xl`. Cards
  ~`rounded-2xl` (16px), CTA band `rounded-3xl`. Crisp, never pill-everything.
- `craft-*` shadows are neutral and low-alpha; depth comes from light +
  translucent borders, not heavy drop-shadows.

## Do / Don't

**Do**
- Keep colour rationed to `--brand` on live/active/link moments.
- Use ink (`--primary`) for the primary CTA; mono for the technical register.
- Let `Reveal` carry entrance motion; respect `motion-reduce`.

**Don't**
- Don't repaint the neutral shadcn `--accent` with brand colour.
- Don't use solid grey hairlines — use the translucent `--hairline`.
- Don't add a third type family or a second accent. One ink, one emerald.
