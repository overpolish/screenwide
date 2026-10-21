// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The project the mocks show, so the look is judged on the same content.

export const PROJECT_NAME = "Onboarding walkthrough";

/** The annotation palette: content colours, the same in every direction. */
export const ANNOTATION_COLORS = [
  "#ff453a",
  "#ff9f0a",
  "#ffd60a",
  "#30d158",
  "#0a84ff",
  "#bf5af2",
  "#ffffff",
  "#1c1c1e",
];

export const ARROW_COLOR = ANNOTATION_COLORS[0];

export type ArrowStyle = {
  color: string;
  name: string;
  width: number;
  isHandDrawn?: boolean;
};

export const PRESETS: ArrowStyle[] = [
  { color: "#ff453a", name: "Callout", width: 4 },
  { color: "#ffffff", name: "Subtle", width: 2 },
  { color: "#ffd60a", isHandDrawn: true, name: "Marker", width: 5 },
  { color: "#0a84ff", name: "Pointer", width: 3 },
];

/** A deterministic waveform: `count` levels between 0 and 1. */
export const waveform = (count: number, seed: number): number[] =>
  Array.from({ length: count }, (_, index) => {
    const value =
      Math.abs(Math.sin(index * 0.37 + seed)) * 0.6 +
      Math.abs(Math.sin(index * 0.11 + seed * 2)) * 0.4;
    return Math.max(0.08, Math.min(1, value));
  });

/** The selected arrow's bounds on the recording, in percent of the picture. */
export const SELECTION_BOX = { height: 30, left: 38, top: 40, width: 24 };

/** The selected arrow's ends, in percent of `SELECTION_BOX`, where its
 * handles sit: dragging an end is how an arrow is reshaped. */
export const ARROW_ENDS = [
  { x: 7, y: 90 },
  { x: 90, y: 14 },
];
