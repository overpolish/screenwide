// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clapperboard } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { SeekHandler } from "../../timeline/timeline-seek";
import { TimelineViewportState } from "../../timeline/timeline-viewport";
import { TimedSpanStrip } from "../../timeline/tracks/timed-span-strip";
import { TimelineTrackHeader } from "../../timeline/tracks/timeline-track-header";

import { recordingSceneLabel } from "./recording-scene-labels";
import {
  RECORDING_SCENE_MINIMUM_MS,
  RecordingSceneClip,
  sceneNeedsCamera,
} from "./recording-scenes";

/**
 * The lane the recording's scenes sit in, above every track. A click chooses
 * a scene and takes up the Scene tool with the playhead parked in the middle
 * of it, so the panel shows that scene; Cmd or Ctrl adds a scene to the
 * choice or takes it away, and a band drawn from empty lane chooses every
 * scene it touches. Its body slides it in time and its edges trim it, as
 * `TimedSpanStrip` describes. The header opens the Scene panel, where scenes
 * are added. A scene that places the camera is dimmed while the camera is
 * not baked in: it is kept and can still be moved, but nothing draws it until
 * one is chosen in the panel again.
 */
export function RecordingSceneLane({
  clips,
  edit,
  isPaused = false,
  onActivate,
  onChange,
  onClearSelection,
  onDraftChange,
  onOpenPanel,
  onSeek,
  onSelect,
  onSelectSwept,
  selectedIds,
  sourceDurationMs,
  viewport,
}: {
  clips: RecordingSceneClip[];
  edit: RecordingTimelineEdit;
  /** A scene was clicked rather than dragged: take up the Scene tool. */
  onActivate: () => void;
  onChange: (clips: RecordingSceneClip[]) => void;
  /** Let every scene go, from a click on empty lane. */
  onClearSelection: () => void;
  /** Choose the scene `id` alone, or with `toggle`, add or take it away. */
  onSelect: (id: string, toggle: boolean) => void;
  /** Choose the scenes a band swept over, alone or added to the choice. */
  onSelectSwept: (ids: string[], additive: boolean) => void;
  selectedIds: ReadonlySet<string>;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  /** Whether the camera is not baked in, which leaves the scenes that place
   * it idle. */
  isPaused?: boolean;
  /** Shows a drag's draft in the preview, or the committed clips for null. */
  onDraftChange?: (clips: RecordingSceneClip[] | null) => void;
  onOpenPanel?: () => void;
  onSeek?: SeekHandler;
}) {
  return (
    <div className="flex items-center gap-section">
      <TimelineTrackHeader
        icon={<Clapperboard />}
        label={t("editor-timeline-scenes")}
        onSelect={onOpenPanel}
      />
      <TimedSpanStrip
        className="rounded-control bg-fill-tertiary"
        clips={clips}
        edit={edit}
        isIdle={(clip) => isPaused && sceneNeedsCamera(clip)}
        label={recordingSceneLabel}
        minimumMs={RECORDING_SCENE_MINIMUM_MS}
        onActivate={onActivate}
        onChange={onChange}
        onClearSelection={onClearSelection}
        onDraftChange={onDraftChange}
        onSeek={onSeek}
        onSelect={onSelect}
        onSelectSwept={onSelectSwept}
        selectedIds={selectedIds}
        sourceDurationMs={sourceDurationMs}
        viewport={viewport}
      />
    </div>
  );
}
