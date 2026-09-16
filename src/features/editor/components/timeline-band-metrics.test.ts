// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { TIMED_LANE_ROW_HEIGHT_PX } from "./timed-lane-layout";
import {
  CONTROL_GAP_PX,
  CONTROL_INSET_PX,
  clampTimelineHeight,
  fitTimelineHeight,
  TIMELINE_HEADER_BLOCK_HEIGHT_PX,
  TIMELINE_MIN_HEIGHT_PX,
  TIMELINE_PLAYBACK_ROW_HEIGHT_PX,
  timelineLanesHeight,
  timelineMaximumHeight,
  timelineMeterHeight,
} from "./timeline-band-metrics";

/** A measured transport row, taller than the token fallback. */
const headerHeight = 44;

describe("the height a lanes block asks the band for", () => {
  it("counts the fixed ruler row above the rows that scroll", () => {
    expect(timelineLanesHeight(200)).toBe(
      200 + TIMELINE_HEADER_BLOCK_HEIGHT_PX,
    );
  });

  it("is the whole band once the transport row is added to it", () => {
    const rowsHeight = 2 * TIMED_LANE_ROW_HEIGHT_PX + CONTROL_GAP_PX;
    expect(
      fitTimelineHeight({
        contentHeight: timelineLanesHeight(rowsHeight + CONTROL_INSET_PX),
        headerHeight: TIMELINE_PLAYBACK_ROW_HEIGHT_PX,
        maximum: 520,
      }),
    ).toBe(TIMELINE_MIN_HEIGHT_PX);
  });
});

describe("the band's minimum height", () => {
  it("lands near two lanes' worth of band", () => {
    // Pinned so the sum below cannot drift silently when a token moves. The
    // Windows skin's 2rem rows make the minimum 160; macOS's 1.5rem rows make
    // 128. Both keep the transport, the ruler and two lane rows on screen.
    expect(TIMELINE_MIN_HEIGHT_PX).toBe(
      /Windows/i.test(navigator.userAgent) ? 160 : 128,
    );
  });

  it("is the sum of the rows that must stay visible", () => {
    expect(TIMELINE_MIN_HEIGHT_PX).toBe(
      TIMELINE_PLAYBACK_ROW_HEIGHT_PX +
        TIMELINE_HEADER_BLOCK_HEIGHT_PX +
        2 * TIMED_LANE_ROW_HEIGHT_PX +
        CONTROL_GAP_PX +
        CONTROL_INSET_PX,
    );
  });

  it("leaves room for the transport, the ruler and two lanes", () => {
    const lanes =
      TIMELINE_MIN_HEIGHT_PX -
      TIMELINE_PLAYBACK_ROW_HEIGHT_PX -
      TIMELINE_HEADER_BLOCK_HEIGHT_PX -
      CONTROL_INSET_PX;
    expect(lanes).toBe(2 * TIMED_LANE_ROW_HEIGHT_PX + CONTROL_GAP_PX);
  });

  it("stops a smaller height, and a larger one at the maximum", () => {
    expect(clampTimelineHeight(10, 400)).toBe(TIMELINE_MIN_HEIGHT_PX);
    expect(clampTimelineHeight(900, 400)).toBe(400);
    expect(clampTimelineHeight(220, 400)).toBe(220);
  });
});

describe("fitting the band to its content", () => {
  it("is the transport row plus the whole block under it", () => {
    expect(
      fitTimelineHeight({ contentHeight: 300, headerHeight, maximum: 520 }),
    ).toBe(300 + headerHeight);
  });

  it("counts the transport row as it is measured, not as it is guessed", () => {
    const grown = fitTimelineHeight({
      contentHeight: 300,
      headerHeight: headerHeight + 12,
      maximum: 520,
    });
    expect(
      grown -
        fitTimelineHeight({ contentHeight: 300, headerHeight, maximum: 520 }),
    ).toBe(12);
  });

  it("is clamped exactly like a drag", () => {
    expect(
      fitTimelineHeight({ contentHeight: 4000, headerHeight, maximum: 520 }),
    ).toBe(520);
    expect(
      fitTimelineHeight({ contentHeight: 0, headerHeight, maximum: 520 }),
    ).toBe(TIMELINE_MIN_HEIGHT_PX);
  });
});

describe("timelineMaximumHeight", () => {
  it("stops at the transport row plus the block, so no empty band opens", () => {
    expect(
      timelineMaximumHeight({
        contentHeight: 300,
        headerHeight,
        spaceHeight: 900,
      }),
    ).toBe(300 + headerHeight);
  });

  it("lets a fit at the maximum show the whole timeline", () => {
    const contentHeight = 300;
    const maximum = timelineMaximumHeight({
      contentHeight,
      headerHeight,
      spaceHeight: 900,
    });
    expect(fitTimelineHeight({ contentHeight, headerHeight, maximum })).toBe(
      maximum,
    );
  });

  it("yields to the space when the block is taller than it", () => {
    expect(
      timelineMaximumHeight({
        contentHeight: 2000,
        headerHeight,
        spaceHeight: 400,
      }),
    ).toBe(400);
  });

  it("never drops below the minimum, however little there is to show", () => {
    expect(
      timelineMaximumHeight({
        contentHeight: 10,
        headerHeight,
        spaceHeight: 20,
      }),
    ).toBe(TIMELINE_MIN_HEIGHT_PX);
  });

  it("yields to the space while the block is still unmeasured", () => {
    expect(
      timelineMaximumHeight({
        contentHeight: 0,
        headerHeight,
        spaceHeight: 400,
      }),
    ).toBe(400);
  });
});

describe("timelineMeterHeight", () => {
  it("stops where the rows' own bottom inset begins", () => {
    expect(timelineMeterHeight(200)).toBe(200 - CONTROL_INSET_PX);
  });

  it("keeps a row's worth however little of the band is left", () => {
    expect(timelineMeterHeight(4)).toBe(TIMED_LANE_ROW_HEIGHT_PX);
  });
});
