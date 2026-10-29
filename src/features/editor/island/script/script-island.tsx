// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScrollText } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import { ScriptCaptions } from "./script-captions";

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { CaptionsPatch } from "../../recording/captions/recording-captions";
import type { CaptionText } from "../../tool-panels/captions/caption-text";
import type { TranscriptCorrection } from "../../tool-panels/tool-panel-transcript";
import type { TranscriptMark } from "../../tool-panels/transcript/use-transcript-marks";
import type { EditorIslandContent } from "../editor-island";

// The recording's script: what was said, as a whole, to read and put right
// in one pass rather than a caption at a time. It stands at the editor's
// end, the picture between it and the island it is open beside: with the
// Captions island, it shows the script as the captions break it, and edits
// their breaks, hidden words and key words. A word's spelling is the
// transcript's, so a correction made here shows wherever the script does.
// The Transcript mode will open it alone, to cut, as the next part of it.

export type ScriptIslandProps = {
  captions: RecordingCaptions;
  /** The recording's length, which breaks and choices are shares of. */
  durationMs: number;
  /** The fillers and profanity marked in the script, by word. */
  marks: ReadonlyMap<number, TranscriptMark>;
  /** A break, a hidden or shown word or a key word placed or taken away by
   * hand, which undoes with the edit. */
  onCaptionsChange: (patch: CaptionsPatch) => void;
  /** Words' spelling put right, or back to what was heard, in the
   * transcript; several at once are one step of the edit history. */
  onCorrect: (corrections: TranscriptCorrection[]) => void;
  /** Moves the playhead, to a word's caption when one is pressed. */
  onSeek: (ms: number) => void;
  playheadMs: number;
  /** The script as the captions last made break it. */
  text: CaptionText;
};

export function scriptIsland(props: ScriptIslandProps): EditorIslandContent {
  return {
    anchor: "end",
    key: "script",
    section: <ScriptCaptions {...props} />,
    title: (
      <IslandRowTitle
        glyph={<ScrollText aria-hidden className="size-icon" />}
        title={t("editor-island-script")}
      />
    ),
  };
}
