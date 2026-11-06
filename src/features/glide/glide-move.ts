// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

// How Glide carries a window, for pictures of it at work: the native tween's
// length and curve (`src-tauri/src/glide/core/tween_motion.rs`), kept in
// step with it, so a window in a picture moves as a real one does.

/** How long a move takes, end to end, in seconds. */
export const GLIDE_MOVE_SECONDS = 0.22;

/** How far a move is `fraction` of the way through its time: ease-out
 * cubic, leaving fast and arriving softly. */
export function glideMoveEased(fraction: number) {
  const remaining = 1 - Math.min(Math.max(fraction, 0), 1);
  return 1 - remaining * remaining * remaining;
}
