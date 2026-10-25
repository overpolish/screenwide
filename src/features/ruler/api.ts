// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { RulerIslandAction } from "../../bindings/RulerIslandAction";
import type { RulerIslandState } from "../../bindings/RulerIslandState";
import type { RulerTolerance } from "../../bindings/RulerTolerance";
import type { RulerTool } from "../../bindings/RulerTool";

/** What the island shows of the Ruler, or `null` while it is not up.
 * Changes arrive as `ruler://island`. */
export const getRulerIslandState = () =>
  invoke<RulerIslandState | null>("ruler_island_state");

export const setRulerTool = (tool: RulerTool) =>
  invoke<null>("set_ruler_tool", { tool });

export const setRulerTolerance = (tolerance: RulerTolerance) =>
  invoke<null>("set_ruler_tolerance", { tolerance });

export const runRulerAction = (action: RulerIslandAction) =>
  invoke<null>("ruler_island_action", { action });

/** A press on the island is done with: the keyboard goes back to the
 * Ruler, where its keys work. */
export const returnRulerKeyboard = () => invoke<null>("return_ruler_keyboard");

/** Leaves the Ruler, as Escape does. */
export const closeRuler = () => invoke<null>("cancel_ruler");
