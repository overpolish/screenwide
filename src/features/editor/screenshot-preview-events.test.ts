// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  screenshotAnnotationChange,
  screenshotAnnotationHover,
} from "./screenshot-preview-events";

const arrow = {
  aboveCamera: false,
  animated: true,
  id: "arrow-1",
  shape: {
    control: { x: 60, y: 40 },
    end: { x: 110, y: 60 },
    kind: "arrow",
    start: { x: 10, y: 20 },
  },
  style: {
    align: "left",
    color: "#0a84ff",
    head: "end",
    radius: 0,
    redaction: "erase",
    strength: 0,
    width: 6,
  },
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
      textEdit: null,
    });
  });

  it("reads where in a text box's typing a commit falls", () => {
    for (const textEdit of ["begin", "update", "end"])
      expect(
        screenshotAnnotationChange(payload({ textEdit }), 7)?.textEdit,
      ).toBe(textEdit);
    expect(
      screenshotAnnotationChange(payload({ textEdit: "sideways" }), 7)
        ?.textEdit,
    ).toBeNull();
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

  it("drops an annotation the compositor could not place", () => {
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

describe("screenshotAnnotationHover", () => {
  it("reads which annotation the halo is on", () => {
    expect(
      screenshotAnnotationHover({ annotationId: "arrow-1", sessionId: 7 }, 7),
    ).toEqual({ annotationId: "arrow-1" });
  });

  it("reads a retired halo as nothing pointed at", () => {
    expect(
      screenshotAnnotationHover({ annotationId: null, sessionId: 7 }, 7),
    ).toEqual({ annotationId: null });
  });

  it("ignores another session's halo", () => {
    expect(
      screenshotAnnotationHover({ annotationId: "arrow-1", sessionId: 8 }, 7),
    ).toBeNull();
  });
});
