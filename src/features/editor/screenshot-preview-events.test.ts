// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { screenshotAnnotationChange } from "./screenshot-preview-events";

const arrow = {
  aboveCamera: false,
  id: "arrow-1",
  shape: {
    control: { x: 60, y: 40 },
    end: { x: 110, y: 60 },
    kind: "arrow",
    start: { x: 10, y: 20 },
  },
  style: { color: "#0a84ff", head: "end", width: 6 },
};

const payload = (overrides: Record<string, unknown> = {}) => ({
  annotations: [arrow],
  paneIndex: 0,
  selectedAnnotationId: "arrow-1",
  sessionId: 7,
  ...overrides,
});

describe("screenshotAnnotationChange", () => {
  it("reads a finished arrow gesture", () => {
    expect(screenshotAnnotationChange(payload(), 7)).toEqual({
      annotations: [arrow],
      paneIndex: 0,
      selectedAnnotationId: "arrow-1",
    });
  });

  it("ignores another session's gesture", () => {
    expect(screenshotAnnotationChange(payload(), 8)).toBeNull();
    expect(screenshotAnnotationChange(undefined, 7)).toBeNull();
  });

  it("ignores a payload without a layer to commit to", () => {
    expect(
      screenshotAnnotationChange(payload({ paneIndex: null }), 7),
    ).toBeNull();
  });

  it("drops a mark the compositor could not place", () => {
    const broken = {
      ...arrow,
      shape: { ...arrow.shape, end: { x: Number.NaN, y: 0 } },
    };
    const change = screenshotAnnotationChange(
      payload({ annotations: [arrow, broken] }),
      7,
    );
    expect(change?.annotations).toEqual([arrow]);
  });

  it("reads a cleared selection as nothing chosen", () => {
    const change = screenshotAnnotationChange(
      payload({ selectedAnnotationId: null }),
      7,
    );
    expect(change?.selectedAnnotationId).toBeNull();
  });
});
