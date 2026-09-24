// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingAnnotationClip } from "../recording-annotations";
import {
  RecordingKeyboardTimelineItem,
  RecordingPreviewLayout,
  RecordingTimelineThumbnails,
} from "../types";

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
    aboveCamera: false,
    animated: true,
    id: `counter-${value.toString()}`,
    shape: { angle: 0, center: { x: 200, y: 200 }, kind: "counter", value },
    style: { align: "left", color: "#ffcc00", head: "none", width: 56 },
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
