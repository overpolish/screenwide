// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, use } from "react";

/**
 * The band owns a height; the block inside it owns a scroller. Since the ruler
 * stands above that scroller, the block's natural height is no longer anything
 * the band can read off the DOM it holds - the part that scrolls is clipped to
 * whatever the band currently is. So the block reports the height it wants
 * through here, and the band uses it for its own arithmetic: what a refit
 * shows, and how far the divider may travel before empty band opens.
 *
 * `null` unreports it, which is what a block without a scroller of its own
 * leaves in place: the band then measures the block it holds, the way it did
 * before anything inside it scrolled.
 *
 * The callback is stable, so reporting cannot re-render the block that reports.
 */

export type ReportBlockHeight = (height: number | null) => void;

export const ReportBlockHeightContext = createContext<ReportBlockHeight>(() => {
  // No band around this block - nothing is sizing itself to it.
});

/** The timeline block tells the band how tall it would like the band to be. */
export const useReportTimelineBlockHeight = () => use(ReportBlockHeightContext);
