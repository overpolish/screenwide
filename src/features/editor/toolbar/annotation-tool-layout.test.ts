// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { annotationToolLayout } from "./annotation-tool-layout";

const order = ["image", "arrow", "text", "redact", "draw"];

describe("the annotation strip's overflow", () => {
  it("shows every tool while all of them fit", () => {
    expect(annotationToolLayout(order, 5, "draw")).toEqual({
      hidden: [],
      slot: null,
      visible: order,
    });
  });

  it("gives the slot and the overflow button a place each once one does not fit", () => {
    expect(annotationToolLayout(order, 4, null)).toEqual({
      hidden: ["redact", "draw"],
      slot: "text",
      visible: ["image", "arrow"],
    });
  });

  it("keeps the last hidden tool taken up in the slot without shifting the row", () => {
    expect(annotationToolLayout(order, 4, "draw")).toEqual({
      hidden: ["text", "redact"],
      slot: "draw",
      visible: ["image", "arrow"],
    });
  });

  it("lets the slot go once its tool fits in its own place", () => {
    expect(annotationToolLayout(order, 3, "arrow")).toEqual({
      hidden: ["text", "redact", "draw"],
      slot: "arrow",
      visible: ["image"],
    });
    expect(annotationToolLayout(order, 4, "arrow")).toEqual({
      hidden: ["redact", "draw"],
      slot: "text",
      visible: ["image", "arrow"],
    });
  });

  it("still shows the slot when there is no room at all", () => {
    expect(annotationToolLayout(order, 0, null)).toEqual({
      hidden: ["arrow", "text", "redact", "draw"],
      slot: "image",
      visible: [],
    });
  });
});
