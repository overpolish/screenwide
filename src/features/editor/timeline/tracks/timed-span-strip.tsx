// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { KeyboardEvent, PointerEvent } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { recordingTimelineLengthMs } from "../editing/recording-timeline-time";
import { SeekHandler } from "../timeline-seek";
import { TimelineViewportState } from "../timeline-viewport";
import { TimelineViewportContent } from "../timeline-viewport-content";

import {
  layoutTimelineLaneItems,
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "./timed-lane-layout";
import { resizeTimedSpanClip, TimedSpanClip } from "./timed-span-clips";
import {
  sweptLaneItems,
  togglesChoice,
  useTimelineLaneBand,
} from "./timeline-lane-band";
import { TimelineLaneBandBox } from "./timeline-lane-band-box";
import { TimedSpanDragEdge, useTimedSpanDrag } from "./use-timed-span-drag";

/** The narrowest a clip is drawn, so a short one can still be grabbed. */
const SPAN_CLIP_MINIMUM_WIDTH_PX = 12;

/** A stretch of a clip, in milliseconds into the edited video. */
export type TimedSpan = { endMs: number; startMs: number };

/** The parts of `clip` that `spans` cover, faded over its box. */
function PendingSpans({
  clip,
  spans,
}: {
  clip: TimedSpan;
  spans: readonly TimedSpan[];
}) {
  const lengthMs = clip.endMs - clip.startMs;
  if (lengthMs <= 0) return null;
  return spans.map((span) => {
    const from = Math.max(span.startMs, clip.startMs) - clip.startMs;
    const to = Math.min(span.endMs, clip.endMs) - clip.startMs;
    return to > from ? (
      <div
        aria-hidden
        className="pointer-events-none absolute inset-y-0 bg-content/30"
        key={span.startMs}
        style={{
          left: `${String((from / lengthMs) * 100)}%`,
          width: `${String(((to - from) / lengthMs) * 100)}%`,
        }}
      />
    ) : null;
  });
}

/**
 * One row of span clips, each holding its stretch of the edited video to
 * itself. A click chooses a clip and, alone, parks the playhead in the
 * middle of it and activates it; Cmd or Ctrl adds a clip to the choice or
 * takes it away, and a band drawn from empty row chooses every clip it
 * touches. Its body slides it in time, past its neighbours into whichever gap
 * it is dropped in, and its edges trim it; the arrow keys step a focused edge
 * a tenth of a second, a whole second with Shift.
 *
 * A clip marked idle is dimmed: it is kept and can still be moved. The parts
 * of a clip `pendingSpans` names are drawn faded, for work on it still to
 * come.
 */
export function TimedSpanStrip<Clip extends TimedSpanClip>({
  className,
  clips,
  edit,
  isIdle,
  label,
  minimumMs,
  onActivate,
  onChange,
  onClearSelection,
  onDraftChange,
  onSeek,
  onSelect,
  onSelectSwept,
  pendingSpans,
  selectedIds,
  sourceDurationMs,
  viewport,
}: {
  clips: Clip[];
  edit: RecordingTimelineEdit;
  label: (clip: Clip) => string;
  /** The shortest a clip may be trimmed to. */
  minimumMs: number;
  onChange: (clips: Clip[]) => void;
  /** Let every clip go, from a click on empty row. */
  onClearSelection: () => void;
  /** Choose the clip `id` alone, or with `toggle`, add or take it away. */
  onSelect: (id: string, toggle: boolean) => void;
  /** Choose the clips a band swept over, alone or added to the choice. */
  onSelectSwept: (ids: string[], additive: boolean) => void;
  selectedIds: ReadonlySet<string>;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  className?: string;
  isIdle?: (clip: Clip) => boolean;
  /** A clip was clicked rather than dragged. */
  onActivate?: (clip: Clip) => void;
  /** Shows a drag's draft in the preview, or the committed clips for null. */
  onDraftChange?: (clips: Clip[] | null) => void;
  onSeek?: SeekHandler;
  pendingSpans?: (clip: Clip) => readonly TimedSpan[];
}) {
  const { beginDrag, draft, laneRef, movedRef } = useTimedSpanDrag({
    clips,
    edit,
    minimumMs,
    onCommit: onChange,
    onDraftChange,
    onSeek,
    sourceDurationMs,
    viewport,
  });
  const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
  const fragments = layoutTimelineLaneItems(draft ?? clips, lengthMs);
  const { band, pressLane } = useTimelineLaneBand({
    laneRef,
    onClear: onClearSelection,
    onSweep: onSelectSwept,
    sweep: (box, laneWidthPx) =>
      sweptLaneItems(fragments, box, {
        laneWidthPx,
        minimumWidthPx: SPAN_CLIP_MINIMUM_WIDTH_PX,
        viewport,
      }),
  });
  // While a band is drawn, the row shows the choice it will make.
  const isSelected = (id: string) =>
    band
      ? band.ids.has(id) || (band.additive && selectedIds.has(id))
      : selectedIds.has(id);
  const press =
    (id: string, edge: TimedSpanDragEdge) => (event: PointerEvent) => {
      if (event.button !== 0) return;
      event.stopPropagation();
      // A press with the toggle held is a click to come, never a drag.
      if (edge === "body" && togglesChoice(event)) return;
      beginDrag({ clientX: event.clientX, edge, id });
    };
  const activate = (clip: Clip) => {
    if (lengthMs > 0)
      onSeek?.((clip.startMs + clip.endMs) / 2 / lengthMs, "end");
    onActivate?.(clip);
  };
  const nudge =
    (clip: Clip, edge: "endMs" | "startMs") => (event: KeyboardEvent) => {
      if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
      event.preventDefault();
      event.stopPropagation();
      if (lengthMs <= 0) return;
      const step = (event.shiftKey ? 1000 : 100) / lengthMs;
      onChange(
        resizeTimedSpanClip({
          clips,
          edge,
          edit,
          id: clip.id,
          minimumMs,
          output:
            clip[edge] / lengthMs + (event.key === "ArrowLeft" ? -step : step),
          sourceDurationMs,
        }),
      );
    };
  return (
    <div
      className={cn("relative min-w-0 grow overflow-hidden", className)}
      // Every press on a clip stops here, so what arrives is empty row.
      onPointerDown={pressLane}
      ref={laneRef}
      style={{ height: TIMED_LANE_ROW_HEIGHT_PX }}
    >
      <TimelineViewportContent viewport={viewport}>
        {fragments.map((fragment) => {
          const clip = fragment.item;
          const name = label(clip);
          const selected = isSelected(clip.id);
          return (
            <div
              className={cn(
                "absolute overflow-hidden rounded-control text-footnote transition-opacity",
                selected
                  ? "bg-primary-surface text-primary-fg"
                  : "bg-fill-secondary text-content-fg",
                isIdle?.(clip) && "opacity-50",
              )}
              key={fragment.fragmentId}
              style={{
                ...timedLaneFragmentBox(0),
                left: `${String(fragment.outputStart * 100)}%`,
                minWidth: SPAN_CLIP_MINIMUM_WIDTH_PX,
                width: `${String((fragment.outputEnd - fragment.outputStart) * 100)}%`,
              }}
            >
              {pendingSpans ? (
                <PendingSpans clip={clip} spans={pendingSpans(clip)} />
              ) : null}
              <button
                aria-label={name}
                aria-pressed={selected}
                className="relative h-full w-full truncate px-control-inset text-left focus-visible:outline-2 focus-visible:outline-primary"
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
                {name}
              </button>
              {(["startMs", "endMs"] as const).map((edge) => (
                <button
                  aria-label={
                    edge === "startMs"
                      ? t("editor-clip-start", { clip: name })
                      : t("editor-clip-end", { clip: name })
                  }
                  className={`absolute inset-y-0 w-control-inset cursor-ew-resize focus-visible:bg-primary focus-visible:outline-none ${edge === "startMs" ? "left-0" : "right-0"}`}
                  key={edge}
                  onClick={(event) => {
                    event.stopPropagation();
                  }}
                  onKeyDown={nudge(clip, edge)}
                  onPointerDown={press(clip.id, edge)}
                  type="button"
                />
              ))}
            </div>
          );
        })}
      </TimelineViewportContent>
      {band ? <TimelineLaneBandBox box={band.box} /> : null}
    </div>
  );
}
