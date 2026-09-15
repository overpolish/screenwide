// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

/**
 * The transport row stands above the band's scroller, so it is part of the
 * band's height without being part of the height that scrolls. The band needs
 * that row's height for its own arithmetic - what fits, and how far the
 * divider may travel - but the row is a child handed in by the editor rather
 * than something the band renders itself. So the band publishes a register
 * callback here and the row reports itself through it.
 *
 * The callback is stable, which keeps the memoized transport row's memo intact
 * while the band's height changes under a drag.
 */

export type RegisterPlaybackRow = (element: HTMLElement | null) => void;

export const RegisterPlaybackRowContext = createContext<RegisterPlaybackRow>(
  () => {
    // No band around this row - nothing is sizing itself to it.
  },
);

/** The transport row reports its height to the band that holds it. */
export const useRegisterTimelinePlaybackRow = () =>
  use(RegisterPlaybackRowContext);
