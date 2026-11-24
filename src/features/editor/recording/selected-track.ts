// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type EditorArtifact,
  recordingAudioTrackId,
  type RecordingTrackId,
} from "../types";

/** The track the editor has in hand: the one chosen for this recording
 * where it still has it, else its screen, its camera or its first sound, in
 * that order; none outside a recording. */
export const selectedRecordingTrack = (
  artifact: EditorArtifact | null,
  selected: { artifactId: number; trackId: RecordingTrackId | null } | null,
): RecordingTrackId | null => {
  if (artifact?.kind !== "recording") return null;
  if (
    selected?.artifactId === artifact.id &&
    (selected.trackId === null ||
      selected.trackId === "primary" ||
      selected.trackId === "camera" ||
      artifact.audioTracks.some(
        (track) =>
          recordingAudioTrackId(track.streamIndex) === selected.trackId,
      ))
  )
    return selected.trackId;
  if (artifact.primaryKind !== "audio") return "primary";
  if (artifact.camera) return "camera";
  const [first] = artifact.audioTracks;
  return artifact.audioTracks.length > 0
    ? recordingAudioTrackId(first.streamIndex)
    : null;
};
