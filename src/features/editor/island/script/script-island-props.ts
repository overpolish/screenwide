// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { CaptionsPatch } from "../../recording/captions/recording-captions";
import type { CaptionText } from "../../tool-panels/captions/caption-text";
import type {
  ToolPanelTranscript,
  TranscriptCorrection,
} from "../../tool-panels/tool-panel-transcript";
import type { TranscriptEditingProps } from "../../tool-panels/transcript/use-transcript-editing";
import type { TranscriptMark } from "../../tool-panels/transcript/use-transcript-marks";
import type { IslandTranscript } from "../island-transcript";

/** Words' spelling put right, or back to what was heard, in the
 * transcript; several at once are one step of the edit history. */
type OnCorrect = (corrections: TranscriptCorrection[]) => void;

/** The script beside the Captions island, as the captions break it. */
export type ScriptCaptionsProps = {
  captions: RecordingCaptions;
  /** The recording's length, which breaks and choices are shares of. */
  durationMs: number;
  /** The fillers and profanity marked in the script, by word. */
  marks: ReadonlyMap<number, TranscriptMark>;
  /** A break, a hidden or shown word or a key word placed or taken away by
   * hand, which undoes with the edit. */
  onCaptionsChange: (patch: CaptionsPatch) => void;
  onCorrect: OnCorrect;
  /** Moves the playhead, to a word's caption when one is pressed. */
  onSeek: (ms: number) => void;
  playheadMs: number;
  /** The script as the captions last made break it. */
  text: CaptionText;
};

/** The script on its own, to cut, move and put right. */
export type ScriptTranscriptProps = Omit<
  TranscriptEditingProps,
  "onCorrect" | "transcript"
> & {
  onCorrect: OnCorrect;
  onDownloadModel: () => void;
  /** Cuts the pauses the microphone's Remove silences finds. */
  onRemoveSilences: () => void;
  /** Brings back every cut made in the script, and every pause cut. */
  onRestoreAll: () => void;
  onTranscribe: () => void;
  /** The script's edits: what is cut, corrected, moved and allowed. */
  script: TranscriptEditingProps["transcript"] &
    Pick<ToolPanelTranscript, "canRestore" | "silences">;
  transcript: IslandTranscript;
};

export type ScriptIslandProps =
  | (ScriptCaptionsProps & { mode: "captions" })
  | (ScriptTranscriptProps & { mode: "transcript" });
