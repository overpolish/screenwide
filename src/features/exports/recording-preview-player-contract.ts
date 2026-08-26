// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingPreviewLayout } from "./types";

export type RecordingPreviewPlayerEvent =
  | { event: "ended" }
  | { data: { positionMs: number }; event: "rangeEnded" }
  | { data: { message: string }; event: "error" }
  | {
      data: { positionMs: number };
      event: "paused" | "playing" | "position";
    }
  | { data: { positionMs: number; requestId: number }; event: "ready" };

export type RecordingPreviewPlayerInfo = {
  durationMs: number;
  framesPerSecond: number | null;
  layout: RecordingPreviewLayout;
};
