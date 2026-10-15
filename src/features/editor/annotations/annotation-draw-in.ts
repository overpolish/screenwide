// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * How long an animated annotation takes to arrive where its clip names no
 * pace, in source milliseconds. The twins of `REVEAL_DRAW_IN_MS`,
 * `COUNTER_REVEAL_IN_MS`, `TEXT_REVEAL_IN_MS`, `SPOTLIGHT_FADE_IN_MS` and
 * `MAGNIFY_REVEAL_IN_MS` in `src-tauri/crates/annotations/src/reveal.rs`,
 * `counter/reveal.rs`, `text/reveal.rs`, `spotlight/reveal.rs` and
 * `magnify/reveal.rs`, which Rust tests hold to these lines. A clip the
 * editor paces arrives over its path's own time instead, as
 * `annotation-pace.ts` works it out. A text box's is its box's arrival and
 * then its pointer's, and a spotlight's is its shade deepening.
 */
export const ANNOTATION_DRAW_IN_MS = 1000;
export const ANNOTATION_COUNTER_DRAW_IN_MS = 320;
export const ANNOTATION_TEXT_DRAW_IN_MS = 600;
export const ANNOTATION_SPOTLIGHT_DRAW_IN_MS = 400;
export const ANNOTATION_MAGNIFY_DRAW_IN_MS = 450;
