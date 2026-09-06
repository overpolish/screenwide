// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { defaultScreenshotOutput } from "../screenshot-output";
import { EditorArtifact } from "../types";

/** Shared story captures, so the panel and the export dialog show the same ones. */
export const screenshot: EditorArtifact = {
  extension: "png",
  height: 2234,
  id: 1,
  items: [{ height: 2234, id: 2, width: 3456 }],
  kind: "screenshot",
  suggestedFileStem: "Screenwide 2026-08-08 at 14.32.05",
  width: 3456,
};
export const screenshotOutput = defaultScreenshotOutput(3456, 2234);

export const recording: Extract<EditorArtifact, { kind: "recording" }> = {
  audioTracks: [
    { kind: "system-audio", label: "System audio", streamIndex: 0 },
    { kind: "microphone", label: "Microphone", streamIndex: 1 },
  ],
  camera: null,
  canCompress: true,
  cursorDataVersion: 1,
  durationMs: 3_845_000,
  extension: "mp4",
  hasCursorData: true,
  hasKeyboardData: true,
  height: 2160,
  id: 2,
  keyboardDataVersion: 1,
  kind: "recording",
  originalSizeBytes: 186_400_000,
  // The working file is a QuickTime movie; `extension` is what saving it
  // delivers, which is not the same thing.
  path: "/tmp/Recordings/recording-20260808-143205.000.mov",
  primaryKind: "screen",
  sourceScalePercent: 200,
  suggestedFileStem: "Screenwide 2026-08-08 at 14.32.05",
  width: 3840,
};
