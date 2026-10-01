// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { selectTimelineItem } from "./timeline-item-selection";

describe("timeline item selection", () => {
  it("replaces the selection on an ordinary click", () => {
    expect(selectTimelineItem(new Set([1, 2]), 3, false)).toEqual(new Set([3]));
  });

  it("toggles one item without disturbing the rest", () => {
    expect(selectTimelineItem(new Set([1, 2]), 2, true)).toEqual(new Set([1]));
    expect(selectTimelineItem(new Set([1]), 2, true)).toEqual(new Set([1, 2]));
  });
});
