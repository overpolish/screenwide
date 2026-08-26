// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  TIMELINE_MAX_ZOOM,
  clampTimelinePan,
  fitTimelineViewport,
  normalizeTimelineViewport,
  panTimelineViewportByPixels,
  resetTimelineViewport,
  timelineFractionToX,
  timelineXToFraction,
  zoomTimelineViewportAt,
} from "./timeline-viewport";

const rect = { left: 100, width: 800 };
const close = (a: number, b: number) => {
  expect(a).toBeCloseTo(b, 10);
};

describe("timeline viewport", () => {
  it("normalizes zoom and pan to their legal ranges", () => {
    expect(normalizeTimelineViewport({ panOffset: 4, zoom: -2 })).toEqual({
      panOffset: 0,
      zoom: 1,
    });
    expect(normalizeTimelineViewport({ panOffset: 4, zoom: 100 })).toEqual({
      panOffset: 1 - 1 / TIMELINE_MAX_ZOOM,
      zoom: TIMELINE_MAX_ZOOM,
    });
    expect(clampTimelinePan(-1, 4)).toBe(0);
    expect(clampTimelinePan(1, 4)).toBe(0.75);
  });

  it("round-trips fractions and viewport x coordinates", () => {
    const state = { panOffset: 0.2, zoom: 4 };
    for (const fraction of [0, 0.2, 0.5, 1]) {
      close(
        timelineXToFraction(
          timelineFractionToX(fraction, state, rect),
          state,
          rect,
        ),
        fraction,
      );
    }
  });

  it("keeps the cursor anchor fixed during zoom", () => {
    const state = { panOffset: 0.1, zoom: 2 };
    const cursorX = 460;
    const before = timelineXToFraction(cursorX, state, rect);
    const next = zoomTimelineViewportAt(state, { cursorX, factor: 2, rect });
    close(timelineXToFraction(cursorX, next, rect), before);
    expect(next.zoom).toBe(4);
  });

  it("clamps cursor-anchored zoom at the edges", () => {
    const next = zoomTimelineViewportAt(
      { panOffset: 0.8, zoom: 19 },
      { cursorX: 100, factor: 10, rect },
    );
    expect(next.zoom).toBe(TIMELINE_MAX_ZOOM);
    expect(next.panOffset).toBeGreaterThanOrEqual(0);
    expect(next.panOffset).toBeLessThanOrEqual(1 - 1 / TIMELINE_MAX_ZOOM);
  });

  it("pans by pixel distance and clamps at both ends", () => {
    const state = { panOffset: 0.25, zoom: 4 };
    const moved = panTimelineViewportByPixels(state, -80, rect.width);
    close(moved.panOffset, 0.275);
    expect(
      panTimelineViewportByPixels(state, 10_000, rect.width).panOffset,
    ).toBe(0);
    expect(
      panTimelineViewportByPixels(state, -10_000, rect.width).panOffset,
    ).toBe(0.75);
  });

  it("supports fit/reset and safely handles unusable geometry", () => {
    expect(resetTimelineViewport()).toEqual(fitTimelineViewport());
    const state = { panOffset: 0.2, zoom: 3 };
    expect(
      zoomTimelineViewportAt(state, {
        cursorX: 0,
        factor: 2,
        rect: { left: 0, width: 0 },
      }),
    ).toEqual(normalizeTimelineViewport(state));
    expect(panTimelineViewportByPixels(state, 10, 0)).toEqual(
      normalizeTimelineViewport(state),
    );
  });
});
