// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Flag, Scissors } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import { MomentPlayer } from "./moment-player";
import { MomentTranscriptLine } from "./moment-transcript";

import type { EditorIslandContent } from "../editor-island";

// A moment flagged while recording, in hand: its kind, in the kind's
// colour, and the cut every beat has. With a voice note, the note hangs
// beneath it: played here, its waveform filling with the kind's colour,
// and what was said in it under that. A moment is flagged for what
// happened in it, so it is heard and read before it is kept or cut.

/** What was said in a voice note, as far as transcription has got. With no
 * model to transcribe with, `download` is how far one has come down, from
 * 0 to 1, or null before it is asked for. */
export type MomentTranscript =
  | { download: number | null; status: "noModel" }
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
  const status = props.note?.transcript.status;
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-moment-confirm-cut")}
        icon={<Scissors aria-hidden />}
        label={t("editor-island-moment-cut")}
        onConfirm={props.onCut}
        variant="ghost"
      />
    ),
    // While what was said is being worked out, its line is centred; a note
    // with nothing to show hangs its line from the start, as the recording
    // and annotate islands hang their parts. Needing a model spans the
    // island, as what was said will.
    anchor:
      status === "queued" || status === "transcribing" ? "center" : "start",
    below: props.note ? <MomentPlayer {...props} note={props.note} /> : null,
    key: `moment-${props.id}`,
    section: props.note ? (
      <MomentTranscriptLine
        onDownloadModel={props.onDownloadModel}
        transcript={props.note.transcript}
      />
    ) : null,
    sectionKey: props.note
      ? `moment-${props.id}-${props.note.transcript.status}`
      : undefined,
    title: (
      <IslandRowTitle
        glyph={<Flag style={{ color: props.color }} />}
        title={props.name}
      />
    ),
  };
}
