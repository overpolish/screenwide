// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { RecordingTimelineEdit } from "../recording-timeline-edit";

import {
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
  TimedLaneFragment,
  TimedLaneItem,
} from "./timed-lane-layout";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { useTimedLaneRows } from "./use-timed-lane-rows";

/**
 * Mirrors `--spacing-control`, the gap that separates every row in the timeline
 * band. A sublane badge is inset by it top and bottom so stacked rows read as
 * separate rows without a rule between them.
 */

export function TimelineItemLane<
  Item extends TimedLaneItem & { label: string },
>({
  edit,
  hiddenFragmentIds,
  hiddenItemIds,
  icon,
  items,
  label,
  minimumItemWidthPx = 6,
  onClearSelection,
  onSelect,
  selectedFragmentIds,
  sourceDurationMs,
  viewport,
  warningFragmentIds,
}: {
  edit: RecordingTimelineEdit;
  icon: ReactNode;
  items: Item[];
  label: string;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  hiddenFragmentIds?: ReadonlySet<string>;
  hiddenItemIds?: ReadonlySet<Item["id"]>;
  minimumItemWidthPx?: number;
  onClearSelection?: () => void;
  onSelect?: (
    fragment: TimedLaneFragment<Item>,
    outputPosition: number,
    toggle: boolean,
  ) => void;
  selectedFragmentIds?: ReadonlySet<string>;
  warningFragmentIds?: ReadonlySet<string>;
}) {
  // Overlapping fragments stack into sublanes so a badge that is still
  // fading stays visible beside its successor; the lane grows as needed.
  const { fragments, rowCount } = useTimedLaneRows({
    edit,
    hiddenFragmentIds,
    hiddenItemIds,
    items,
    sourceDurationMs,
  });

  return (
    <div className="flex items-center gap-section">
      <TimelineTrackHeader
        icon={icon}
        isSelected={(selectedFragmentIds?.size ?? 0) > 0}
        label={label}
      />
      <div
        className="relative min-w-0 grow overflow-hidden rounded-control bg-fill-tertiary transition-[height] duration-200"
        onClick={onClearSelection}
        style={{ height: rowCount * TIMED_LANE_ROW_HEIGHT_PX }}
      >
        <TimelineViewportContent viewport={viewport}>
          {fragments.map((fragment) => {
            const { fragmentId, item, outputEnd, outputStart, row } = fragment;
            const selected = selectedFragmentIds?.has(fragmentId) ?? false;
            const warning = warningFragmentIds?.has(fragmentId) ?? false;
            // A segment split inside one item renders as a joined run: the
            // seam drops the rounding between the pair and the label appears
            // once, on the run's widest fragment. Run members also waive the
            // minimum width and label padding - the run as a whole stays
            // clickable, and an inflated sliver would overlap the fragment
            // it continues into.
            const inRun =
              fragment.continuesPrevious || fragment.continuedByNext;
            const seam = `${
              fragment.continuesPrevious ? "rounded-l-none " : ""
            }${fragment.continuedByNext ? "rounded-r-none " : ""}${
              fragment.showLabel ? "px-control-inset" : ""
            }`;
            return (
              <button
                aria-label={item.label}
                aria-pressed={selectedFragmentIds ? selected : undefined}
                // A badge, built from the tokens the base Badge uses: footnote
                // text on a fill. An adjusted fragment keeps its translucent
                // warning fill whether or not it is selected, since text on
                // the solid yellow cannot be read; its selection is a warning
                // ring instead. Any other selected one takes the accent.
                className={`absolute flex items-center overflow-hidden rounded-control text-left text-footnote whitespace-nowrap transition-[color,background-color] duration-200 ${seam} ${
                  warning
                    ? selected
                      ? "bg-warning/35 text-content-fg inset-ring-2 inset-ring-warning"
                      : "bg-warning/35 text-content-fg"
                    : selected
                      ? "bg-primary-surface text-primary-fg"
                      : "bg-fill-secondary text-content-fg"
                } ${onSelect ? "cursor-default" : "pointer-events-none"}`}
                key={fragmentId}
                onClick={(event) => {
                  event.stopPropagation();
                  onSelect?.(
                    fragment,
                    (outputStart + outputEnd) / 2,
                    event.metaKey || event.ctrlKey,
                  );
                }}
                style={{
                  ...timedLaneFragmentBox(row),
                  left: `${(outputStart * 100).toString()}%`,
                  minWidth: inRun ? undefined : minimumItemWidthPx,
                  width: `${((outputEnd - outputStart) * 100).toString()}%`,
                }}
                title={item.label}
                type="button"
              >
                {fragment.showLabel ? item.label : null}
              </button>
            );
          })}
        </TimelineViewportContent>
      </div>
    </div>
  );
}
