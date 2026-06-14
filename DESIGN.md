## Identity

GlyphX is a local-first, privacy-first **LaTeX** editor for researchers,
students, engineers, mathematicians, and technical writers.

The visual language is **editorial monochrome**: near-white paper in light
mode, graphite-black surfaces in dark mode, ink as the primary voice, and a
single rationed **GlyphX blue** accent derived from the brand mark.

Monospace (Geist Mono) carries the technical register — file names, metrics,
engine badges, diagnostics, and status indicators.

Directionally, GlyphX sits closer to the disciplined end of modern product
design (Linear, Vercel, Raycast, VS Code, OpenAI) than colourful AI products
or consumer SaaS dashboards.

The product should feel:

* Technical
* Precise
* Quiet
* Fast
* Trustworthy

The editor is the product.

The chrome should disappear.

---

## Principles

### 1. Document first

The document is always the focal point.

Whitespace is preferable to decoration.

Every UI decision should make writing easier, not louder.

### 2. Greyscale first, brand second

Design every screen in monochrome first.

Introduce blue only where meaning exists.

Target roughly:

* 90% neutral surfaces
* 10% brand color

The blue should feel intentional, not decorative.

### 3. Brand color is reserved

GlyphX blue communicates:

* Active
* Selected
* Focused
* Linked
* Branded

Avoid flooding entire screens with blue.

The logo should be the most saturated object in the product.

### 4. Type creates hierarchy

Hierarchy comes from:

* Size
* Weight
* Spacing
* Contrast

Not color.

Typography should carry most of the visual structure.

### 5. Motion settles, never performs

One shared easing:

```css
cubic-bezier(.625,.05,0,1)
```

Elements should:

* Fade
* Rise
* Blur
* Settle

Never bounce.

Never feel playful.

GlyphX is a precision tool.

---

## Colour tokens

Semantic tokens drive the component library and editor chrome.

Blue is the brand accent.

Neutral surfaces remain dominant.

| Token                | Light                  | Dark                     | Role                  |
| -------------------- | ---------------------- | ------------------------ | --------------------- |
| `--background`       | `#FAFBFC`              | `#0A0A0C`                | Page floor            |
| `--canvas`           | `#FAFBFC`              | `#0A0A0C`                | Root surface          |
| `--card`             | `#FFFFFF`              | `#141418`                | Raised surface        |
| `--surface`          | `#F8FAFC`              | `#1C1C22`                | Elevated surface      |
| `--foreground`       | `#18181B`              | `#EBEBEB`                | Primary text          |
| `--muted-foreground` | `#6B6B75`              | `#9A9AA3`                | Secondary text        |
| `--primary`          | `#0A0A0C`              | `#F4F4F5`                | Ink emphasis          |
| `--accent`           | `#ECE9E2`              | `#1F1F25`                | Neutral hover surface |
| `--border`           | `rgb(9 9 11 / .08)`    | `rgb(255 255 255 / .08)` | Hairline borders      |
| `--brand`            | `#007ACC`              | `#1F9CF0`                | GlyphX blue           |
| `--brand-hover`      | `#1177BB`              | `#38B6FF`                | Interactive hover     |
| `--brand-subtle`     | `rgb(0 122 204 / .08)` | `rgb(31 156 240 / .12)`  | Soft brand wash       |
| `--success`          | `#10B981`              | `#34D399`                | Success state         |
| `--warning`          | `#F59E0B`              | `#FBBF24`                | Warning state         |
| `--destructive`      | `#EF4444`              | `#F87171`                | Error state           |

---

## Brand colours

### Blue 400

```css
#1F9CF0
```

Gradient highlight.

### Blue 500

```css
#007ACC
```

Primary brand color.

### Blue 600

```css
#1177BB
```

Gradient depth and hover state.

### Graphite 950

```css
#0A0A0C
```

Primary dark-mode foundation.

### Graphite 900

```css
#141418
```

Raised surfaces.

### Graphite 800

```css
#1C1C22
```

Elevated surfaces.

---

## Brand gradient

Official GlyphX gradient:

```css
linear-gradient(
  135deg,
  #1F9CF0 0%,
  #007ACC 55%,
  #1177BB 100%
)
```

Used only for:

* Logo
* App icon
* Marketing graphics
* Hero illustrations

Avoid gradients in application UI.

---

## Dark mode philosophy

Dark mode should feel like graphite, not navy.

The interface should resemble:

* Paper and ink in light mode
* Graphite and ink in dark mode

Blue appears only in:

* Active states
* Links
* Selections
* Focus indicators
* Brand surfaces

The application itself should remain neutral.
