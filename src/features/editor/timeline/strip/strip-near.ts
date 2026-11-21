// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, useState, useSyncExternalStore } from "react";

import { type Span, type TimelineScale, type View } from "../timeline-scale";

import { type StripViewNow } from "./strip-view-now";

// What of the strip is drawn: the parts near the view, not the whole
// recording. A long recording has hundreds of beats, lulls, captions and
// bars, and drawing them all again for every step of a zoom or a trim kept
// the strip from following the hand. What is drawn reaches a view's width
// past each side of it, so a pan draws nothing new until it has come within
// half a view of the edge of what is drawn, and then only what is near the
// view again. What is chosen, or carried, is drawn wherever it is, since
// its island and its drag follow it.

/** How far past each side of the view parts are drawn, in views. */
const MARGIN = 1;

/** The part of the strip whose parts are drawn, as shares of the edited
 * recording; null, as outside the strip, to draw everything. */
export const StripNearContext = createContext<View | null>(null);

/** Whether `span`, laid along `scale`, is near enough the view to be
 * drawn. */
export const isNear = (near: View | null, scale: TimelineScale, span: Span) =>
  near === null ||
  (scale.toX(span.end, true) >= near.start &&
    scale.toX(span.start) <= near.end);

const around = (view: View): View => {
  const span = view.end - view.start;
  return { end: view.end + span * MARGIN, start: view.start - span * MARGIN };
};

/** What is drawn for `view`: as it is while the view keeps half a view
 * inside it on each side and it is not many views wider than the view, or
 * else afresh round the view. */
const nearFor = (current: View, view: View): View => {
  const span = view.end - view.start;
  const isInside =
    view.start >= current.start + span / 2 &&
    view.end <= current.end - span / 2;
  const isTooWide = current.end - current.start > span * (1 + MARGIN * 2) * 2;
  return isInside && !isTooWide ? current : around(view);
};

/** The part of the strip to draw for the view `now` shows, drawing again
 * only when that changes. */
export function useStripNear(now: StripViewNow): View {
  const [store] = useState(() => {
    let near = around(now.get());
    return {
      get: () => {
        near = nearFor(near, now.get());
        return near;
      },
    };
  });
  return useSyncExternalStore(now.subscribe, store.get);
}
