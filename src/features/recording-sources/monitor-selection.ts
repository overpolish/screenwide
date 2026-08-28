// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MonitorDetails } from "./types";

export type MonitorNavigationOrientation = "horizontal" | "vertical";

export const orderMonitorsForNavigation = (
  monitors: MonitorDetails[],
  orientation: MonitorNavigationOrientation,
): MonitorDetails[] => {
  const primaryAxis = orientation === "horizontal" ? "x" : "y";
  const secondaryAxis = orientation === "horizontal" ? "y" : "x";

  return [...monitors].sort(
    (left, right) =>
      left.layoutPosition[primaryAxis] - right.layoutPosition[primaryAxis] ||
      left.layoutPosition[secondaryAxis] -
        right.layoutPosition[secondaryAxis] ||
      left.id - right.id,
  );
};

export const findCurrentMonitor = (
  monitors: MonitorDetails[],
  selected: MonitorDetails | null,
): MonitorDetails | null => {
  if (selected) {
    const sameCaptureTarget = monitors.find(
      (monitor) => monitor.id === selected.id,
    );
    if (sameCaptureTarget) return sameCaptureTarget;

    const sameDisplay = monitors.find(
      (monitor) =>
        monitor.name === selected.name &&
        monitor.size.width === selected.size.width &&
        monitor.size.height === selected.size.height,
    );
    if (sameDisplay) return sameDisplay;
  }

  const primary = monitors.find((monitor) => monitor.isPrimary);
  if (primary) return primary;
  if (monitors.length === 0) return null;
  return monitors[0];
};
