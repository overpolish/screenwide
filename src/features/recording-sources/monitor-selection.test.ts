// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  findCurrentMonitor,
  orderMonitorsForNavigation,
} from "./monitor-selection";
import { MonitorDetails } from "./types";

const monitor = (
  id: number,
  name: string,
  overrides: Partial<MonitorDetails> = {},
): MonitorDetails => ({
  id,
  isBuiltin: false,
  isPrimary: false,
  layoutPosition: { x: 0, y: 0 },
  layoutSize: { height: 1080, width: 1920 },
  name,
  physicalPosition: { x: 0, y: 0 },
  physicalSize: { height: 1080, width: 1920 },
  position: { x: 0, y: 0 },
  scaleFactor: 1,
  size: { height: 1080, width: 1920 },
  ...overrides,
});

describe("findCurrentMonitor", () => {
  it("returns refreshed details for the selected capture target", () => {
    const selected = monitor(2, "External");
    const refreshed = monitor(2, "External", { scaleFactor: 2 });

    expect(findCurrentMonitor([refreshed], selected)).toBe(refreshed);
  });

  it("recognizes the same display when its capture id changes", () => {
    const selected = monitor(2, "External");
    const reconnected = monitor(7, "External");

    expect(findCurrentMonitor([reconnected], selected)).toBe(reconnected);
  });

  it("falls back to the primary display when selection disappears", () => {
    const selected = monitor(2, "External");
    const primary = monitor(1, "Built-in", { isPrimary: true });

    expect(findCurrentMonitor([primary], selected)).toBe(primary);
  });
});

describe("orderMonitorsForNavigation", () => {
  it("orders a stacked layout from top to bottom", () => {
    const bottom = monitor(1, "Built-in", {
      layoutPosition: { x: 0, y: 1080 },
    });
    const top = monitor(2, "External", {
      layoutPosition: { x: 0, y: 0 },
    });

    expect(orderMonitorsForNavigation([bottom, top], "vertical")).toEqual([
      top,
      bottom,
    ]);
  });

  it("orders a side-by-side layout from left to right", () => {
    const right = monitor(1, "Built-in", {
      layoutPosition: { x: 1920, y: 0 },
    });
    const left = monitor(2, "External", {
      layoutPosition: { x: 0, y: 0 },
    });

    expect(orderMonitorsForNavigation([right, left], "horizontal")).toEqual([
      left,
      right,
    ]);
  });
});
