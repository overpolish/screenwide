// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { canHoldSelect, followedTool } from "./editor-choice";

describe("when ⌘ holds Select", () => {
  const hold = (tool: string, hasLayerInHand = false, isExportOpen = false) =>
    canHoldSelect({ hasLayerInHand, isExportOpen, tool });

  it("holds over a pen, with or without a layer in hand", () => {
    expect(hold("arrow")).toBe(true);
    expect(hold("text", true)).toBe(true);
  });

  it("holds over Select only on a chosen layer, whose chrome it stands down", () => {
    expect(hold("select", true)).toBe(true);
    expect(hold("select")).toBe(false);
  });

  it("never holds over the tools that draw their own chrome, nor under Export", () => {
    for (const tool of ["crop", "canvas", "scene"])
      expect(hold(tool, true)).toBe(false);
    expect(hold("arrow", true, true)).toBe(false);
  });
});

describe("the tool following the choice", () => {
  const follow = (
    tool: string,
    kind: Parameters<typeof followedTool>[0]["kind"],
    gate: { isHeld?: boolean; isPicturePressed?: boolean } = {},
  ) =>
    followedTool({
      isHeld: gate.isHeld ?? false,
      isPicturePressed: gate.isPicturePressed ?? false,
      kind,
      tool,
    });

  it("takes up the pen of the one annotation chosen, from Select or another pen", () => {
    expect(follow("select", "counter")).toBe("counter");
    expect(follow("arrow", "counter")).toBe("counter");
    expect(follow("counter", "counter")).toBeNull();
  });

  it("waits while the picture is pressed or ⌘ holds Select", () => {
    expect(follow("select", "arrow", { isPicturePressed: true })).toBeNull();
    expect(follow("select", "arrow", { isHeld: true })).toBeNull();
  });

  it("takes up nothing for several chosen, nor over a mode", () => {
    expect(follow("select", null)).toBeNull();
    expect(follow("crop", "arrow")).toBeNull();
    expect(follow("scene", "arrow")).toBeNull();
  });
});
