// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  browseExportDirectory,
  copyEditorToClipboard,
  setExportDirectory,
} from "./api";
import {
  closeEditorWindow,
  minimizeEditorWindow,
  toggleEditorWindowMaximized,
} from "./editor-actions";
import { cameraOutputChoice } from "./export/camera-output";
import { cancelActiveExport } from "./export/cancel-active-export";
import {
  clearExportFailure,
  failExport,
  finishExport,
} from "./export/export-island-store";
import { seededExportChoices } from "./export/recording-export-choices";
import {
  cameraExportSettings,
  DEFAULT_COMPRESSION,
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
  recordingSavePlan,
} from "./export/recording-export-settings";
import { saveFormats } from "./export/save-formats";
import { useExportMeasures } from "./export/use-export-measures";
import { useExportProgress } from "./export/use-export-progress";
import { useRecordingFormat } from "./export/use-recording-format";
import { useScreenshotExport } from "./export/use-screenshot-export";
import { openProjectActions } from "./open-project-actions";
import { EditorPanel } from "./panel/editor-panel";
import { useNativePressBlur } from "./preview/use-native-press-blur";
import { usePointerAway } from "./preview/use-pointer-away";
import { useEditorFormats } from "./recording/formats/use-editor-formats";
import { openedRecordingOutput } from "./recording/recording-captures";
import { recordingOutputForEdit } from "./recording/recording-output-edit";
import { selectedRecordingTrack } from "./recording/selected-track";
import { withTrackVolume } from "./recording/track-volumes";
import {
  seededTrackChoices,
  useProjectLook,
} from "./recording/use-project-look";
import { useRecordingEditorPreview } from "./recording/use-recording-editor-preview";
import { sourceScalePercent } from "./resolution";
import { newestScreenshotLayer } from "./screenshot/screenshot-blank";
import {
  defaultRecordingOutput,
  defaultScreenshotOutput,
  RecordingOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  withScreenshotWorkspaceItemOutput,
} from "./screenshot/screenshot-output";
import {
  reopenedScreenshotWorkspace,
  seedScreenshotWorkspace,
} from "./screenshot/screenshot-seed";
import { useScreenshotLayers } from "./screenshot/use-screenshot-layers";
import { useScreenshotProject } from "./screenshot/use-screenshot-project";
import {
  selectArtifact,
  selectDirectory,
  selectSnapshot,
  useEditorStore,
} from "./store";
import { useRecordingTimelineEditState } from "./timeline/editing/use-recording-timeline-edit-state";
import {
  AudioTrackVolume,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "./types";
import {
  EditorEditGestureContext,
  EditorEditState,
  useEditorEditHistory,
} from "./use-editor-edit-history";
import { currentEditorKind } from "./window-kind";

import type { CameraKey } from "../../bindings/CameraKey";
import type { ExportSubtitles } from "../../bindings/ExportSubtitles";
import type { FrameFormats } from "../../bindings/FrameFormats";

const EMPTY_AUDIO_TRACK_VOLUMES: AudioTrackVolume[] = [];

export function EditorWindow() {
  // This webview is one workspace's window and only ever renders that one.
  // Its label is what says which, so nothing has to be passed in or invoked.
  const kind = currentEditorKind() ?? "recording";
  const artifact = useEditorStore(selectArtifact(kind));
  const directory = useEditorStore(selectDirectory(kind));
  const {
    cursorEffects: persistedCursorEffects,
    keyboardEffects: persistedKeyboardEffects,
    recordingExportChoices: persistedRecordingExportChoices,
    recordingOutput: persistedRecordingOutput,
    screenshotBackgroundRadiusPercent: persistedScreenshotBackgroundRadius,
    screenshotDeleteProjectAfterExport: persistedScreenshotDeleteProject,
    screenshotOutput: persistedScreenshotOutput,
    screenshotRadius: persistedScreenshotRadius,
  } = useEditorStore(selectSnapshot(kind));
  const [fileStem, setFileStem] = useState("");
  const [collapseAudio, setCollapseAudio] = useState(false);
  const [cameraKey, setCameraKey] = useState<CameraKey>("green");
  const [deleteProjectAfterExport, setDeleteProjectAfterExport] =
    useState(false);
  const [subtitles, setSubtitles] = useState<ExportSubtitles>("none");
  const [compression, setCompression] = useState(DEFAULT_COMPRESSION);
  const [cameraCompression, setCameraCompression] =
    useState(DEFAULT_COMPRESSION);
  const [bakeCamera, setBakeCamera] = useState(false);
  const [cameraOverlay, setCameraOverlay] = useState(defaultCameraOverlay);
  const [cursorEffects, setCursorEffects] = useState(DEFAULT_CURSOR_EFFECTS);
  const [keyboardEffects, setKeyboardEffects] = useState(
    DEFAULT_KEYBOARD_EFFECTS,
  );
  const [cameraResolutionScalePercent, setCameraResolutionScalePercent] =
    useState(100);
  const [resolutionScalePercent, setResolutionScalePercent] = useState(100);
  const [screenshotOutput, setScreenshotOutput] =
    useState<ScreenshotWorkspaceOutputSettings>(() => ({
      ...defaultScreenshotOutput(1, 1),
      items: [],
    }));
  const [selectedScreenshotItemId, setSelectedScreenshotItemId] = useState<
    number | null
  >(null);
  const [recordingOutput, setRecordingOutput] =
    useState<RecordingOutputSettings>(() =>
      defaultRecordingOutput({ primary: { height: 1, width: 1 } }),
    );
  const [recordingFormats, setRecordingFormats] = useState<FrameFormats | null>(
    null,
  );
  const [recordingTimelineEdit, setTimelineEdit, commitTimelineEdit] =
    useRecordingTimelineEditState(artifact);
  const [isSaving, setIsSaving] = useState(false);
  const [isCancelingSave, setIsCancelingSave] = useState(false);

  useNativePressBlur();
  usePointerAway();
  const [trackSelection, setTrackSelection] = useState<{
    artifactId: number;
    streamIndices: number[];
  } | null>(null);
  const [videoTrackSelection, setVideoTrackSelection] = useState<{
    artifactId: number;
    tracks: RecordingVideoTrackId[];
  } | null>(null);
  const [selectedTrack, setSelectedTrack] = useState<{
    artifactId: number;
    trackId: RecordingTrackId | null;
  } | null>(null);
  const [audioTrackVolumes, setAudioTrackVolumes] = useState<{
    artifactId: number;
    values: AudioTrackVolume[];
  } | null>(null);
  const seenScreenshotItemIdsRef = useRef<Set<number>>(new Set());
  const openProject = openProjectActions();

  // Keyed on the capture rather than the object, so a replacement always
  // refetches - including the full-resolution copy, whose cached URL belongs to
  // the previous capture's pixels.
  const artifactId = artifact?.id;
  const saveProgress = useExportProgress(artifactId);
  const [recordingFormat, chooseRecordingFormat, recordingFormatPayload] =
    useRecordingFormat(artifactId);
  const originalResolutionScale =
    artifact?.kind === "recording" ? sourceScalePercent(artifact) : 100;
  const cameraExport = cameraExportSettings(
    artifact,
    cameraCompression,
    cameraResolutionScalePercent,
  );
  const shouldPrepareRecordingPreview =
    artifact?.kind === "recording" &&
    (artifact.audioTracks.length > 0 || artifact.camera !== null);
  const {
    error: recordingPreviewError,
    isPreparing: isPreparingRecordingPreview,
    preview: recordingPreview,
  } = useRecordingEditorPreview({
    artifactId,
    shouldPrepare: shouldPrepareRecordingPreview,
  });
  // Retain array identity so downstream selection effects do not reset.
  const recordingPreviewTracks = recordingPreview?.tracks;

  // Start with every recorded track until the user changes the selection.
  const enabledStreamIndices =
    artifact?.kind === "recording"
      ? trackSelection?.artifactId === artifact.id
        ? trackSelection.streamIndices
        : artifact.audioTracks.map((track) => track.streamIndex)
      : null;
  const defaultVideoTracks: RecordingVideoTrackId[] =
    artifact?.kind === "recording"
      ? [
          ...(artifact.primaryKind === "audio" ? [] : (["primary"] as const)),
          ...(artifact.camera ? (["camera"] as const) : []),
        ]
      : [];
  const selectedVideoTracks =
    artifact?.kind === "recording" &&
    videoTrackSelection?.artifactId === artifact.id
      ? videoTrackSelection.tracks
      : defaultVideoTracks;
  const enabledVideoTracks = selectedVideoTracks;
  const includePrimaryVideo = enabledVideoTracks.includes("primary");
  const includeCamera = enabledVideoTracks.includes("camera");
  const effectiveBakeCamera =
    bakeCamera && includePrimaryVideo && includeCamera;
  const cameraOutput = cameraOutputChoice({
    artifact,
    bakeCamera,
    enabledVideoTracks,
  });
  const formats = useEditorFormats({
    artifact,
    cameraOutput,
    cameraOverlay,
    commit: commitTimelineEdit,
    edit: recordingTimelineEdit,
    formats: recordingFormats,
    isRecording: artifact?.kind === "recording",
    onCameraOverlayChange: setCameraOverlay,
    onFormatsChange: setRecordingFormats,
    onRecordingOutputChange: setRecordingOutput,
    recordingOutput,
  });
  const { exportEdits } = formats;
  const effectiveCollapseAudio =
    collapseAudio && (enabledStreamIndices?.length ?? 0) > 1;
  const currentAudioTrackVolumes =
    artifact?.kind === "recording" &&
    audioTrackVolumes?.artifactId === artifact.id
      ? audioTrackVolumes.values
      : EMPTY_AUDIO_TRACK_VOLUMES;
  const selectedTrackId = selectedRecordingTrack(artifact, selectedTrack);
  const editState = useMemo<EditorEditState>(
    () => ({
      audioTrackVolumes,
      bakeCamera,
      cameraCompression,
      cameraOverlay,
      cameraResolutionScalePercent,
      collapseAudio,
      compression,
      cursorEffects,
      keyboardEffects,
      recordingFormats,
      recordingOutput,
      recordingTimelineEdit,
      resolutionScalePercent,
      screenshotOutput,
      trackSelection,
      videoTrackSelection,
    }),
    [
      audioTrackVolumes,
      bakeCamera,
      cameraCompression,
      cameraOverlay,
      cameraResolutionScalePercent,
      collapseAudio,
      compression,
      cursorEffects,
      keyboardEffects,
      recordingFormats,
      recordingOutput,
      recordingTimelineEdit,
      resolutionScalePercent,
      screenshotOutput,
      trackSelection,
      videoTrackSelection,
    ],
  );
  const applyEditState = useCallback(
    (next: EditorEditState) => {
      setAudioTrackVolumes(next.audioTrackVolumes);
      setBakeCamera(next.bakeCamera);
      setCameraCompression(next.cameraCompression);
      setCameraOverlay(next.cameraOverlay);
      setCameraResolutionScalePercent(next.cameraResolutionScalePercent);
      setCollapseAudio(next.collapseAudio);
      setCompression(next.compression);
      setCursorEffects(next.cursorEffects);
      setKeyboardEffects(next.keyboardEffects);
      setTimelineEdit(next.recordingTimelineEdit);
      setResolutionScalePercent(next.resolutionScalePercent);
      setRecordingFormats(next.recordingFormats);
      setRecordingOutput(recordingOutputForEdit(next.recordingOutput));
      setScreenshotOutput(next.screenshotOutput);
      setTrackSelection(next.trackSelection);
      setVideoTrackSelection(next.videoTrackSelection);
      clearExportFailure();
    },
    [setTimelineEdit],
  );
  // The history starts over when the seeded settings for an artifact land,
  // not when the artifact arrives: the seeding is a separate commit, and a
  // reset keyed on arrival could lift before it, leaving the placeholder
  // canvas as the first thing undo goes back to.
  const [seededArtifactId, setSeededArtifactId] = useState<number | undefined>(
    undefined,
  );
  const editGesture = useEditorEditHistory({
    apply: applyEditState,
    resetKey:
      artifact?.kind === "screenshot" && seededArtifactId !== undefined
        ? `${seededArtifactId.toString()}:${artifact.items.map((item) => item.id).join(":")}`
        : seededArtifactId,
    state: editState,
  });
  const exportInputs = {
    artifact,
    audioTrackVolumes: currentAudioTrackVolumes,
    bakeCamera: effectiveBakeCamera,
    camera: cameraExport,
    cameraOverlay,
    collapseAudio: effectiveCollapseAudio,
    compression,
    cursorEffects,
    enabledStreamIndices,
    includeCamera,
    includePrimaryVideo,
    keyboardEffects,
    recordingOutput: exportEdits.recordingOutput,
    recordingTimelineEdit: exportEdits.timelineEdit,
    resolutionScalePercent,
  };
  const screenshotExport = useScreenshotExport(screenshotOutput);
  const { exportCompare, lookQualities } = useExportMeasures(exportInputs, {
    cameraOutput,
    recordingFormat,
    screenshot: {
      format: screenshotExport.format,
      fullColour: screenshotExport.fullColour,
      output: screenshotOutput,
    },
  });

  const onEnabledTracksChange = useCallback(
    (streamIndices: number[]) => {
      if (artifactId === undefined) return;
      setTrackSelection({ artifactId, streamIndices });
    },
    [artifactId],
  );

  useEffect(() => {
    // A project that kept its look reopens with it exactly as it was left;
    // one that kept none takes the remembered look, fitted to this recording.
    const look =
      artifact?.kind === "recording" ? (artifact.projectLook ?? null) : null;
    const seededRecordingOutput = openedRecordingOutput(
      artifact,
      persistedRecordingOutput,
    );
    const seeded = seededExportChoices({
      artifact,
      choices: persistedRecordingExportChoices,
      screen: seededRecordingOutput.primary,
    });
    const tracks = seededTrackChoices(artifactId, look);
    /* eslint-disable @eslint-react/set-state-in-effect */
    setTrackSelection(tracks.trackSelection);
    setVideoTrackSelection(tracks.videoTrackSelection);
    setSelectedTrack(null);
    setAudioTrackVolumes(tracks.audioTrackVolumes);
    setBakeCamera(look?.bakeCamera ?? seeded.bakeCamera);
    setCameraOverlay(look?.cameraOverlay ?? seeded.cameraOverlay);
    setCursorEffects(look?.cursorEffects ?? persistedCursorEffects);
    setKeyboardEffects(look?.keyboardEffects ?? persistedKeyboardEffects);
    setCollapseAudio(seeded.collapseAudio);
    // Each kind keeps its own choice, as a screenshot is far cheaper to make
    // again than a recording.
    setDeleteProjectAfterExport(
      artifact?.kind === "screenshot"
        ? (persistedScreenshotDeleteProject ?? false)
        : seeded.deleteProjectAfterExport,
    );
    setCompression(seeded.compression);
    setSubtitles(seeded.subtitles);
    setCameraCompression(seeded.cameraCompression);
    setCameraResolutionScalePercent(seeded.cameraResolutionScalePercent);
    setResolutionScalePercent(seeded.resolutionScalePercent);
    setRecordingOutput(seededRecordingOutput);
    setRecordingFormats(look?.formats ?? null);
    setScreenshotOutput(
      artifact?.kind === "screenshot" && artifact.projectWorkspace
        ? reopenedScreenshotWorkspace(artifact, artifact.projectWorkspace)
        : seedScreenshotWorkspace({
            artifact,
            backgroundRadius: persistedScreenshotBackgroundRadius,
            persisted: persistedScreenshotOutput,
            radius: persistedScreenshotRadius,
          }),
    );
    setSelectedScreenshotItemId(
      artifact?.kind === "screenshot"
        ? (newestScreenshotLayer(
            artifact.items.filter(
              (item) => !artifact.deletedItemIds.includes(item.id),
            ),
          )?.id ?? null)
        : null,
    );
    seenScreenshotItemIdsRef.current = new Set(
      artifact?.kind === "screenshot"
        ? artifact.items.map((item) => item.id)
        : [],
    );
    setSeededArtifactId(artifactId);
    /* eslint-enable @eslint-react/set-state-in-effect */
    // A cancelled or failed save restores the same artifact through a fresh
    // snapshot. Its controls are still the user's current editing session and
    // must not be reset merely because the object was deserialized again.
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [artifactId]);
  useProjectLook({
    audioTrackVolumes,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    formats: recordingFormats ?? undefined,
    keyboardEffects,
    recordingId:
      artifact?.kind === "recording" && seededArtifactId === artifact.id
        ? artifact.id
        : undefined,
    recordingOutput,
    trackSelection,
    videoTrackSelection,
  });
  useScreenshotProject(
    artifact?.kind === "screenshot" && seededArtifactId === artifact.id
      ? artifact.id
      : undefined,
    screenshotOutput,
  );

  useScreenshotLayers({
    onSelectedItemChange: setSelectedScreenshotItemId,
    screenshot:
      artifact?.kind === "screenshot" && seededArtifactId === artifact.id
        ? artifact
        : null,
    seenRef: seenScreenshotItemIdsRef,
    setWorkspace: setScreenshotOutput,
    workspace: screenshotOutput,
  });

  // Keep the workspace name aligned with the latest capture suggestion.
  useEffect(() => {
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setFileStem(artifact?.suggestedFileStem ?? "");
  }, [artifact?.suggestedFileStem]);

  const report = (action: string) => (cause: unknown) => {
    console.error(`Could not ${action} the export`, cause);
    failExport(cause instanceof Error ? cause.message : String(cause));
    setIsSaving(false);
    setIsCancelingSave(false);
    saveProgress.reset();
  };

  return (
    <EditorEditGestureContext value={editGesture}>
      <EditorPanel
        {...openProject}
        allFormats={formats.savesAll}
        artifact={artifact}
        audioTrackVolumes={currentAudioTrackVolumes}
        bakeCamera={effectiveBakeCamera}
        cameraCompression={cameraCompression}
        cameraKey={cameraKey}
        cameraOverlay={cameraOverlay}
        cameraResolutionScalePercent={cameraResolutionScalePercent}
        collapseAudio={collapseAudio}
        compression={compression}
        cursorEffects={cursorEffects}
        deleteProjectAfterExport={deleteProjectAfterExport}
        directory={directory}
        enabledAudioTrackCount={enabledStreamIndices?.length ?? 0}
        enabledStreamIndices={enabledStreamIndices ?? undefined}
        enabledVideoTracks={enabledVideoTracks}
        exportCompare={exportCompare}
        fileStem={fileStem}
        frameFormats={formats.control}
        isCancelingSave={isCancelingSave}
        isPreparingRecordingAudio={isPreparingRecordingPreview}
        isPreparingRecordingPreview={isPreparingRecordingPreview}
        isPreviewPreparing={isPreparingRecordingPreview}
        isSaving={isSaving}
        keyboardEffects={keyboardEffects}
        lookQualities={lookQualities}
        onAllFormatsChange={formats.setSavesAll}
        onBakeCameraChange={setBakeCamera}
        onBrowse={() => {
          browseExportDirectory()
            .then(async (chosen) => {
              if (chosen) await setExportDirectory(chosen);
            })
            .catch(report("choose a folder for"));
        }}
        onCameraCompressionChange={(value) => {
          const next = Math.round(value);
          setCameraCompression(next);
          if (next === 0) setCameraResolutionScalePercent(100);
          clearExportFailure();
        }}
        onCameraKeyChange={setCameraKey}
        onCameraOverlayChange={formats.editCamera}
        onCameraResolutionScaleChange={(scale) => {
          setCameraResolutionScalePercent(scale);
          if (scale < 100 && cameraCompression === 0) {
            setCameraCompression(1);
          }
          clearExportFailure();
        }}
        onCancel={closeEditorWindow}
        onCancelSave={() => {
          cancelActiveExport(setIsCancelingSave);
        }}
        onCanvasResize={setScreenshotOutput}
        onCollapseAudioChange={setCollapseAudio}
        onCompressionChange={(value) => {
          const next = Math.round(value);
          setCompression(next);
          if (next === 0) setResolutionScalePercent(originalResolutionScale);
          clearExportFailure();
        }}
        onCopy={() => {
          copyEditorToClipboard(
            screenshotOutput,
            deleteProjectAfterExport,
          ).catch(report("copy"));
        }}
        onCursorEffectsChange={setCursorEffects}
        onDeleteProjectAfterExportChange={setDeleteProjectAfterExport}
        onEnabledTracksChange={onEnabledTracksChange}
        onEnabledVideoTracksChange={(tracks) => {
          if (artifactId === undefined) return;
          setVideoTrackSelection({ artifactId, tracks });
        }}
        onFileStemChange={(value) => {
          setFileStem(value);
          clearExportFailure();
        }}
        onKeyboardEffectsChange={setKeyboardEffects}
        onMinimize={minimizeEditorWindow}
        onRecordingFormatChange={chooseRecordingFormat}
        onRecordingOutputChange={(trackId, settings) => {
          formats.editOutput(trackId, settings);
          clearExportFailure();
        }}
        onRecordingTimelineEditChange={formats.timeline.commit}
        onResolutionScaleChange={(scale) => {
          setResolutionScalePercent(scale);
          if (scale < originalResolutionScale && compression === 0) {
            setCompression(1);
          }
          clearExportFailure();
        }}
        onSave={() => {
          setIsSaving(true);
          setIsCancelingSave(false);
          clearExportFailure();
          const jobs = formats.saveJobs(fileStem).map((job) => ({
            ...job,
            plan: recordingSavePlan({
              ...exportInputs,
              ...job,
              collapseAudio,
              originalResolutionScale,
            }),
          }));
          saveProgress.begin(
            jobs.some(({ plan }) => plan.showsMeasuredProgress),
          );
          saveFormats(fileStem, jobs, (job) => ({
            ...job.plan.options,
            cameraKey,
            deleteProjectAfterExport,
            fileStem: job.fileStem,
            ...recordingFormatPayload(),
            screenshotFormat: screenshotExport.format,
            screenshotFullColour: screenshotExport.fullColour,
            screenshotOutput,
            subtitles,
            timelineEdit: job.timelineEdit,
          }))
            .then((file) => {
              saveProgress.complete();
              setIsCancelingSave(false);
              setIsSaving(false);
              if (file) finishExport(file);
              else saveProgress.reset();
            })
            .catch(report("save"));
        }}
        onScreenshotFormatChange={screenshotExport.setFormat}
        onScreenshotFullColourChange={screenshotExport.setFullColour}
        onScreenshotOutputChange={(settings, itemId) => {
          const targetItemId = itemId ?? selectedScreenshotItemId;
          setScreenshotOutput((current) =>
            withScreenshotWorkspaceItemOutput(current, settings, targetItemId),
          );
          clearExportFailure();
        }}
        onSelectedScreenshotItemChange={setSelectedScreenshotItemId}
        onSelectedTrackChange={(trackId) => {
          if (artifactId === undefined) return;
          setSelectedTrack({ artifactId, trackId });
        }}
        onSubtitlesChange={setSubtitles}
        onToggleMaximize={toggleEditorWindowMaximized}
        onTrackVolumeChange={(streamIndex, decibels) => {
          if (artifactId === undefined) return;
          const values = withTrackVolume(
            currentAudioTrackVolumes,
            streamIndex,
            decibels,
          );
          setAudioTrackVolumes({ artifactId, values });
        }}
        recordingFormat={recordingFormat}
        recordingOutput={recordingOutput}
        recordingPreviewError={recordingPreviewError}
        recordingPreviewTracks={recordingPreviewTracks}
        recordingTimelineEdit={formats.timeline.edit}
        resolutionScalePercent={resolutionScalePercent}
        screenshotFormat={screenshotExport.format}
        screenshotFullColour={screenshotExport.fullColour}
        screenshotOutput={screenshotOutput}
        selectedScreenshotItemId={selectedScreenshotItemId}
        selectedTrack={selectedTrackId}
        subtitles={subtitles}
        unavailableScreenshotFormats={screenshotExport.unavailable}
      />
    </EditorEditGestureContext>
  );
}
