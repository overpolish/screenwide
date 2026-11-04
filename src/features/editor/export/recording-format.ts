// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { stripIslandChoice } from "../timeline/recording-strip/strip-island-channel";

import type { GifRange } from "../../../bindings/GifRange";
import type { RecordingFormat } from "../../../bindings/RecordingFormat";
import type {
  ToolPanelGif,
  ToolPanelRecordingFormat,
} from "../tools/tool-panel-export";
import type { ToolPanelStripItem } from "../tools/tool-panel-strip";
import type { EditorArtifact } from "../types";

type Recording = Extract<EditorArtifact, { kind: "recording" }>;

const SOUND_FORMATS: RecordingFormat[] = ["m4a", "wav"];

/** The longest stretch a GIF is made of, in milliseconds, as Rust's
 * `GIF_LONGEST_MS` has it. */
export const GIF_LONGEST_MS = 15_000;

/** The formats a recording is offered in and the one it is saved as: its
 * video, a stretch of it as a GIF, and its sound alone as M4A or WAV where
 * it has any. With no picture in the export, only the sound is offered. A
 * choice the recording cannot take, such as WAV after the sound is switched
 * off, falls back to the first offered. */
export function recordingFormatChoice(
  recording: Recording,
  { chosen, isAudioOnly }: { chosen: RecordingFormat; isAudioOnly: boolean },
): ToolPanelRecordingFormat {
  const formats: RecordingFormat[] = isAudioOnly
    ? SOUND_FORMATS
    : recording.audioTracks.length > 0
      ? ["mp4", "gif", ...SOUND_FORMATS]
      : ["mp4", "gif"];
  return {
    offered: formats.map((format) => ({
      extension: format === "mp4" ? recording.extension : format,
      format,
    })),
    selected: formats.includes(chosen) ? chosen : formats[0],
  };
}

/** The stretch of the edited video drawn on the strip, which a GIF is made
 * of; null with no range drawn, or one that plays for no time. */
export function gifRangeOf(item: ToolPanelStripItem | null): GifRange | null {
  if (item?.kind !== "range") return null;
  const { endMs, startMs } = item.played;
  return endMs > startMs ? { endMs, startMs } : null;
}

/** Whether a GIF can be made of what is drawn on the strip. */
export const gifReadiness = (range: GifRange | null): ToolPanelGif =>
  range === null
    ? "noRange"
    : range.endMs - range.startMs > GIF_LONGEST_MS
      ? "tooLong"
      : "ready";

/** The format the save is asked for, with the stretch a GIF is made of;
 * null for any other format. */
export type RecordingFormatPayload = {
  gifRange: GifRange | null;
  recordingFormat: RecordingFormat;
};

export const recordingFormatPayload = (
  recordingFormat: RecordingFormat,
): RecordingFormatPayload => ({
  gifRange:
    recordingFormat === "gif"
      ? gifRangeOf(stripIslandChoice("recording").item)
      : null,
  recordingFormat,
});
