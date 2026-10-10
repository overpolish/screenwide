// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

type PreviewStill = {
  goTo: (positionMs: number) => void;
  /** Where the preview last stood still, in the recording's milliseconds. */
  positionMs: number;
  setPositionMs: (positionMs: number) => void;
  /** Where a tool outside the preview asks it to go, until it has gone. */
  wanted: { positionMs: number } | null;
};

/**
 * Where the recording preview stands while it is not playing, for tools
 * outside the preview that act on the frame on screen, such as picking the
 * camera's objects, and where such a tool asks it to go.
 */
export const usePreviewStill = create<PreviewStill>()((set) => ({
  goTo: (positionMs) => {
    set({ wanted: { positionMs } });
  },
  positionMs: 0,
  setPositionMs: (positionMs) => {
    set({ positionMs });
  },
  wanted: null,
}));
