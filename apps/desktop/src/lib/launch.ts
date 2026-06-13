/**
 * Holds the folder / `.tex` / `.glyx` path the OS asked GlyphX to open (via a
 * file association or "Open with GlyphX"). The root layout sets it and routes to
 * the folder-mode editor, which reads it once on mount. A plain mutable object
 * (not a rune) — it's read imperatively at mount, not tracked reactively.
 */
export const launch: { path: string | null } = { path: null };
