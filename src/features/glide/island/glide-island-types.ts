// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { GlideAction } from "../detection/glide-detection";
import type { GlideRegion } from "../detection/glide-regions";
import type { GlideFit } from "../glide-geometry";

/**
 * Where a gesture is, as the island hears it: none under way, gliding, ended
 * with the window sent on its way, or found on a window the platform will not
 * let us place.
 */
export type GlideIslandPhase = "committed" | "gliding" | "idle" | "locked";

export type GlideIslandState = {
  /** The last settle's report, or null while a destination is in flight. */
  fit: GlideFit | null;
  pending: GlideAction | null;
  phase: GlideIslandPhase;
  region: GlideRegion | null;
};

export type GlideIslandProps = GlideIslandState & {
  /** The glided app's icon, once it resolves. */
  iconSrc: null | string;
  /** Counts the rests completed; each one plays the ready breath. */
  pulse: number;
};

export const hasDestination = ({
  pending,
  region,
}: Pick<GlideIslandState, "pending" | "region">) =>
  pending !== null || region !== null;
