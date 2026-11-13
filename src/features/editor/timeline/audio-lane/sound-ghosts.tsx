// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLines, Music } from "lucide-react";

import { type TimelineScale } from "../timeline-scale";

import { type TimelineAudioClip } from "./audio-bar";

// Where sounds carried over a lane would land if let go now: each a dashed
// outline over the time it would play, with its glyph and title, drawn over
// the lane's clips and taking no presses. A sound not yet read has no
// length, so it shows as a stub where it would start.

/** How wide, in px, a sound not yet read is drawn, and a short one at least. */
const STUB_PX = 24;

export function SoundGhosts({
  ghosts,
  scale,
  width,
}: {
  ghosts: TimelineAudioClip[];
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
}) {
  return ghosts.map((ghost) => {
    const start = scale.toX(ghost.start) * width;
    const end = scale.toX(ghost.end, true) * width;
    const Glyph = ghost.kind === "music" ? Music : AudioLines;
    return (
      <span
        aria-hidden
        className="pointer-events-none absolute inset-y-0 z-20 flex items-center gap-1 overflow-hidden rounded-part border border-dashed border-fg-secondary bg-fill px-1 text-caption whitespace-nowrap text-fg [&_svg]:size-icon-sm [&_svg]:shrink-0"
        dir="ltr"
        key={ghost.id}
        style={{
          insetInlineStart: start,
          width: Math.max(end - start, STUB_PX),
        }}
      >
        <Glyph />
        <span className="truncate" dir="auto">
          {ghost.title}
        </span>
      </span>
    );
  });
}
