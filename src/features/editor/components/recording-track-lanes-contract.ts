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

import type { AudioTrackVolumes } from "./audio-level";
import type { Playhead } from "./scrub-playhead";
import type { SeekHandler } from "./scrub-timeline";
import type { TimelineBladeController } from "./timeline-blade";
import type { TimelineItemSelection } from "./timeline-item-selection";

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
  onAnnotationSelect?: (id: string) => void;
  onAnnotationsChange?: (clips: RecordingAnnotationClip[]) => void;
  onAnnotationsPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
  selectedAnnotationId?: string | null;
};

/**
 * Where the lanes begin, for an overlay that has to line up with them: the
 * window's own inset, which the rows carry rather than the scroll area around
 * them, plus the gutter column and the section gap that separates it from the
 * lanes.
 */
export const TIMELINE_LANE_LEFT_CLASS =
  "left-[calc(var(--spacing-window-inset)+var(--spacing-timeline-gutter)+var(--spacing-section))]";
