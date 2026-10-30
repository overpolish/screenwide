// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** How a part of the strip is drawn while it is lifted to be moved, in
 * front of the rest. It is carried along by `--lift-x`, which the drag sets
 * on the row as the pointer goes, at once, so it stays under the pointer;
 * raising it off the row eases in, the strip's glide (`--glide`) the only
 * transition it keeps. Let go, the strip's glide takes it from the pointer
 * to its place. */
export const LIFTED =
  "z-30 cursor-grabbing [transform:translateX(var(--lift-x,0px))] [translate:0_-6px] transition-[translate]! duration-(--glide)! ease-out!";
