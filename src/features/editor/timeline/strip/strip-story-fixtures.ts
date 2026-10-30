// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ANNOTATION_SWATCHES } from "../../../../composites/color-grid/palette";
import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineBeat } from "../beat-clip/beat-label";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { storyCaption } from "../caption-lane/caption-story-fixtures";
import { type TimelineCut } from "../cut-seams/cut-seams";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { storyScene } from "../scene-rail/scene-story-fixtures";
import { type Span } from "../timeline-scale";
import { storyFrame, storyPeaks } from "../timeline-story-fixtures";

import { type StripBeat } from "./strip-beat-row";

import type { LullPlay } from "../../island/lull/lull-island";

// A recording behind the strip's stories, made up the same way every time
// for the same length: beats of every kind with lulls between, some long
// enough to be dead air and some where the picture moves, moments in their
// kinds' colours, a voice with its captions, a few scenes and annotations.

export type StoryLull = {
  id: string;
  play: LullPlay;
  span: Span;
  moving?: Span[];
};

export type StoryRecording = {
  annotations: TimelineAnnotation[];
  beats: StripBeat[];
  /** The voice's lines, or null with no voice. */
  captions: TimelineCaption[] | null;
  cuts: TimelineCut[];
  /** How long it was recorded, in seconds. */
  duration: number;
  lulls: StoryLull[];
  scenes: TimelineScene[];
};

/** The same run of numbers from 0 to 1 for the same seed. */
function numbers(seed: number) {
  let state = seed;
  return () => {
    state = (state + 0x6d2b79f5) | 0;
    let value = Math.imul(state ^ (state >>> 15), 1 | state);
    value ^= value + Math.imul(value ^ (value >>> 7), 61 | value);
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
  };
}

/** How long, in seconds, the recording waits before its first beat. */
const LEAD_S = 4;

const LINES = [
  "Let's set up a project from a template.",
  "This is where the settings live.",
  "Now we invite the rest of the team.",
  "And that's the whole flow, start to finish.",
];

const DONE: TimelineBeat[] = [
  { kind: "click", label: "New Project" },
  { kind: "type", label: "Onboarding" },
  { keys: ["Command", "S"], kind: "keys" },
  { app: "Safari", kind: "switch" },
  { kind: "screen" },
  { kind: "click", label: "Invite" },
  { kind: "sound" },
];

const MOMENTS = [
  { color: ANNOTATION_SWATCHES[1].color, name: "Notable" },
  { color: ANNOTATION_SWATCHES[5].color, name: "Bug" },
  { color: ANNOTATION_SWATCHES[3].color, name: "Win" },
];

/** The beat at `index`: every seventh a moment, every third said aloud
 * where there is a voice, the rest done on screen. */
function beatAt(index: number, isVoiced: boolean): TimelineBeat {
  if (index % 7 === 5) {
    return { kind: "moment", ...MOMENTS[((index / 7) % MOMENTS.length) | 0] };
  }
  if (isVoiced && index % 3 === 1) {
    return { kind: "speech", line: LINES[((index / 3) % LINES.length) | 0] };
  }
  return DONE[index % DONE.length];
}

/** A recording about `duration` seconds long, ending with its last beat.
 * Without a voice it has no speech and no captions; still loading, its
 * pictures are not read yet; crowded, its annotations stack four deep;
 * cut, two of its beats lose their middle. */
export function storyRecording({
  duration,
  isCrowded = false,
  isCut = false,
  isLoading = false,
  isVoiced = true,
}: {
  duration: number;
  isCrowded?: boolean;
  isCut?: boolean;
  isLoading?: boolean;
  isVoiced?: boolean;
}): StoryRecording {
  const next = numbers(7);
  const beats: StripBeat[] = [];
  // A wait after pressing record, before anything happens, as most
  // recordings open with: a lull like any other, to squeeze, trim or cut.
  let at = LEAD_S;
  const lulls: StoryLull[] = [
    { id: "lull-lead", play: { kind: "squeeze" }, span: { end: at, start: 0 } },
  ];
  for (let index = 0; at < duration - 2; index += 1) {
    const beat = beatAt(index, isVoiced);
    const length = beat.kind === "speech" ? 4 + next() * 4 : 1.5 + next() * 5;
    const span = { end: Math.min(at + length, duration - 1), start: at };
    beats.push({
      beat,
      id: `beat-${String(index)}`,
      peaks:
        beat.kind === "speech" || beat.kind === "sound"
          ? storyPeaks(index + 1)
          : undefined,
      span,
      thumbnail: isLoading ? null : storyFrame(index),
    });
    // Mostly short breaths, every fourth a long wait, so even a short
    // recording has a few beats before the first.
    const gap = index % 4 === 2 ? 6 + next() * 10 : 0.5 + next() * 3;
    const lull = {
      end: Math.min(span.end + gap, duration - 1),
      start: span.end,
    };
    if (lull.end - lull.start > 0.3 && lull.end < duration - 2) {
      const length = lull.end - lull.start;
      lulls.push({
        id: `lull-${String(index)}`,
        // Every other long wait is a page loading part of the way.
        moving:
          length > 6 && index % 8 === 2
            ? [
                {
                  end: lull.start + length * 0.6,
                  start: lull.start + length * 0.3,
                },
              ]
            : undefined,
        play: { kind: "squeeze" },
        span: lull,
      });
    }
    at = lull.end;
  }
  // It ends with its last beat, so no stretch is left that no lull holds.
  const recorded = beats[beats.length - 1].span.end;
  const share = (part: number) => part * recorded;
  const annotations: TimelineAnnotation[] = (
    [
      ["arrow", 0.08, 0.16, 0],
      ["highlight", 0.2, 0.26, 0],
      ["counter", 0.12, 0.22, 1],
      ["text", 0.38, 0.52, 0],
      ["focus", 0.48, 0.58, 1],
      ["redact", 0.7, 0.8, 0],
      ...(isCrowded
        ? ([
            ["spotlight", 0.1, 0.3, 2],
            ["magnify", 0.14, 0.2, 3],
            ["shape", 0.4, 0.5, 2],
          ] as const)
        : []),
    ] as const
  ).map(([kind, start, end, lane]) => ({
    end: share(end),
    id: kind,
    kind,
    lane,
    start: share(start),
  }));
  return {
    annotations,
    beats,
    captions: isVoiced
      ? beats.flatMap(({ beat, span }) =>
          beat.kind === "speech"
            ? [storyCaption(span.start, span.end, beat.line)]
            : [],
        )
      : null,
    cuts: isCut
      ? [beats[2], beats[6]].map(({ id, span }) => ({
          end: span.start + (span.end - span.start) * 0.7,
          id: `cut-${id}`,
          start: span.start + (span.end - span.start) * 0.3,
        }))
      : [],
    duration: recorded,
    lulls,
    scenes: [
      storyScene("intro", "camera-only", { end: share(0.06), start: 0 }),
      storyScene("demo", "split-two-thirds", {
        end: share(0.42),
        start: share(0.14),
      }),
      storyScene("detail", "picture-in-picture", {
        end: share(0.72),
        start: share(0.6),
      }),
    ],
  };
}
