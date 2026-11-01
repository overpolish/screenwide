// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { useCameraBackgroundRanges } from "../../recording/camera/use-camera-background-ranges";
import { useRecordingCaptions } from "../../recording/captions/use-recording-captions";
import { OverlayLayerId } from "../../recording/preview/overlay-layers";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { usePauseForMomentNotes } from "../moments/use-note-playback";
import { useRecordingMoments } from "../moments/use-recording-moments";
import { overlayPress } from "../overlay-press";
import { type Playhead } from "../scrub-playhead";

import type { RecordingKeyboardTimelineItem } from "../../types";
import type { TimelineItemSelection } from "../tracks/timeline-item-selection";

// What the timeline under the picture keeps running besides what it draws:
// the captions made for the preview to draw, the camera's background ranges
// for the camera's island, the pause a voice note asks for, and what a
// press on an overlay in the picture chooses.

export function useRecordingStripServices({
  artifactId,
  captionSelection,
  durationMs,
  edit,
  getPositionMs,
  keyboard,
  onChoose,
  onEdit,
  openOverlayTool,
  output,
  pause,
  playhead,
  pressOverlayRef,
  timelineDurationMs,
}: {
  artifactId: number;
  captionSelection: TimelineItemSelection<string>;
  /** How long the recording was recorded, in ms. */
  durationMs: number;
  edit: RecordingTimelineEdit;
  getPositionMs: () => number;
  keyboard: {
    hiddenFragmentIds: ReadonlySet<string>;
    hiddenItemIds: ReadonlySet<number>;
    items: RecordingKeyboardTimelineItem[];
    selection: TimelineItemSelection<string>;
  };
  /** Lets go of what else is chosen, before an overlay is. */
  onChoose: () => void;
  onEdit: (next: RecordingTimelineEdit) => void;
  openOverlayTool: (layerId: OverlayLayerId) => void;
  /** The output canvas the captions are set in. */
  output: { height: number; width: number } | undefined;
  pause: () => void;
  playhead: Playhead;
  pressOverlayRef: RefObject<(layerId: OverlayLayerId) => void>;
  timelineDurationMs: number;
}) {
  const moments = useRecordingMoments(artifactId);
  const captions = useRecordingCaptions({ artifactId, edit, output });
  usePauseForMomentNotes(pause);
  useCameraBackgroundRanges({
    edit,
    getPositionMs,
    onEdit,
    playhead,
    sourceDurationMs: durationMs,
  });
  // A press on an overlay in the picture reads the captions and shortcuts
  // as they are now.
  pressOverlayRef.current = overlayPress({
    captionSelection,
    captions,
    edit,
    keyboard,
    onChoose,
    openOverlayTool,
    playhead,
    sourceDurationMs: durationMs,
    timelineDurationMs,
  });
  return { moments };
}
