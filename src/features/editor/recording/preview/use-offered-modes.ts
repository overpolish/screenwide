// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import {
  captureParts,
  editorModeGroups,
  type EditorMode,
  offeredModes,
  recordedTrackCount,
  type TranscriptState,
} from "../../toolbar/editor-modes";
import { useTranscriptRevision } from "../transcript/transcript-revision";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";
import type { RecordingVideoTrackId } from "../../types";

/**
 * The modes the title bar offers for the recording open, as its export
 * stands: a track switched off, or a transcript that heard nothing, takes
 * its modes with it, and the mode open among them is put away as its
 * button would (`backToSelect`). Tracks is offered where there are two
 * recorded tracks or more to choose from, and by the preview where songs
 * or sound effects are laid under the recording, each kind a switch there.
 */
export function useOfferedModes(
  { artifactId, hasAnnotations, recorded }: ResolvedScrubPreviewProps,
  column: { backToSelect: () => void; mode: EditorMode | null },
  tracks: {
    /** The audio streams in the export. */
    enabledTracks: ReadonlySet<number>;
    /** The video tracks in the export. */
    selectedVideoTracks: ReadonlySet<RecordingVideoTrackId>;
  },
) {
  // A recording without a picture is edited as audio.
  const kind = hasAnnotations ? "recording" : "audio";
  const transcript = useTranscriptRevision(
    ({ artifactId: id, hasWords, isReady }): TranscriptState =>
      id !== artifactId || !isReady ? "pending" : hasWords ? "heard" : "silent",
  );
  const parts = captureParts({
    enabledStreams: tracks.enabledTracks,
    enabledVideoTracks: tracks.selectedVideoTracks,
    recorded,
    transcript,
  });
  const offered = offeredModes(editorModeGroups({ kind, parts }));
  const isModeGone = column.mode !== null && !offered.has(column.mode);
  const { backToSelect } = column;
  useEffect(() => {
    if (isModeGone) backToSelect();
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- only the mode leaving puts it away; `backToSelect` is remade each render.
  }, [isModeGone]);
  return {
    hasTrackChoice: recordedTrackCount(recorded) > 1,
    kind,
    offered,
    parts,
  } as const;
}
