// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  isSelectHoldKey,
  SelectHold,
  selectHold,
  SelectHoldEvent,
} from "./select-hold";

const run = (events: SelectHoldEvent[], from: SelectHold = "idle") =>
  events.reduce(selectHold, from);

const down = (canHold = true): SelectHoldEvent => ({
  canHold,
  type: "modifierDown",
});

describe("holding the modifier for Select", () => {
  it("holds Select from the modifier going down to it coming up", () => {
    expect(run([down()])).toBe("held");
    expect(run([down(), { type: "modifierUp" }])).toBe("idle");
  });

  it("holds nothing when it goes down mid-press or under a tool that keeps it", () => {
    expect(run([down(false)])).toBe("spent");
    expect(run([down(false), { type: "modifierUp" }])).toBe("idle");
  });

  it("does not start holding once the press it went down in is over", () => {
    // The modifier went down mid-drag; a repeat of it after the release
    // still means nothing until it is let go.
    expect(run([down(false), down(true)])).toBe("spent");
  });

  it("lets go when another key makes it a shortcut, until it is let go", () => {
    expect(run([down(), { type: "otherKey" }])).toBe("spent");
    expect(run([down(), { type: "otherKey" }, down()])).toBe("spent");
    expect(run([down(), { type: "otherKey" }, { type: "modifierUp" }])).toBe(
      "idle",
    );
  });

  it("ignores other keys while the modifier is up", () => {
    expect(run([{ type: "otherKey" }])).toBe("idle");
  });

  it("lets go when the window loses focus", () => {
    expect(run([down(), { type: "blur" }])).toBe("idle");
    expect(run([down(false), { type: "blur" }])).toBe("idle");
  });

  it("is Command on macOS and Ctrl on Windows", () => {
    expect(isSelectHoldKey("Meta", "macos")).toBe(true);
    expect(isSelectHoldKey("Control", "macos")).toBe(false);
    expect(isSelectHoldKey("Control", "windows")).toBe(true);
    expect(isSelectHoldKey("Meta", "windows")).toBe(false);
  });
});
