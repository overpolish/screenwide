// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps, useMemo, useState } from "react";

import { recordingAudioTrackId, RecordingTrackId } from "../types";

import { EditorPanel } from "./editor-panel";

type Args = ComponentProps<typeof EditorPanel>;

/** Keeps a change in the story's own state and still logs it through the
 * story's arg. */
const keep =
  <Value,>(
    arg: ((value: Value) => void) | undefined,
    set: (value: Value) => void,
  ) =>
  (value: Value) => {
    arg?.(value);
    set(value);
  };

/** Renames only the story's own copy of the project. */
function useStoryRename(args: Args) {
  const [title, setTitle] = useState(args.artifact?.suggestedFileStem ?? "");
  const renamed = useMemo(
    () =>
      args.artifact ? { ...args.artifact, suggestedFileStem: title } : null,
    [args.artifact, title],
  );
  return {
    artifact: renamed,
    onRenameProject: keep(args.onRenameProject, setTitle),
  };
}

export function ScreenshotStoryPanel(args: Args) {
  const [fileStem, setFileStem] = useState(args.fileStem);
  const project = useStoryRename(args);
  return (
    <EditorPanel
      {...args}
      artifact={project.artifact}
      fileStem={fileStem}
      onFileStemChange={keep(args.onFileStemChange, setFileStem)}
      onRenameProject={project.onRenameProject}
    />
  );
}

export function AudioRecordingStoryPanel(args: Args) {
  const [enabledTracks, setEnabledTracks] = useState([0, 1]);
  const [fileStem, setFileStem] = useState(args.fileStem);
  const project = useStoryRename(args);
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
      onEnabledTracksChange={keep(args.onEnabledTracksChange, setEnabledTracks)}
      onFileStemChange={keep(args.onFileStemChange, setFileStem)}
      onRenameProject={project.onRenameProject}
      onSelectedTrackChange={keep(args.onSelectedTrackChange, setSelectedTrack)}
      onTrackVolumeChange={(streamIndex, decibels) => {
        args.onTrackVolumeChange?.(streamIndex, decibels);
        setVolumes((current) => ({ ...current, [streamIndex]: decibels }));
      }}
      selectedTrack={selectedTrack}
    />
  );
}

export function RecordingStoryPanel(args: Args) {
  const recording = args.artifact?.kind === "recording" ? args.artifact : null;
  const [fileStem, setFileStem] = useState(args.fileStem);
  const project = useStoryRename(args);
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
      cameraCompression={cameraCompression}
      cameraOverlay={cameraOverlay}
      cameraResolutionScalePercent={cameraResolution}
      collapseAudio={collapseAudio}
      compression={compression}
      enabledAudioTrackCount={enabledAudio.length}
      enabledStreamIndices={enabledAudio}
      enabledVideoTracks={enabledVideo}
      fileStem={fileStem}
      onCameraCompressionChange={keep(
        args.onCameraCompressionChange,
        setCameraCompression,
      )}
      onCameraOverlayChange={keep(args.onCameraOverlayChange, setCameraOverlay)}
      onCameraResolutionScaleChange={keep(
        args.onCameraResolutionScaleChange,
        setCameraResolution,
      )}
      onCollapseAudioChange={keep(args.onCollapseAudioChange, setCollapseAudio)}
      onCompressionChange={keep(args.onCompressionChange, setCompression)}
      onEnabledTracksChange={keep(args.onEnabledTracksChange, setEnabledAudio)}
      onEnabledVideoTracksChange={keep(
        args.onEnabledVideoTracksChange,
        setEnabledVideo,
      )}
      onFileStemChange={keep(args.onFileStemChange, setFileStem)}
      onRenameProject={project.onRenameProject}
      onResolutionScaleChange={keep(
        args.onResolutionScaleChange,
        setResolution,
      )}
      onSelectedTrackChange={keep(args.onSelectedTrackChange, setSelectedTrack)}
      resolutionScalePercent={resolution}
      selectedTrack={selectedTrack}
    />
  );
}
