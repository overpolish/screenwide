// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RulerStampKeys } from "./ruler-island-tools";
import type { RulerIslandState } from "../../../bindings/RulerIslandState";

/** A session a little way in: something measured, the aids at their
 * defaults. */
export const PREVIEW_RULER_STATE: RulerIslandState = {
  canCopy: true,
  canUndo: true,
  centerlines: true,
  crosshair: false,
  tolerance: "balanced",
  tool: "measure",
};

/** The stamp keys as the Ruler binds them by default. */
export const PREVIEW_STAMP_KEYS: RulerStampKeys = {
  horizontal: "Digit1",
  vertical: "Digit2",
};
