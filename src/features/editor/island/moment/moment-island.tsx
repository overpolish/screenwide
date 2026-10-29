// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Flag, Scissors } from "lucide-react";
import { type CSSProperties } from "react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import { MomentPlayer } from "./moment-player";
import { MomentTranscriptLine } from "./moment-transcript";

import type { ToolPanelModel } from "../../tool-panels/tool-panel-selection";
import type { EditorIslandContent } from "../editor-island";

// A moment flagged while recording, in hand: its kind, in the kind's
// colour, and the cut every beat has. With a voice note, the note hangs
// beneath it: played here, its waveform filling with the kind's colour,
// and what was said in it under that. A moment is flagged for what
// happened in it, so it is heard and read before it is kept or cut.

/** What was said in a voice note, as far as transcription has got; with no
 * model to transcribe with, the model it needs and how far it has come
 * down. */
export type MomentTranscript =
  | { model: ToolPanelModel; status: "noModel" }
  | { status: "failed" }
  | { status: "queued" }
  | { status: "ready"; text: string }
  | { status: "transcribing" };

export type MomentNote = {
  isPlaying: boolean;
  /** Its peaks from 0 to 1, evenly along it. */
  peaks: number[];
  /** How far it has played, from 0 to 1. */
  progress: number;
  transcript: MomentTranscript;
};

export type MomentIslandProps = {
  /** The kind's colour, `#rrggbb`. */
  color: string;
  id: string;
  /** The kind's name. */
  name: string;
  /** The voice note recorded with it, if one was. */
  note: MomentNote | null;
  onCut: () => void;
  onDownloadModel: () => void;
  onTogglePlay: () => void;
};

export function momentIsland(props: MomentIslandProps): EditorIslandContent {
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-moment-confirm-cut")}
        icon={<Scissors />}
        label={t("editor-island-moment-cut")}
        onConfirm={props.onCut}
        variant="ghost"
      />
    ),
    // Every note keeps to the start, on the island's text column, at the
    // foot of the block; needing a model spans it, as what was said will.
    anchor: "start",
    below: props.note ? <MomentPlayer {...props} note={props.note} /> : null,
    key: `moment-${props.id}`,
    section: props.note ? (
      <MomentTranscriptLine
        onDownloadModel={props.onDownloadModel}
        transcript={props.note.transcript}
      />
    ) : null,
    // Asking for the model gives way to its progress as the download
    // starts, as one stage of the transcript does to the next.
    sectionKey: props.note
      ? `moment-${props.id}-${transcriptStage(props.note.transcript)}`
      : undefined,
    title: (
      <IslandRowTitle
        glyph={
          // The kind's own colour, drawn whole rather than at the title
          // glyph's alpha, as the recording island draws it.
          <Flag
            style={{ "--icon-alpha": 1, color: props.color } as CSSProperties}
          />
        }
        title={props.name}
      />
    ),
  };
}

function transcriptStage(transcript: MomentTranscript) {
  return transcript.status === "noModel" &&
    transcript.model.status === "downloading"
    ? "downloading"
    : transcript.status;
}
