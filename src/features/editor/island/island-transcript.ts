// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ToolPanelModel } from "../tool-panels/tool-panel-selection";

/** Where the recording's transcript stands, which the captions are made of
 * and the script shows. */
export type IslandTranscript =
  | { model: ToolPanelModel; status: "noModel" }
  | { progress: number; status: "transcribing" }
  | { status: "failed" }
  | { status: "noSpeech" }
  | { status: "ready" };

/** Changes when `IslandTranscriptLine` becomes another, so the section
 * crossfades: a model's download is its own state. */
export const islandTranscriptKey = (transcript: IslandTranscript) =>
  transcript.status === "noModel" && transcript.model.status === "downloading"
    ? "downloading"
    : transcript.status;
