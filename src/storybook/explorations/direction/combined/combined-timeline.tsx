// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";

import {
  ANNOTATIONS,
  CAPTIONS,
  SCENES,
  SHORTCUTS,
  span,
} from "./combined-data";
import { Band, Lane, Sound, TimelineFrame } from "./timeline-parts";
import { Filmstrip } from "./timeline-stills";

// Three rows instead of eight. Everything placed over the recording shares
// one row in bands: the scene in force along the top, annotations and
// shortcuts in the middle, the caption being spoken along the bottom. Then
// the picture, the camera's stills over the screen's, and the sound with
// both voices. Every timeline edit acts on screen, camera and sound
// together, so no track needs a row of its own; whether a track is in the
// export is set from the Tracks menu beside the timeline.

export function Timeline({ hasPicture }: { hasPicture: boolean }) {
  return (
    <TimelineFrame>
      {hasPicture ? (
        <>
          <Lane className="h-[46px]" label="Clips">
            <Band className="top-0 h-3">
              {SCENES.map(({ end, label, start }) => (
                <span
                  className="absolute inset-y-0 truncate border-s-2 border-(--clip-scene) ps-1 text-[10px] leading-3 text-(--fg-3)"
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
            </Band>
            <Band className="top-3.5 h-[18px]">
              {ANNOTATIONS.map(({ end, isSelected, label, start }) => (
                <span
                  className={cn(
                    "absolute inset-y-0 flex items-center truncate rounded-[5px] px-1.5 text-[11px]",
                    isSelected ? "combined-accent" : "bg-(--clip-annotation)",
                  )}
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
              {SHORTCUTS.map(({ end, label, start }) => (
                <span
                  className="combined-raised absolute inset-y-0.5 flex items-center justify-center rounded-[4px] text-[10px] font-medium text-(--fg-2)"
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
            </Band>
            <Band className="bottom-0 h-3">
              {CAPTIONS.map(({ end, label, start }) => (
                <span
                  className="absolute inset-y-0 truncate rounded-[3px] bg-(--fill) px-1 text-[10px] leading-3 text-(--fg-3)"
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
            </Band>
          </Lane>
          <Lane className="h-[46px]" label="Picture">
            {/* The screen is what the recording is about, so it takes two
                thirds; the camera's third is enough to know it is there. */}
            <Band className="top-0 h-[14px]">
              <Filmstrip source="camera" />
            </Band>
            <Band className="bottom-0 h-[30px]">
              <Filmstrip source="screen" />
            </Band>
          </Lane>
        </>
      ) : null}
      <Lane className="h-8" label="Sound">
        <Band className="top-0 h-1/2">
          <Sound className="bg-(--fill) text-(--fg-3)" seed={1} />
        </Band>
        <Band className="bottom-0 h-1/2">
          <Sound className="bg-(--fill) text-(--level)" seed={4} />
        </Band>
      </Lane>
    </TimelineFrame>
  );
}
