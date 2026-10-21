// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A recording with everything the editor shows in its timeline today: scenes,
// shortcuts, captions, a cut, a sped-up stretch, moments with a voice note,
// the camera with its background removed, two audio tracks and annotations.

export const DURATION_S = 94;
export const PLAYHEAD_S = 31;

/** The timeline's column of row names, in pixels: wide enough for the
 * timeline's three zoom buttons at the head of the ruler. */
export const TIMELINE_LABEL_WIDTH = 60;

/** Where `seconds` falls along the recording, in percent. */
export const at = (seconds: number) =>
  `${String((seconds / DURATION_S) * 100)}%`;

/** A span placed by start and end in seconds. Lanes are `dir="ltr"`: time
 * runs left to right in every language. */
export const span = (start: number, end: number) => ({
  insetInlineStart: at(start),
  width: at(end - start),
});

export const clock = (seconds: number) =>
  `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;

export type Timed = { end: number; label: string; start: number };

export const SCENES: Timed[] = [
  { end: 20, label: "Screen only", start: 0 },
  { end: 46, label: "Zoom", start: 20 },
  { end: 70, label: "Side by side", start: 46 },
  { end: 94, label: "Picture in picture", start: 70 },
];

/** Shortcuts pressed while recording, each as the keys shown. */
export const SHORTCUTS: (Omit<Timed, "label"> & { keys: string[] })[] = [
  { end: 21, keys: ["Command", "C"], start: 17 },
  { end: 44, keys: ["Command", "V"], start: 40 },
  { end: 70, keys: ["Shift", "Command", "4"], start: 64 },
];

export const CAPTIONS: Timed[] = [
  { end: 14, label: "Welcome to the dashboard", start: 1 },
  { end: 30, label: "Click New project", start: 17 },
  { end: 49, label: "Name it and pick a template", start: 33 },
  { end: 70, label: "Invite your team", start: 54 },
  { end: 90, label: "And you are ready to go", start: 74 },
];

/** Where seconds were cut out. The timeline closes up behind a cut, leaving
 * a sliver so the join shows; the button over it on the ruler restores it. */
export const CUT = { at: 38, seconds: 4 };

/** Half the sliver between the footage either side of a cut, in seconds. */
const JOIN = 0.18;

/** The footage either side of the cut. */
export const STRETCHES = [
  [0, CUT.at - JOIN],
  [CUT.at + JOIN, DURATION_S],
] as const;

/** Played faster; the ruler shows the rate with handles at its ends. */
export const SPEED = { end: 66, rate: "2×", start: 52 };

export type Moment = {
  at: number;
  color: string;
  name: string;
  note?: { duration: string; transcript: string };
};

export const MOMENTS: Moment[] = [
  { at: 18, color: "#ffcc00", name: "Funny" },
  {
    at: 47.5,
    color: "#0088ff",
    name: "Notable",
    note: {
      duration: "0:04",
      transcript:
        "Cut the intro here, the demo really starts once the settings window opens.",
    },
  },
  { at: 91, color: "#ffcc00", name: "Funny" },
];

export const ANNOTATIONS: (Timed & { isSelected?: boolean })[] = [
  { end: 16, label: "Counter 1", start: 6 },
  { end: 36, isSelected: true, label: "Arrow 1", start: 26 },
  { end: 60, label: "Text 1", start: 51 },
  { end: 84, label: "Redaction 1", start: 73 },
];
