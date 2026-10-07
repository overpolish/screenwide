// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps, useMemo, useState } from "react";

import {
  EditorArtifact,
  recordingAudioStreamIndex,
  recordingAudioTrackId,
  RecordingTrackId,
} from "../types";

import { EditorPanel } from "./editor-panel";

/** Renames only the story's own copy of the project. */
function useStoryRename(artifact: EditorArtifact | null) {
  const [title, setTitle] = useState(artifact?.suggestedFileStem ?? "");
  const renamed = useMemo(
    () => (artifact ? { ...artifact, suggestedFileStem: title } : null),
    [artifact, title],
  );
  return { artifact: renamed, onRenameProject: setTitle };
}

export function ScreenshotStoryPanel(args: ComponentProps<typeof EditorPanel>) {
  const [fileStem, setFileStem] = useState(args.fileStem);
  const project = useStoryRename(args.artifact);
  return (
    <EditorPanel
      {...args}
      artifact={project.artifact}
      fileStem={fileStem}
      onCopy={() => undefined}
      onFileStemChange={setFileStem}
      onRenameProject={project.onRenameProject}
      onSave={() => undefined}
    />
  );
}

export function AudioRecordingStoryPanel(
  args: ComponentProps<typeof EditorPanel>,
) {
  const [enabledTracks, setEnabledTracks] = useState([0, 1]);
  const [fileStem, setFileStem] = useState(args.fileStem);
  const project = useStoryRename(args.artifact);
  const [selectedTrack, setSelectedTrack] = useState<RecordingTrackId | null>(
    () => recordingAudioTrackId(0),
  );
  const [volumes, setVolumes] = useState<Record<number, number>>({});
  return (
    <EditorPanel
      {...args}
      artifact={project.artifact}
      audioTrackVolumes={Object.entries(volumes).map(
        ([streamIndex, decibels]) => ({
          decibels,
          streamIndex: Number(streamIndex),
        }),
      )}
      enabledAudioTrackCount={enabledTracks.length}
      enabledStreamIndices={enabledTracks}
      fileStem={fileStem}
      onEnabledTracksChange={setEnabledTracks}
      onFileStemChange={setFileStem}
      onRenameProject={project.onRenameProject}
      onSelectedTrackChange={setSelectedTrack}
      onSelectedTrackVolumeChange={(decibels) => {
        const streamIndex = recordingAudioStreamIndex(selectedTrack);
        if (streamIndex === null) return;
        setVolumes((current) => ({ ...current, [streamIndex]: decibels }));
      }}
      selectedTrack={selectedTrack}
    />
  );
}

export function RecordingStoryPanel(args: ComponentProps<typeof EditorPanel>) {
  const recording = args.artifact?.kind === "recording" ? args.artifact : null;
  const [fileStem, setFileStem] = useState(args.fileStem);
  const project = useStoryRename(args.artifact);
  const [bakeCamera, setBakeCamera] = useState(args.bakeCamera ?? false);
  const [cameraCompression, setCameraCompression] = useState(
    args.cameraCompression ?? 0,
  );
  const [cameraOverlay, setCameraOverlay] = useState(args.cameraOverlay);
  const [cameraResolution, setCameraResolution] = useState(
    args.cameraResolutionScalePercent ?? 100,
  );
  const [collapseAudio, setCollapseAudio] = useState(
    args.collapseAudio ?? false,
  );
  const [compression, setCompression] = useState(args.compression ?? 0);
  const [enabledAudio, setEnabledAudio] = useState(
    () =>
      args.enabledStreamIndices ??
      recording?.audioTracks.map((track) => track.streamIndex) ??
      [],
  );
  const [enabledVideo, setEnabledVideo] = useState(
    () => args.enabledVideoTracks ?? [],
  );
  const [resolution, setResolution] = useState(
    args.resolutionScalePercent ?? 100,
  );
  const [selectedTrack, setSelectedTrack] = useState<RecordingTrackId | null>(
    () =>
      args.selectedTrack ??
      (recording?.primaryKind === "audio"
        ? recording.audioTracks[0]
          ? recordingAudioTrackId(recording.audioTracks[0].streamIndex)
          : null
        : "primary"),
  );

  return (
    <EditorPanel
      {...args}
      artifact={project.artifact}
      bakeCamera={bakeCamera}
      cameraCompression={cameraCompression}
      cameraOverlay={cameraOverlay}
      cameraResolutionScalePercent={cameraResolution}
      collapseAudio={collapseAudio}
      compression={compression}
      enabledAudioTrackCount={enabledAudio.length}
      enabledStreamIndices={enabledAudio}
      enabledVideoTracks={enabledVideo}
      fileStem={fileStem}
      onBakeCameraChange={setBakeCamera}
      onCameraCompressionChange={setCameraCompression}
      onCameraOverlayChange={setCameraOverlay}
      onCameraResolutionScaleChange={setCameraResolution}
      onCollapseAudioChange={setCollapseAudio}
      onCompressionChange={setCompression}
      onEnabledTracksChange={setEnabledAudio}
      onEnabledVideoTracksChange={setEnabledVideo}
      onFileStemChange={setFileStem}
      onRenameProject={project.onRenameProject}
      onResolutionScaleChange={setResolution}
      onSelectedTrackChange={setSelectedTrack}
      resolutionScalePercent={resolution}
      selectedTrack={selectedTrack}
    />
  );
}
