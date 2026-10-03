// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clapperboard } from "lucide-react";
import { KeyboardEvent, PointerEvent } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";
import { SeekHandler } from "../../timeline/timeline-seek";
import { TimelineViewportState } from "../../timeline/timeline-viewport";
import { TimelineViewportContent } from "../../timeline/timeline-viewport-content";
import {
  layoutTimedLaneItems,
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "../../timeline/tracks/timed-lane-layout";
import {
  sweptLaneItems,
  togglesChoice,
  useTimelineLaneBand,
} from "../../timeline/tracks/timeline-lane-band";
import { TimelineLaneBandBox } from "../../timeline/tracks/timeline-lane-band-box";
import { TimelineTrackHeader } from "../../timeline/tracks/timeline-track-header";

import { recordingSceneLabel } from "./recording-scene-labels";
import {
  RecordingSceneClip,
  resizeRecordingSceneClip,
  sceneNeedsCamera,
} from "./recording-scenes";
import {
  RecordingSceneDragEdge,
  useRecordingSceneDrag,
} from "./use-recording-scene-drag";

/** The narrowest a clip is drawn, so a short one can still be grabbed. */
const SCENE_CLIP_MINIMUM_WIDTH_PX = 12;

/**
 * The lane the recording's scenes sit in, above every track. A click chooses
 * a scene and takes up the Scene tool with the playhead parked in the middle
 * of it, so the panel shows that scene; Cmd or Ctrl adds a scene to the
 * choice or takes it away, and a band drawn from empty lane chooses every
 * scene it touches. Its body slides it in time, past its neighbours into
 * whichever gap it is dropped in, and its edges trim it; the arrow keys step
 * a focused edge a tenth of a second, a whole second with Shift. A clip that
 * crosses a cut is drawn in pieces, with handles only on its true ends. The
 * header opens the Scene panel, where scenes are added. A scene that places
 * the camera is dimmed while the camera is not baked in: it is kept and can
 * still be moved, but nothing draws it until one is chosen in the panel
 * again.
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
  const { beginDrag, draft, laneRef, movedRef } = useRecordingSceneDrag({
    clips,
    edit,
    onCommit: onChange,
    onDraftChange,
    onSeek,
    sourceDurationMs,
    viewport,
  });
  const fragments = layoutTimedLaneItems({
    edit,
    items: draft ?? clips,
    sourceDurationMs,
  });
  const { band, pressLane } = useTimelineLaneBand({
    laneRef,
    onClear: onClearSelection,
    onSweep: onSelectSwept,
    sweep: (box, laneWidthPx) =>
      sweptLaneItems(fragments, box, {
        laneWidthPx,
        minimumWidthPx: SCENE_CLIP_MINIMUM_WIDTH_PX,
        viewport,
      }),
  });
  // While a band is drawn, the lane shows the choice it will make.
  const isSelected = (id: string) =>
    band
      ? band.ids.has(id) || (band.additive && selectedIds.has(id))
      : selectedIds.has(id);
  const press =
    (id: string, edge: RecordingSceneDragEdge) => (event: PointerEvent) => {
      if (event.button !== 0) return;
      event.stopPropagation();
      // A press with the toggle held is a click to come, never a drag.
      if (edge === "body" && togglesChoice(event)) return;
      beginDrag({ clientX: event.clientX, edge, id });
    };
  // The middle of what the timeline keeps of the clip, so a clip that loses
  // its own middle to a cut still parks the playhead inside it.
  const activate = (clip: RecordingSceneClip) => {
    if (sourceDurationMs > 0) {
      const start = recordingTimelineSourceToOutput(
        edit,
        clip.startMs / sourceDurationMs,
      );
      const end = recordingTimelineSourceToOutput(
        edit,
        clip.endMs / sourceDurationMs,
      );
      onSeek?.((start + end) / 2, "end");
    }
    onActivate();
  };
  const nudge =
    (clip: RecordingSceneClip, edge: "endMs" | "startMs") =>
    (event: KeyboardEvent) => {
      if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
      event.preventDefault();
      event.stopPropagation();
      const step =
        (event.shiftKey ? 1000 : 100) /
        (sourceDurationMs * recordingTimelineRetainedDuration(edit));
      const output = recordingTimelineSourceToOutput(
        edit,
        clip[edge] / sourceDurationMs,
      );
      onChange(
        resizeRecordingSceneClip({
          clips,
          edge,
          edit,
          id: clip.id,
          output: output + (event.key === "ArrowLeft" ? -step : step),
          sourceDurationMs,
        }),
      );
    };
  return (
    <div className="flex items-center gap-section">
      <TimelineTrackHeader
        icon={<Clapperboard />}
        label="Scenes"
        onSelect={onOpenPanel}
      />
      <div
        className="relative min-w-0 grow overflow-hidden rounded-control bg-fill-tertiary"
        // Every press on a clip stops here, so what arrives is empty lane.
        onPointerDown={pressLane}
        ref={laneRef}
        style={{ height: TIMED_LANE_ROW_HEIGHT_PX }}
      >
        <TimelineViewportContent viewport={viewport}>
          {fragments.map((fragment, index) => {
            const clip = fragment.item;
            const label = recordingSceneLabel(clip);
            const isIdle = isPaused && sceneNeedsCamera(clip);
            const selected = isSelected(clip.id);
            const continuesPrevious = fragments[index - 1]?.item.id === clip.id;
            const continuedByNext = fragments[index + 1]?.item.id === clip.id;
            return (
              <div
                className={`absolute overflow-hidden rounded-control text-footnote transition-opacity ${selected ? "bg-primary-surface text-primary-fg" : "bg-fill-secondary text-content-fg"} ${isIdle ? "opacity-50" : ""} ${continuesPrevious ? "rounded-l-none" : ""} ${continuedByNext ? "rounded-r-none" : ""}`}
                key={fragment.fragmentId}
                style={{
                  ...timedLaneFragmentBox(0),
                  left: `${String(fragment.outputStart * 100)}%`,
                  minWidth: SCENE_CLIP_MINIMUM_WIDTH_PX,
                  width: `${String((fragment.outputEnd - fragment.outputStart) * 100)}%`,
                }}
              >
                <button
                  aria-label={label}
                  aria-pressed={selected}
                  className="h-full w-full truncate px-control-inset text-left focus-visible:outline-2 focus-visible:outline-primary"
                  // A press that slid the clip was a drag, not a click.
                  onClick={(event) => {
                    if (movedRef.current) {
                      movedRef.current = false;
                      return;
                    }
                    const toggle = togglesChoice(event);
                    onSelect(clip.id, toggle);
                    if (!toggle) activate(clip);
                  }}
                  onPointerDown={press(clip.id, "body")}
                  type="button"
                >
                  {label}
                </button>
                {(["startMs", "endMs"] as const).map((edge) =>
                  (edge === "startMs" && continuesPrevious) ||
                  (edge === "endMs" && continuedByNext) ? null : (
                    <button
                      aria-label={`${edge === "startMs" ? "Start" : "End"} of ${label}`}
                      className={`absolute inset-y-0 w-control-inset cursor-ew-resize focus-visible:bg-primary focus-visible:outline-none ${edge === "startMs" ? "left-0" : "right-0"}`}
                      key={edge}
                      onClick={(event) => {
                        event.stopPropagation();
                      }}
                      onKeyDown={nudge(clip, edge)}
                      onPointerDown={press(clip.id, edge)}
                      type="button"
                    />
                  ),
                )}
              </div>
            );
          })}
        </TimelineViewportContent>
        {band ? <TimelineLaneBandBox box={band.box} /> : null}
      </div>
    </div>
  );
}
