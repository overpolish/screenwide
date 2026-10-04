// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { validAnnotations } from "./annotation-documents";

/** A stored sticker, as a document would carry it, with `shape` over the
 * sticker's own. */
const sticker = (shape: Record<string, unknown>) => ({
  animated: true,
  id: "s",
  shape: {
    angle: 0.5,
    aspect: 2,
    asset: "emoji:👍",
    center: { x: 40, y: 30 },
    kind: "sticker",
    size: 96,
    ...shape,
  },
  style: {
    align: "left",
    blur: false,
    color: "#ffffff",
    handDrawn: false,
    head: "none",
    manual: false,
    radius: 0,
    redaction: "erase",
    shadow: false,
    softness: 0,
    strength: 0,
    tint: false,
    width: 1,
  },
});

describe("a stored sticker", () => {
  it("reads back unmirrored, and is dropped where it cannot be sized", () => {
    const [read] = validAnnotations([sticker({})]);
    expect(read.shape).toEqual({
      angle: 0.5,
      aspect: 2,
      asset: "emoji:👍",
      center: { x: 40, y: 30 },
      flip: false,
      kind: "sticker",
      size: 96,
    });
    expect(validAnnotations([sticker({ size: 0 })])).toEqual([]);
    expect(validAnnotations([sticker({ aspect: -1 })])).toEqual([]);
    expect(validAnnotations([sticker({ asset: undefined })])).toEqual([]);
  });

  it("keeps how a moving picture plays, and reads one of one frame as still", () => {
    const [moving] = validAnnotations([
      sticker({ play: { cycleMs: 400, frame: 9, frames: 4, once: true } }),
    ]);
    // A frame past the end is held to the last.
    expect(moving.shape).toMatchObject({
      play: { cycleMs: 400, frame: 3, frames: 4, once: true },
    });
    const [still] = validAnnotations([
      sticker({ play: { cycleMs: 400, frames: 1 } }),
    ]);
    expect(still.shape).not.toHaveProperty("play");
  });
});
