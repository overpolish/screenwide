// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void;
};
