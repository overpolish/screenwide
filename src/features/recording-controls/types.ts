// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingSnapshot } from "../../bindings/RecordingSnapshot";
import type { RecordingStatus } from "../../bindings/RecordingStatus";
import type { ReplaySnapshot } from "../../bindings/ReplaySnapshot";
import type { StartRecordingOptions } from "../../bindings/StartRecordingOptions";

export type {
  RecordingSnapshot,
  RecordingStatus,
  ReplaySnapshot,
  StartRecordingOptions,
};

export const initialRecordingSnapshot: RecordingSnapshot = {
  accumulatedMs: 0,
  countdownSecondsRemaining: 0,
  mode: null,
  pausedAtMs: null,
  startedAtMs: null,
  status: "idle",
};

/** What the screenshot button is currently reflecting. */
export type ScreenshotState = "done" | "failed" | "idle" | "pending";
export type ScreenshotAction = "clipboard" | "editor" | "scrolling";

type RecordingErrorPhase = "start" | "pause" | "resume" | "stop";

export type RecordingError = {
  message: string;
  phase: RecordingErrorPhase;
};

export const initialReplaySnapshot: ReplaySnapshot = {
  available: false,
  lengthSeconds: 30,
  saving: false,
  status: "off",
};
