// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  nearestTimelineSnapTarget,
  TimelineSnapGesture,
} from "./timeline-snap";

const gesture = (targets: number[], snapping = true): TimelineSnapGesture => ({
  isSnapping: () => snapping,
  showGuide: () => undefined,
  targets,
  threshold: 0.05,
});

describe("nearestTimelineSnapTarget", () => {
  it("takes the closest target within reach and none beyond it", () => {
    expect(nearestTimelineSnapTarget(gesture([0.2, 0.3]), 0.27)).toBe(0.3);
    expect(nearestTimelineSnapTarget(gesture([0.2, 0.3]), 0.4)).toBeNull();
  });

  it("holds nothing while snapping is inverted off", () => {
    expect(nearestTimelineSnapTarget(gesture([0.2], false), 0.2)).toBeNull();
  });
});
