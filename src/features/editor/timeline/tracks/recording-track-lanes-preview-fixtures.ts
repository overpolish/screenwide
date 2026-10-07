// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingAnnotationClip } from "../../recording/annotations/recording-annotations";
import { RecordingSceneClip } from "../../recording/scenes/recording-scenes";
import {
  RecordingKeyboardTimelineItem,
  RecordingPreviewLayout,
  RecordingTimelineThumbnails,
} from "../../types";
import { RecordingMoment } from "../moments/recording-moments";

/** What the timeline preview stories lay onto their lanes. */
export const STORY_DURATION_MS = 120_000;
export const STORY_FRAMES_PER_SECOND = 60_000 / 1_001;

export const STORY_KEYBOARD_ITEMS: RecordingKeyboardTimelineItem[] = [
  { endMs: 11_800, id: 0, label: "⌘ C", startMs: 10_000 },
  { endMs: 29_600, id: 1, label: "⌘ V", startMs: 28_000 },
  { endMs: 74_900, id: 2, label: "⇧ ⌘ 4", startMs: 72_000 },
];

const counter = (
  value: number,
  startMs: number,
  endMs: number,
): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id: `counter-${value.toString()}`,
    shape: { angle: 0, center: { x: 200, y: 200 }, kind: "counter", value },
    style: {
      align: "left",
      blur: false,
      color: "#ffcc00",
      handDrawn: false,
      head: "none",
      manual: false,
      radius: 0,
      redaction: "erase",
      shadow: false,
      softness: 0,
      strength: 0,
      tint: false,
      width: 56,
    },
  },
  endMs,
  startMs,
  trackId: "primary",
});

/** Two counters near enough to the badges and to each other to snap. */
export const STORY_ANNOTATION_CLIPS = [
  counter(1, 14_000, 26_000),
  counter(2, 40_000, 58_000),
];

/** A zoom and a side by side butted together, then a picture in picture on
 * its own later in the recording. */
export const STORY_SCENE_CLIPS: RecordingSceneClip[] = [
  {
    endMs: 30_000,
    id: "scene-1",
    preset: "full",
    screen: { focusX: 0.3, focusY: 0.4, zoom: 2 },
    startMs: 20_000,
  },
  {
    endMs: 38_000,
    id: "scene-2",
    preset: "split-two-thirds",
    startMs: 30_000,
  },
  {
    endMs: 96_000,
    id: "scene-3",
    preset: "picture-in-picture",
    startMs: 80_000,
  },
];

export const STORY_LAYOUT: RecordingPreviewLayout = {
  height: 1080,
  panes: [
    {
      height: 1080,
      kind: "screen",
      sourceHeight: 1080,
      sourceWidth: 1920,
      width: 1920,
      x: 0,
      y: 0,
    },
  ],
  width: 1920,
};

export const STORY_THUMBNAILS: RecordingTimelineThumbnails = {
  camera: [],
  primary: Array.from({ length: 24 }, (_, index) => ({
    id: `primary-${index.toString()}`,
    url: null,
  })),
};

/** A moment alone, one with a voice note, and another alone. */
export const STORY_MOMENTS: RecordingMoment[] = [
  {
    color: "#ffcc00",
    index: 0,
    kindId: "funny",
    name: "Funny",
    note: null,
    sourceMs: 18_000,
  },
  {
    color: "#0088ff",
    index: 1,
    kindId: "notable",
    name: "Notable",
    note: {
      durationMs: 4_200,
      waveform: Array.from(
        { length: 160 },
        (_, point) => Math.abs(Math.sin(point / 7)) * 0.8,
      ),
    },
    sourceMs: 47_500,
  },
  {
    color: "#ffcc00",
    index: 2,
    kindId: "funny",
    name: "Funny",
    note: null,
    sourceMs: 91_000,
  },
];
