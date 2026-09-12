// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { reducePreviewZoom } from "./preview-zoom-state";

describe("preview zoom reports and requests", () => {
  it("does not send delayed pinch readouts back as zoom commands", () => {
    let state = { percent: 100 };
    // The reproduction alternated older readouts with newer native samples.
    for (const percent of [65, 64, 77, 79, 78, 82, 78, 84]) {
      state = reducePreviewZoom(state, { percent, source: "native" });
      expect(state).toEqual({ percent, request: undefined });
    }
  });

  it("keeps an explicit request unchanged when native reports race with it", () => {
    const requested = reducePreviewZoom(
      { percent: 64 },
      { percent: 100, source: "control" },
    );
    const delayed = reducePreviewZoom(requested, {
      percent: 65,
      source: "native",
    });
    const acknowledged = reducePreviewZoom(delayed, {
      percent: 100,
      source: "native",
    });
    expect(delayed.percent).toBe(65);
    expect(delayed.request).toBe(requested.request);
    expect(acknowledged.request).toBe(requested.request);
    expect(acknowledged.request?.percent).toBe(100);
  });

  it("allows requesting the same percentage again after a pinch", () => {
    const first = reducePreviewZoom(
      { percent: 64 },
      { percent: 100, source: "control" },
    );
    const pinched = reducePreviewZoom(first, { percent: 75, source: "native" });
    const repeated = reducePreviewZoom(pinched, {
      percent: 100,
      source: "control",
    });
    expect(repeated.request).toEqual({ percent: 100 });
    expect(repeated.request).not.toBe(first.request);
  });
});
