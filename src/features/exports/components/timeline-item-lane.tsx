// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useMemo } from "react";

import { RecordingTimelineEdit } from "../recording-timeline-edit";

import {
  TimedLaneFragment,
  TimedLaneItem,
  layoutTimedLaneItems,
} from "./timed-lane-layout";
import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";

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
  const fragments = useMemo(
    () => layoutTimedLaneItems({ edit, items, sourceDurationMs }),
    [edit, items, sourceDurationMs],
  );

  return (
    <div className="flex items-center gap-2">
      <div
        className={`flex h-8 w-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-1.25rem)] shrink-0 items-center gap-2 rounded px-2 text-xs font-medium text-content-fg ${selectedFragmentIds?.size ? "bg-info/15" : ""}`}
      >
        <span className="shrink-0 text-muted">{icon}</span>
        <span className="min-w-0 grow truncate">{label}</span>
      </div>
      <div
        className="relative h-8 min-w-0 grow overflow-hidden rounded-sm bg-muted/8"
        onClick={onClearSelection}
      >
        <TimelineViewportContent viewport={viewport}>
          {fragments.map((fragment) => {
            const { fragmentId, item, outputEnd, outputStart } = fragment;
            if (
              hiddenFragmentIds?.has(fragmentId) ||
              hiddenItemIds?.has(item.id)
            )
              return null;
            const selected = selectedFragmentIds?.has(fragmentId) ?? false;
            const warning = warningFragmentIds?.has(fragmentId) ?? false;
            return (
              <button
                aria-label={item.label}
                aria-pressed={selectedFragmentIds ? selected : undefined}
                className={`absolute inset-y-1 min-w-1.5 overflow-hidden rounded-sm border px-1.5 text-left text-[10px] leading-5 whitespace-nowrap transition-colors ${
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
                  left: `${(outputStart * 100).toString()}%`,
                  minWidth: minimumItemWidthPx,
                  width: `${((outputEnd - outputStart) * 100).toString()}%`,
                }}
                title={item.label}
                type="button"
              >
                {item.label}
              </button>
            );
          })}
        </TimelineViewportContent>
      </div>
    </div>
  );
}
