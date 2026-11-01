// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import {
  EditorArtifact,
  recordingAudioStreamIndex,
  recordingAudioTrackId,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../../types";
import { type ToolPanelHandlers } from "../tool-panel-handlers";
import { type ToolPanelTrack } from "../tool-panel-strip";

// The recorded tracks the Tracks island switches, read from what the editor
// holds in the export: the screen or camera the recording was made of, a
// camera beside it, then each audio stream. Something has to be left to
// export, so the last track still in is never left out.

/** Which audio streams the export holds, and how to change which tracks
 * it holds. */
export type TrackInclusion = {
  /** The audio streams in the export; every one where unset. */
  enabledStreamIndices: number[] | undefined;
  onEnabledTracksChange?: (streamIndices: number[]) => void;
  onEnabledVideoTracksChange?: (tracks: RecordingVideoTrackId[]) => void;
};

export type EditorTracksInputs = TrackInclusion & {
  artifact: EditorArtifact | null;
  enabledVideoTracks: RecordingVideoTrackId[];
};

export function editorTracks({
  artifact,
  enabledStreamIndices,
  enabledVideoTracks,
}: EditorTracksInputs): ToolPanelTrack[] {
  if (artifact?.kind !== "recording") return [];
  const audio =
    enabledStreamIndices ??
    artifact.audioTracks.map(({ streamIndex }) => streamIndex);
  const video: ToolPanelTrack[] = [];
  if (artifact.primaryKind !== "audio")
    video.push({
      id: "primary",
      isEnabled: enabledVideoTracks.includes("primary"),
      label:
        artifact.primaryKind === "camera"
          ? t("editor-island-camera")
          : t("editor-island-screen"),
    });
  if (artifact.camera)
    video.push({
      id: "camera",
      isEnabled: enabledVideoTracks.includes("camera"),
      label: t("editor-island-camera"),
    });
  return [
    ...video,
    ...artifact.audioTracks.map((track) => ({
      id: recordingAudioTrackId(track.streamIndex),
      isEnabled: audio.includes(track.streamIndex),
      label: track.label || t("editor-island-audio"),
    })),
  ];
}

/** What the Tracks island's switches do: each holds its track in the
 * export and the preview or leaves it out, never the last one in. */
export const tracksPanelHandlers = (
  inputs: EditorTracksInputs,
): Pick<ToolPanelHandlers, "onTrackEnabledChange"> => ({
  onTrackEnabledChange: (trackId, isEnabled) => {
    const enabled = editorTracks(inputs)
      .filter((track) => track.isEnabled && track.id !== trackId)
      .map(({ id }) => id as RecordingTrackId);
    if (!isEnabled && enabled.length === 0) return;
    const next = isEnabled
      ? [...enabled, trackId as RecordingTrackId]
      : enabled;
    if (recordingAudioStreamIndex(trackId as RecordingTrackId) === null)
      inputs.onEnabledVideoTracksChange?.(
        next.filter(
          (id): id is RecordingVideoTrackId =>
            id === "camera" || id === "primary",
        ),
      );
    else
      inputs.onEnabledTracksChange?.(
        next.flatMap((id) => recordingAudioStreamIndex(id) ?? []),
      );
  },
});
