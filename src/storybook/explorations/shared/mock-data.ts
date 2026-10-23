// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The project the mocks show, so the look is judged on the same content.

export const PROJECT_NAME = "Onboarding walkthrough";

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
