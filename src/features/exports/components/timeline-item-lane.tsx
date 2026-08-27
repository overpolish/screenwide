// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { RecordingTimelineEdit } from "../recording-timeline-edit";

import { TimedLaneFragment, TimedLaneItem } from "./timed-lane-layout";
import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { useTimedLaneRows } from "./use-timed-lane-rows";

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
  const rowHeightPx = 32;

  return (
    <div className="flex items-center gap-2">
      <div
        className={`flex h-8 w-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-1.25rem)] shrink-0 items-center gap-2 rounded px-2 text-xs font-medium text-content-fg ${selectedFragmentIds?.size ? "bg-info/15" : ""}`}
      >
        <span className="shrink-0 text-muted">{icon}</span>
        <span className="min-w-0 grow truncate">{label}</span>
      </div>
      <div
        className="relative min-w-0 grow overflow-hidden rounded-sm bg-muted/8 transition-[height] duration-200"
        onClick={onClearSelection}
        style={{ height: rowCount * rowHeightPx }}
      >
        <TimelineViewportContent viewport={viewport}>
          {fragments.map((fragment) => {
            const { fragmentId, item, outputEnd, outputStart, row } = fragment;
            const selected = selectedFragmentIds?.has(fragmentId) ?? false;
            const warning = warningFragmentIds?.has(fragmentId) ?? false;
            // A segment split inside one item renders as a joined run: the
            // seam keeps a single hairline divider and the label appears
            // once, on the run's widest fragment. Run members also waive the
            // minimum width and label padding - the run as a whole stays
            // clickable, and an inflated sliver would overlap the fragment
            // it continues into.
            const inRun = fragment.continuesPrevious || fragment.continuedByNext;
            const seam = `${
              fragment.continuesPrevious ? "rounded-l-none border-l-0 " : ""
            }${fragment.continuedByNext ? "rounded-r-none " : ""}${
              inRun ? "" : "min-w-1.5 "
            }${fragment.showLabel ? "px-1.5" : ""}`;
            return (
              <button
                aria-label={item.label}
                aria-pressed={selectedFragmentIds ? selected : undefined}
                className={`absolute overflow-hidden rounded-sm border text-left text-[10px] leading-5 whitespace-nowrap transition-[left,width,top,color,background-color,border-color] duration-200 ${seam} ${
                  warning
                    ? selected
                      ? "border-warning bg-warning/35 text-content-fg shadow-[inset_0_0_0_1px] shadow-warning"
                      : "border-warning/50 bg-warning/20 text-content-fg/90"
                    : selected
                      ? "border-info bg-info/40 text-content-fg shadow-[inset_0_0_0_1px] shadow-info"
                      : "border-info/35 bg-info/18 text-content-fg/90"
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
                  height: rowHeightPx - 8,
                  left: `${(outputStart * 100).toString()}%`,
                  minWidth: inRun ? undefined : minimumItemWidthPx,
                  top: row * rowHeightPx + 4,
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
