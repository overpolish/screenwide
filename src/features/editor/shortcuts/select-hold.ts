// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Platform } from "../../../lib/platform";

/**
 * Holding ⌘ (Ctrl on Windows) puts Select in hand for as long as it is held,
 * as Photoshop and Illustrator spring-load their selection tools.
 *
 * - `idle`: the modifier is up.
 * - `held`: the modifier went down with a pen in hand and nothing pressed, so
 *   the picture acts as Select until it is let go.
 * - `spent`: the modifier is down but holds nothing: it went down mid-press,
 *   under a tool that keeps its own meaning for it, or another key joined it
 *   to make a shortcut (⌘Z, ⌘D). It means nothing more until it is let go.
 */
export type SelectHold = "idle" | "held" | "spent";

export type SelectHoldEvent =
  /** The modifier went down; `canHold` when nothing is pressed and the tool
   * in hand lets go of the picture for it. */
  | { canHold: boolean; type: "modifierDown" }
  | { type: "modifierUp" }
  | { type: "otherKey" }
  | { type: "blur" };

export const selectHold = (
  state: SelectHold,
  event: SelectHoldEvent,
): SelectHold => {
  switch (event.type) {
    case "modifierDown":
      if (state !== "idle") return state;
      return event.canHold ? "held" : "spent";
    case "otherKey":
      return state === "held" ? "spent" : state;
    case "modifierUp":
    case "blur":
      return "idle";
  }
};

/** Whether `key` is the modifier that holds Select on `platform`: Command on
 * macOS, Ctrl elsewhere. */
export const isSelectHoldKey = (key: string, platform: Platform) =>
  key === (platform === "macos" ? "Meta" : "Control");
