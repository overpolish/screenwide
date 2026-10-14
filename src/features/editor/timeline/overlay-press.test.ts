// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createRecordingTimelineEdit } from "./editing/recording-timeline-edit";
import { captionAt, keyboardFragmentAt } from "./overlay-press";

import type { Caption } from "../../../bindings/Caption";

const edit = createRecordingTimelineEdit(1);
const shortcut = (id: number, startMs: number, endMs: number) => ({
  endMs,
  id,
  label: String(id),
  startMs,
});
const at = (
  ratio: number,
  hidden: { fragments?: string[]; items?: number[] } = {},
) =>
  keyboardFragmentAt({
    edit,
    hiddenFragmentIds: new Set(hidden.fragments),
    hiddenItemIds: new Set(hidden.items),
    items: [shortcut(1, 1_000, 4_000), shortcut(2, 3_000, 5_000)],
    ratio,
    sourceDurationMs: 10_000,
  });

describe("keyboardFragmentAt", () => {
  it("picks the shortcut that began last where two still show", () => {
    expect(at(0.35)).toMatch(/^2/u);
    expect(at(0.2)).toMatch(/^1/u);
  });

  it("passes over deleted shortcuts to the one still drawn", () => {
    expect(at(0.35, { items: [2] })).toMatch(/^1/u);
    expect(at(0.45, { items: [2] })).toBeNull();
  });
});

describe("captionAt", () => {
  const caption = (id: number, startMs: number, endMs: number) =>
    ({ endMs, id, startMs }) as Caption;
  const captions = [caption(0, 0, 1_000), caption(1_000, 1_000, 2_000)];

  it("hands a caption over to the next at the moment it ends", () => {
    expect(captionAt(captions, 999)).toBe("0");
    expect(captionAt(captions, 1_000)).toBe("1000");
    expect(captionAt(captions, 2_000)).toBeNull();
  });
});
