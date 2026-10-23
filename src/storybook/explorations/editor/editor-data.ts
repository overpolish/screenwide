// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ACTIONS,
  type Action,
  RECORDING_S,
  SEGMENTS,
} from "../ideas/ideas-data";

// The editor edits the same 48-second walkthrough as the ideas. Its backbone
// is what was done, not what was said: the recording's cursor and key files
// already give every click, drag, typing run, shortcut and app switch
// (`src-tauri/src/editor/auto_zoom/signals.rs`), so a recording with no
// voice has as much to edit by as one with. Speech, where there is any, is
// a layer over that.

export { ACTIONS, RECORDING_S };

/** A stretch of the recording to edit by: something done, from the pointer
 * setting off to the work settling, or for a recording with only sound, a
 * spoken line. Between beats is still time, which a squeeze gives up. */
export type Beat =
  | { action: Action; end: number; id: string; kind: "action"; start: number }
  | { end: number; id: string; kind: "speech"; start: number; text: string };

/** How long before each action the pointer sets off, and after it the work
 * settles, in seconds. */
const LEAD: Record<Action["kind"], [number, number]> = {
  click: [1.4, 0.5],
  idle: [0, 0],
  keys: [0.4, 0.5],
  switch: [0.4, 0.6],
  type: [0.6, 1.2],
};

export const ACTION_BEATS: Beat[] = ACTIONS.filter(
  ({ kind }) => kind !== "idle",
).map((action) => ({
  action,
  end: action.at + LEAD[action.kind][1],
  id: `a${String(action.at)}`,
  kind: "action",
  start: action.at - LEAD[action.kind][0],
}));

export const SPEECH = SEGMENTS.flatMap((segment) =>
  segment.kind === "speech" ? [segment] : [],
);

export const SPEECH_BEATS: Beat[] = SPEECH.map((segment) => ({
  end: segment.end,
  id: `s${String(segment.start)}`,
  kind: "speech",
  start: segment.start,
  text: segment.text,
}));

/** Every word with the moment it starts, spread over its sentence. */
export const WORDS = SPEECH.flatMap((segment) => {
  const words = segment.text.split(" ");
  const step = (segment.end - segment.start) / words.length;
  return words.map((word, index) => ({
    at: segment.start + index * step,
    word,
  }));
});

/** Where the work moved on: at an app switch or a long still stretch, not
 * at a sentence. `element` is what the step is about, outlined on its
 * still. */
export type Step = {
  end: number;
  start: number;
  title: string;
  /** A shortcut is done on no element, so its step outlines none. */
  element?: string;
};

export const STEPS: Step[] = [
  { element: "new", end: 11.5, start: 0, title: "Start a project" },
  { element: "name", end: 17.5, start: 11.5, title: "Name it" },
  { element: "create", end: 30, start: 17.5, title: "Pick and create" },
  { end: 36, start: 30, title: "Save" },
  { element: "share", end: RECORDING_S, start: 36, title: "Share it" },
];

export type Layout = "follow" | "pip" | "screen" | "side";

export const LAYOUTS: { id: Layout; label: string }[] = [
  { id: "screen", label: "Screen" },
  { id: "follow", label: "Follow pointer" },
  { id: "side", label: "Side by side" },
  { id: "pip", label: "Picture in picture" },
];

export const DEFAULT_SCENES: Layout[] = [
  "screen",
  "follow",
  "follow",
  "screen",
  "pip",
];

/** When each seeded mark (`look-pens-model.ts`) shows: from the action on
 * its element to the end of that action's step. */
export const SEEDED_TIMES: Record<number, { end: number; start: number }> = {
  1: { end: 30, start: 16.2 },
  2: { end: 11.5, start: 8.9 },
  3: { end: RECORDING_S, start: 43.1 },
};

export type Backdrop = "dark" | "none" | "warm";

export const BACKDROPS: { background: string; id: Backdrop; label: string }[] =
  [
    { background: "transparent", id: "none", label: "None" },
    {
      background: "linear-gradient(135deg, #f6d365, #fda085)",
      id: "warm",
      label: "Warm",
    },
    {
      background: "linear-gradient(135deg, #2b2d42, #111)",
      id: "dark",
      label: "Dark",
    },
  ];

export const backdropOf = (id: Backdrop) =>
  BACKDROPS.find((item) => item.id === id)?.background ?? "transparent";

export const stepAt = (seconds: number) =>
  Math.max(
    0,
    STEPS.findIndex(({ end, start }) => seconds >= start && seconds < end),
  );
