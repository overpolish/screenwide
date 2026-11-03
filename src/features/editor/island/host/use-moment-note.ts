// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useModel } from "../../recording/microphone/use-model";
import { useNotePlayback } from "../../timeline/moments/use-note-playback";

import type { NoteTranscript } from "../../timeline/moments/recording-moments";
import type { ToolPanelModel } from "../../tools/tool-panel-selection";
import type { ToolPanelStripItem } from "../../tools/tool-panel-strip";
import type { MomentNote, MomentTranscript } from "../moment/moment-island";

// The chosen moment's voice note, played here in the island's own window
// rather than by the editor: the note is a file of its own, apart from the
// recording the editor plays, and the editor pauses while a note plays
// (`usePauseForMomentNotes`). Notes are transcribed with the Transcripts
// model, offered for download here as every island offers its models.

/** What was said in the note as the island tells it; with no word yet,
 * waiting its turn, and with no model, the model it needs once the app has
 * said where that is. */
function momentTranscript(
  transcript: NoteTranscript | null,
  model: ToolPanelModel | null,
): MomentTranscript {
  if (transcript === null) return { status: "queued" };
  if (transcript.status !== "noModel") return transcript;
  return model ? { model, status: "noModel" } : { status: "queued" };
}

/** The chosen moment's voice note as its island shows it, and its asks. */
export type MomentNoteState = {
  /** Null where the moment has no voice note, or none is chosen. */
  note: MomentNote | null;
  onDownloadModel: () => void;
  onTogglePlay: () => void;
};

export function useMomentNote(
  item: ToolPanelStripItem | null | undefined,
): MomentNoteState {
  const moment = item?.kind === "moment" ? item : null;
  const playback = useNotePlayback(
    moment?.artifactId ?? 0,
    moment?.note ? moment.index : null,
  );
  const model = useModel("transcripts");
  return {
    note: moment?.note
      ? {
          isPlaying: playback.playing,
          peaks: moment.note.peaks,
          progress: playback.progress,
          transcript: momentTranscript(moment.note.transcript, model.shown),
        }
      : null,
    onDownloadModel: model.download,
    onTogglePlay: () => {
      void playback.toggle();
    },
  };
}
