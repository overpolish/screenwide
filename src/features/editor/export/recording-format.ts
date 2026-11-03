// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type { ToolPanelRecordingFormat } from "../tools/tool-panel-export";
import type { EditorArtifact } from "../types";

type Recording = Extract<EditorArtifact, { kind: "recording" }>;

const SOUND_FORMATS: RecordingFormat[] = ["m4a", "wav"];

/** The formats a recording is offered in and the one it is saved as: its
 * video, and its sound alone as M4A or WAV where it has any. With no picture
 * in the export, only the sound is offered. A choice the recording cannot
 * take, such as WAV after the sound is switched off, falls back to the first
 * offered. */
export function recordingFormatChoice(
  recording: Recording,
  { chosen, isAudioOnly }: { chosen: RecordingFormat; isAudioOnly: boolean },
): ToolPanelRecordingFormat {
  const formats: RecordingFormat[] = isAudioOnly
    ? SOUND_FORMATS
    : recording.audioTracks.length > 0
      ? ["mp4", ...SOUND_FORMATS]
      : ["mp4"];
  return {
    offered: formats.map((format) => ({
      extension: format === "mp4" ? recording.extension : format,
      format,
    })),
    selected: formats.includes(chosen) ? chosen : formats[0],
  };
}
