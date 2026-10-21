// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";

import { ANNOTATIONS, CAPTIONS, SCENES, SHORTCUTS, span } from "./editor-data";
import { Band, Lane, Sound, TimelineFrame } from "./timeline-parts";
import { Filmstrip } from "./timeline-stills";

// Three rows instead of eight. Everything placed over the recording shares
// one row in bands: the scene in force along the top, annotations and
// shortcuts in the middle, the caption being spoken along the bottom. Then
// the picture, the camera's stills over the screen's, and the sound with
// both voices. Every timeline edit acts on screen, camera and sound
// together, so no track needs a row of its own; whether a track is in the
// export is set from the Tracks menu beside the timeline.
//
// A clip is a grey part; the selected one is filled in the primary colour,
// as everything chosen is. A shortcut is its own keycaps.

export function Timeline({ hasPicture }: { hasPicture: boolean }) {
  return (
    <TimelineFrame>
      {hasPicture ? (
        <>
          <Lane className="h-12" label="Clips">
            <Band className="top-0 h-3">
              {SCENES.map(({ end, label, start }) => (
                <span
                  className="absolute inset-y-0 truncate border-s-2 border-fg-tertiary ps-1 text-[10px] leading-3 text-fg-tertiary"
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
            </Band>
            <Band className="top-3.5 h-part-height">
              {ANNOTATIONS.map(({ end, isSelected, label, start }) => (
                <span
                  className={cn(
                    "absolute inset-y-0 flex items-center truncate rounded-sm px-1.5 text-caption",
                    isSelected
                      ? "bg-primary text-on-primary"
                      : "bg-fill-strong text-fg",
                  )}
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
              {SHORTCUTS.map(({ end, keys, start }) => (
                <span
                  className="absolute inset-y-0 flex justify-center"
                  key={start}
                  style={span(start, end)}
                >
                  <Shortcut>
                    {keys.map((key) => (
                      <Keyboard key={key}>{key}</Keyboard>
                    ))}
                  </Shortcut>
                </span>
              ))}
            </Band>
            <Band className="bottom-0 h-3">
              {CAPTIONS.map(({ end, label, start }) => (
                <span
                  className="absolute inset-y-0 truncate rounded-sm bg-fill px-1 text-[10px] leading-3 text-fg-tertiary"
                  key={start}
                  style={span(start, end)}
                >
                  {label}
                </span>
              ))}
            </Band>
          </Lane>
          <Lane className="h-12" label="Picture">
            {/* The screen is what the recording is about, so it takes two
                thirds; the camera's third is enough to know it is there. */}
            <Band className="top-0 h-3.5">
              <Filmstrip source="camera" />
            </Band>
            <Band className="bottom-0 h-8">
              <Filmstrip source="screen" />
            </Band>
          </Lane>
        </>
      ) : null}
      <Lane className="h-8" label="Sound">
        <Band className="top-0 h-1/2">
          <Sound className="bg-fill text-fg-tertiary" seed={1} />
        </Band>
        <Band className="bottom-0 h-1/2">
          <Sound className="bg-fill text-fg-secondary" seed={4} />
        </Band>
      </Lane>
    </TimelineFrame>
  );
}
