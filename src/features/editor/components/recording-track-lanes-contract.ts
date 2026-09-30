// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingAnnotationClip } from "../recording-annotations";
import {
  PreparedAudioTrack,
  RecordingKeyboardTimelineItem,
  RecordingPreviewLayout,
  RecordingTimelineThumbnails,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";
import { RecordingPinStatus } from "../use-recording-pin-status";

import { AnnotationClipPinning } from "./use-annotation-clip-menu";

import type { AudioTrackVolumes } from "./audio-level";
import type { Playhead } from "./scrub-playhead";
import type { TimelineBladeController } from "./timeline-blade";
import type { TimelineItemSelection } from "./timeline-item-selection";
import type { SeekHandler } from "./timeline-seek";

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
  videoTrackOrder: RecordingVideoTrackId[];
  volumes: AudioTrackVolumes;
  annotationClips?: RecordingAnnotationClip[];
  /** How each pinned annotation's path is coming along, by annotation id. */
  annotationPinStatus?: ReadonlyMap<string, RecordingPinStatus>;
  /** What an annotation clip's menu does to its pin. Without it, the lane
   * offers no pinning. */
  annotationPinning?: AnnotationClipPinning;
  /** Choose an annotation alone, or with `toggle`, add or take it away. */
  onAnnotationSelect?: (id: string, toggle: boolean) => void;
  onAnnotationsChange?: (clips: RecordingAnnotationClip[]) => void;
  /** Let every annotation go, from a click on empty annotation lane. */
  onAnnotationsClear?: () => void;
  onAnnotationsPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  /** Choose what a band over the annotation lane swept, alone or added. */
  onAnnotationsSweep?: (ids: string[], additive: boolean) => void;
  /** Picking a keyboard shortcut puts it in hand: the caller clears the
   * annotation selection and takes up the Select tool, mirroring what the
   * annotation lane's select does. */
  onSelectKeyboardShortcut?: () => void;
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
  selectedAnnotationIds?: ReadonlySet<string>;
};

/**
 * Where the lanes begin, for an overlay that has to line up with them: the
 * window's own inset, which the rows carry rather than the scroll area around
 * them, plus the gutter column and the section gap that separates it from the
 * lanes.
 */
export const TIMELINE_LANE_LEFT_CLASS =
  "left-[calc(var(--spacing-window-inset)+var(--spacing-timeline-gutter)+var(--spacing-section))]";
