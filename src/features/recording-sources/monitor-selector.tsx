// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useLayoutEffect, useRef, useState } from "react";

import { Button } from "../../components/base/button/button";
import { ButtonGroup } from "../../components/base/button-group/button-group";
import { Text } from "../../components/base/text/text";
import { cn } from "../../lib/styling";

import { orderMonitorsForNavigation } from "./monitor-selection";
import { MonitorDetails } from "./types";

type MonitorSelectorProps = {
  focusContents: boolean;
  monitors: MonitorDetails[];
  onCommit: (monitor: MonitorDetails, returnFocus: boolean) => void;
  onSelect: (monitor: MonitorDetails) => void;
  selectedMonitor: MonitorDetails | null;
  /** A still of each display by its id, drawn in its tile. A display without
   * one keeps the plain fill. */
  thumbnails?: Record<number, string>;
};

/**
 * The largest box of the given aspect ratio that fits the container, so the
 * arrangement is never squashed however the window is proportioned.
 */
function useFittedSize(
  containerRef: RefObject<HTMLDivElement | null>,
  ratio: number,
) {
  const [size, setSize] = useState<{ height: number; width: number }>();
  useLayoutEffect(() => {
    const element = containerRef.current;
    if (!element) return;
    const fit = () => {
      const { clientHeight, clientWidth } = element;
      const style = getComputedStyle(element);
      const width =
        clientWidth -
        parseFloat(style.paddingLeft) -
        parseFloat(style.paddingRight);
      const height =
        clientHeight -
        parseFloat(style.paddingTop) -
        parseFloat(style.paddingBottom);
      const boxWidth = Math.min(width, height * ratio);
      setSize({ height: boxWidth / ratio, width: boxWidth });
    };
    // ResizeObserver reports once on observe, which covers the first layout.
    const observer = new ResizeObserver(fit);
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, [containerRef, ratio]);
  return size;
}

export function MonitorSelector({
  focusContents,
  monitors,
  onCommit,
  onSelect,
  selectedMonitor,
  thumbnails = {},
}: MonitorSelectorProps) {
  const bounds = monitors.reduce(
    (current, monitor) => ({
      maxX: Math.max(
        current.maxX,
        monitor.layoutPosition.x + monitor.layoutSize.width,
      ),
      maxY: Math.max(
        current.maxY,
        monitor.layoutPosition.y + monitor.layoutSize.height,
      ),
      minX: Math.min(current.minX, monitor.layoutPosition.x),
      minY: Math.min(current.minY, monitor.layoutPosition.y),
    }),
    {
      maxX: Number.NEGATIVE_INFINITY,
      maxY: Number.NEGATIVE_INFINITY,
      minX: Number.POSITIVE_INFINITY,
      minY: Number.POSITIVE_INFINITY,
    },
  );
  const layoutWidth = bounds.maxX - bounds.minX;
  const layoutHeight = bounds.maxY - bounds.minY;
  const containerRef = useRef<HTMLDivElement>(null);
  // Hooks run before the empty-state return; an empty layout has no ratio.
  const fitted = useFittedSize(
    containerRef,
    monitors.length > 0 ? layoutWidth / layoutHeight : 1,
  );

  if (monitors.length === 0) {
    return (
      <div className="flex h-full w-full items-center justify-center rounded-panel p-control-inset">
        <Text variant="subheadline">No Displays</Text>
      </div>
    );
  }

  const orientation = layoutWidth >= layoutHeight ? "horizontal" : "vertical";
  const orderedMonitors = orderMonitorsForNavigation(monitors, orientation);
  const focusTargetId = orderedMonitors.some(
    (monitor) => monitor.id === selectedMonitor?.id,
  )
    ? selectedMonitor?.id
    : orderedMonitors[0]?.id;

  return (
    // The display arrangement as Displays settings draws it: tiles laid out
    // as the displays sit, with the chosen one on the accent.
    <div
      className="flex h-full w-full items-center justify-center overflow-hidden rounded-panel p-control-inset"
      ref={containerRef}
    >
      <ButtonGroup
        aria-label="Displays"
        className="relative"
        orientation={orientation}
        style={fitted}
      >
        {orderedMonitors.map((monitor) => {
          const isSelected = selectedMonitor?.id === monitor.id;

          return (
            <Button
              aria-label={`Select ${monitor.name}`}
              className={cn(
                "absolute h-auto min-h-8 min-w-12 transform-gpu justify-center overflow-hidden px-control",
                focusContents && monitor.id === focusTargetId && "focus:ring-3",
              )}
              color={isSelected ? "primary" : "neutral"}
              data-source-selector-focus-target={
                monitor.id === focusTargetId ? "true" : undefined
              }
              key={monitor.id}
              onPress={(event) => {
                onCommit(
                  monitor,
                  ["keyboard", "virtual"].includes(event.pointerType),
                );
              }}
              onPressStart={() => {
                onSelect(monitor);
              }}
              style={{
                height: `${String((monitor.layoutSize.height / layoutHeight) * 100)}%`,
                left: `${String(((monitor.layoutPosition.x - bounds.minX) / layoutWidth) * 100)}%`,
                top: `${String(((monitor.layoutPosition.y - bounds.minY) / layoutHeight) * 100)}%`,
                width: `${String((monitor.layoutSize.width / layoutWidth) * 100)}%`,
              }}
            >
              {/* The still is the tile: position and picture say which
                  display this is, as the Displays settings arrangement does,
                  and the name is left to the accessible label. Without a
                  still the name is all there is to show. */}
              {thumbnails[monitor.id] ? (
                <img
                  alt=""
                  className="absolute inset-0 size-full object-cover"
                  src={thumbnails[monitor.id]}
                />
              ) : (
                <span className="relative max-w-full truncate text-subheadline">
                  {monitor.name}
                </span>
              )}
            </Button>
          );
        })}
      </ButtonGroup>
    </div>
  );
}
