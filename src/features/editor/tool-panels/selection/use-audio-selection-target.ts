// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMicrophoneTools } from "../../recording/microphone/use-microphone-tools";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { AudioTrackVolume, EditorArtifact } from "../../types";

import { editorAudioSelectionTarget } from "./selection-target";

/**
 * The selected audio track as the selection panel acts on it: the level the
 * editor holds for it, the editor's own way of setting it, and for the
 * microphone the speech tools, which edit the timeline.
 */
export function useAudioSelectionTarget({
  artifact,
  audioTrackVolumes,
  edit,
  onEditChange,
  onSelectedTrackVolumeChange,
  selectedTrack,
}: {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  edit: RecordingTimelineEdit | null | undefined;
  selectedTrack: string | null;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
  onSelectedTrackVolumeChange?: (decibels: number) => void;
}) {
  const microphone = useMicrophoneTools({ artifact, edit, onEditChange });
  return editorAudioSelectionTarget({
    artifact,
    audioTrackVolumes,
    microphone,
    onSelectedTrackVolumeChange,
    selectedTrack,
  });
}
