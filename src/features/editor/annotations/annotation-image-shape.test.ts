// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { validAnnotations } from "./annotation-documents";

/** A stored image, as a document would carry it, with `shape` over the
 * image's own. */
const image = (shape: Record<string, unknown>) => ({
  animated: true,
  id: "s",
  shape: {
    angle: 0.5,
    aspect: 2,
    asset: "image:0123456789abcdef0123456789abcdef",
    center: { x: 40, y: 30 },
    kind: "image",
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

describe("a stored image", () => {
  it("reads back unmirrored, and is dropped where it cannot be sized", () => {
    const [read] = validAnnotations([image({})]);
    expect(read.shape).toEqual({
      angle: 0.5,
      aspect: 2,
      asset: "image:0123456789abcdef0123456789abcdef",
      center: { x: 40, y: 30 },
      flip: false,
      kind: "image",
      size: 96,
    });
    expect(validAnnotations([image({ size: 0 })])).toEqual([]);
    expect(validAnnotations([image({ aspect: -1 })])).toEqual([]);
    expect(validAnnotations([image({ asset: undefined })])).toEqual([]);
  });

  it("keeps how a moving picture plays, and reads one of one frame as still", () => {
    const [moving] = validAnnotations([
      image({ play: { cycleMs: 400, frame: 9, frames: 4, once: true } }),
    ]);
    // A frame past the end is held to the last.
    expect(moving.shape).toMatchObject({
      play: { cycleMs: 400, frame: 3, frames: 4, once: true },
    });
    const [still] = validAnnotations([
      image({ play: { cycleMs: 400, frames: 1 } }),
    ]);
    expect(still.shape).not.toHaveProperty("play");
  });
});
