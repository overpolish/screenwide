// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingCameraPlacement } from "../../recording/annotations/recording-annotation-layers";
import { RecordingAnnotationClip } from "../../recording/annotations/recording-annotations";
import { AnnotationClipPinning } from "../../recording/annotations/use-annotation-clip-menu";
import { RecordingPinStatus } from "../../recording/annotations/use-recording-pin-status";
import { RecordingSceneClip } from "../../recording/scenes/recording-scenes";
import {
  PreparedAudioTrack,
  RecordingKeyboardTimelineItem,
  RecordingPreviewLayout,
  RecordingTimelineThumbnails,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../../types";

import type { TimelineItemSelection } from "./timeline-item-selection";
import type { AudioTrackVolumes } from "../audio/audio-level";
import type { TimelineBladeController } from "../editing/timeline-blade";
import type { RecordingMoment } from "../moments/recording-moments";
import type { Playhead } from "../scrub-playhead";
import type { SeekHandler } from "../timeline-seek";

export type RecordingTrackLanesProps = {
  adjustedKeyboardFragmentIds: ReadonlySet<string>;
  audioTracks: PreparedAudioTrack[];
  blade: TimelineBladeController;
  durationMs: number;
  enabledTracks: Set<number>;
  enabledVideoTracks: Set<RecordingVideoTrackId>;
  hiddenKeyboardFragmentIds: ReadonlySet<string>;
  hiddenKeyboardItemIds: ReadonlySet<number>;
  keyboardItems: RecordingKeyboardTimelineItem[];
  keyboardSelection: TimelineItemSelection<string>;
  layout: RecordingPreviewLayout;
  onEnabledTracksChange: (tracks: Set<number>) => void;
  onEnabledVideoTracksChange: (tracks: Set<RecordingVideoTrackId>) => void;
  onSeek: SeekHandler;
  onSelectedTrackChange: (trackId: RecordingTrackId) => void;
  playhead: Playhead;
  selectedTrack: RecordingTrackId | null;
  sourceDurationMs: number;
  thumbnails: RecordingTimelineThumbnails;
  volumes: AudioTrackVolumes;
  /** Where One video draws the screen and the camera at the playhead, which
   * an annotation clip's menu moves its annotation between. */
  annotationCameraPlacement?: RecordingCameraPlacement | null;
  annotationClips?: RecordingAnnotationClip[];
  /** How each pinned annotation's path is coming along, by annotation id. */
  annotationPinStatus?: ReadonlyMap<string, RecordingPinStatus>;
  /** What an annotation clip's menu does to its pin. Without it, the lane
   * offers no pinning. */
  annotationPinning?: AnnotationClipPinning;
  /** Whether the camera is saved as a file of its own rather than drawn into
   * the screen's video, which its row says beside its name. */
  isCameraSeparate?: boolean;
  /** The moments placed while recording, pinned over the ruler. */
  moments?: readonly RecordingMoment[];
  /** Choose an annotation alone, or with `toggle`, add or take it away. */
  onAnnotationSelect?: (id: string, toggle: boolean) => void;
  onAnnotationsChange?: (clips: RecordingAnnotationClip[]) => void;
  /** Let every annotation go, from a click on empty annotation lane. */
  onAnnotationsClear?: () => void;
  onAnnotationsPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  /** Choose what a band over the annotation lane swept, alone or added. */
  onAnnotationsSweep?: (ids: string[], additive: boolean) => void;
  /** A scene clicked rather than dragged: take up the Scene tool. The lane
   * has already parked the playhead in the middle of it. */
  onSceneActivate?: () => void;
  /** Open the Scene panel, from the lane's header. */
  onScenePanelOpen?: () => void;
  /** Choose a scene alone, or with `toggle`, add or take it away. */
  onSceneSelect?: (id: string, toggle: boolean) => void;
  /** Without it the recording has nothing to arrange, and the lane is not
   * drawn. */
  onScenesChange?: (clips: RecordingSceneClip[]) => void;
  /** Let every scene go, from a click on empty scene lane. */
  onScenesClear?: () => void;
  /** Shows a scene drag's draft in the preview while it lasts, or the
   * committed clips again for null. */
  onScenesDraftChange?: (clips: RecordingSceneClip[] | null) => void;
  /** Choose what a band over the scene lane swept, alone or added. */
  onScenesSweep?: (ids: string[], additive: boolean) => void;
  /** Picking a keyboard shortcut puts it in hand: the caller clears the
   * annotation selection and takes up the Select tool, mirroring what the
   * annotation lane's select does. */
  onSelectKeyboardShortcut?: () => void;
  sceneClips?: RecordingSceneClip[];
  /** Whether the camera is not baked in, which leaves the scenes idle. */
  scenesPaused?: boolean;
  selectedAnnotationIds?: ReadonlySet<string>;
  selectedSceneIds?: ReadonlySet<string>;
};

/**
 * Where the lanes begin, for an overlay that has to line up with them: the
 * window's own inset, which the rows carry rather than the scroll area around
 * them, plus the gutter column and the section gap that separates it from the
 * lanes.
 */
export const TIMELINE_LANE_LEFT_CLASS =
  "left-[calc(var(--spacing-window-inset)+var(--spacing-timeline-gutter)+var(--spacing-section))]";
