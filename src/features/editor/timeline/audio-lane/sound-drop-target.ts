// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// Where on the sound lanes a point of the page is, for a drag from the
// system that the page follows only as points (`use-sound-drag.ts`): the
// lanes say, as only they know where they are and how the strip is scaled.

/** A point on a sound lane: the kind its lane makes a sound, and the moment
 * of the edited video under it, in ms. */
export type SoundDropSpot = { atMs: number; kind: AudioClipKind };

type Target = (x: number, y: number) => SoundDropSpot | null;

let target: Target | null = null;

/** Makes `found` what answers where a point is, while the lanes are shown;
 * returns what lets it go. */
export function setSoundDropTarget(found: Target) {
  target = found;
  return () => {
    if (target === found) target = null;
  };
}

/** The spot on the lanes at a point of the page, in CSS px, or none off
 * them. */
export const soundDropSpot = (x: number, y: number) => target?.(x, y) ?? null;
