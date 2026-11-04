// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TimelinePlayhead } from "../playhead/timeline-playhead";
import { type Playhead } from "../scrub-playhead";
import { type ScrubPhase } from "../timeline-seek";

// The band along the strip's top that the playhead's knob rides in, above
// the scenes so it is never taken for part of one. It takes in the room
// between the picture and the strip as well, which the strip's band draws
// into its own top (`recording-strip-band.tsx`), so a press anywhere above
// the scenes moves the playhead, as does dragging the knob; only the range
// tool draws there instead (`useRangeCut`). It is left unfilled, so the
// strip's top stays quiet.

export function StripScrubBand({
  onScrub,
  playhead,
}: {
  /** Told each pointer event of a drag on the knob. */
  onScrub: (event: React.PointerEvent, phase: ScrubPhase) => void;
  playhead: Playhead;
}) {
  return (
    <>
      <span
        aria-hidden
        className="absolute inset-x-0 top-0 h-8"
        data-scrub-band
      />
      {/* Over the scenes, which its line crosses, but pressed only by its
        knob, centred in the knob's room under the picture's. */}
      <span className="pointer-events-none absolute inset-x-0 top-5.5 bottom-0 z-20">
        <TimelinePlayhead onScrub={onScrub} playhead={playhead} />
      </span>
    </>
  );
}
