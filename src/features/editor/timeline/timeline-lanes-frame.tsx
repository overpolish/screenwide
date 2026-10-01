// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useLayoutEffect, useRef, useState } from "react";

import { ScrollArea } from "../../../components/base/scroll-area/scroll-area";

import { useReportTimelineBlockHeight } from "./timeline-band-block";
import { timelineLanesHeight } from "./timeline-band-metrics";

/**
 * The timeline block's own frame. Only the lane rows scroll: the ruler stands
 * fixed above them and the meter fixed beside them, so the overflow shadow
 * annotations exactly where the rows are cut off.
 *
 * The ruler and the rows line up because both carry the same gutter column -
 * the zoom toolbar in the ruler, the track header in each row, each a
 * `w-timeline-gutter` box before the same `gap-section` - and because
 * OverlayScrollbars draws its bar over the content rather than beside it, so
 * a scrollbar appearing never narrows the rows out from under the ruler.
 *
 * It is laid out with `grow` rather than `flex-1` throughout: with the basis
 * left at the rows' own height, the frame fills a band that bounds it and
 * still stands at its content's height where nothing does - the lanes' own
 * story, which has no band around it.
 */
export function TimelineLanesFrame({
  children,
  meter,
  overlays,
  ruler,
}: {
  /** The lane rows - the only thing in the band that scrolls. */
  children: ReactNode;
  /** Playhead, range and blade: they span the ruler and the visible rows. */
  overlays: ReactNode;
  ruler: ReactNode;
  /** Drawn beside the rows, at the height of the rows actually visible. */
  meter?: (visibleHeight: number) => ReactNode;
}) {
  const reportHeight = useReportTimelineBlockHeight();
  const rowsRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const [visibleHeight, setVisibleHeight] = useState(0);
  // What the band is told to fit to: the rows as they would stand unclipped,
  // plus the ruler row above them, which the band cannot see inside here.
  useLayoutEffect(() => {
    const rows = rowsRef.current;
    if (!rows) return;
    const report = () => {
      reportHeight(timelineLanesHeight(rows.scrollHeight));
    };
    // Eagerly, before the band's own first look at the block: a band that has
    // not been told this height yet would otherwise measure the block it
    // clips, which is the band's current height rather than the one it wants.
    report();
    const observer = new ResizeObserver(report);
    observer.observe(rows);
    return () => {
      observer.disconnect();
      reportHeight(null);
    };
  }, [reportHeight]);
  // The meter stands beside the rows rather than scrolling with them, so it is
  // sized to the strip of rows on screen, not to the whole lane content.
  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;
    const observer = new ResizeObserver(() => {
      setVisibleHeight(viewport.clientHeight);
    });
    observer.observe(viewport);
    return () => {
      observer.disconnect();
    };
  }, []);
  return (
    <div className="flex min-h-0 grow items-stretch gap-control">
      {/* The overlays are positioned against this column, which spans the
          ruler and the visible rows: a playhead line runs the full height of
          both, and none of it scrolls away with the rows. Their
          `top-control-height` lands at the end of the ruler row, in the gap
          above the first lane, exactly as it did when the ruler was the
          column's first scrolling row. */}
      <div className="relative flex min-w-0 grow flex-col gap-control">
        <div className="shrink-0 pl-window-inset">{ruler}</div>
        <div className="min-h-0 grow" ref={viewportRef}>
          <ScrollArea edgeEffect="shadow">
            {/* The strip's bottom inset travels with the rows, so the last
                lane clears the band's edge once it is scrolled to. */}
            <div
              className="flex flex-col gap-control pb-control-inset pl-window-inset"
              ref={rowsRef}
            >
              {children}
            </div>
          </ScrollArea>
        </div>
        {overlays}
      </div>
      {meter ? (
        <div className="flex shrink-0 flex-col gap-control pr-window-inset">
          {/* The ruler's row, kept as a real row in the meter's column so the
              meter starts beside the first lane instead of being pushed down
              by a margin the ruler's height has to be guessed into. */}
          <div aria-hidden className="h-control-height shrink-0" />
          {meter(visibleHeight)}
        </div>
      ) : null}
    </div>
  );
}
