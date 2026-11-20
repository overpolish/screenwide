// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useLocale } from "react-aria-components";

import {
  closesAfter,
  glide,
} from "../../../../primitives/island/island-motion";
import { type PopupPanelPart } from "../../../popup-panel/api";
import { EditorIsland, type EditorIslandContent } from "../editor-island";
import { ISLAND_ROOM } from "../island-room";

import { type PictureInset } from "./editor-island-kind";
import { type Bounds, boundsOf } from "./island-bounds";

// One island in the window laid over the editor's content, at one of the
// picture's corners: the top end one, whatever brought it out, the top
// start one, as the Script island stands, or the bottom start one, as the
// models' work stands; or at the picture's foot over a point along it, as a
// strip item's island stands over the item. It reports where its parts are
// as it changes shape, moved to where it now stands. The picture lies
// `inset` in from the window's edges; a top corner's island may reach down
// past the picture's foot to the window's, as the tall Export island does.
// An island over a strip item keeps clear of the models' work in the bottom
// start corner, moving toward the end as it moves to stay on the picture.

/** Between the island and the picture's edges, in px. */
const EDGE_PX = 12;

export function PlacedIsland({
  at = null,
  clearOf = null,
  corner = "end",
  edge = "top",
  inset,
  onParts,
  shown,
}: {
  /** Where the picture lies in the window. */
  inset: PictureInset;
  /** Told where the island's parts are, in the window, as they change. */
  onParts: (parts: PopupPanelPart[]) => void;
  /** What the island holds, or null to put it away. */
  shown: EditorIslandContent | null;
  /** Where along the window, in px from its left edge, its middle stands
   * at the picture's foot, kept over the picture; at a top corner where
   * null. */
  at?: number | null;
  /** Where another island at the picture's foot stands in the window, the
   * models' work in its start corner, which an island over a strip item
   * moves toward the end to keep clear of, as it keeps inside the picture. */
  clearOf?: Bounds | null;
  /** Which of the picture's corners it stands in, along the line of the
   * text, where `at` is null. */
  corner?: "end" | "start";
  /** Whether that corner is at the picture's top or its foot. */
  edge?: "bottom" | "top";
}) {
  const { direction } = useLocale();
  // The island's distance from the window's end edge runs from its right
  // in a left-to-right window and from its left in a right-to-left one.
  const fromRight = direction !== "rtl";
  // What the island holds and where it stands. It changes what it holds
  // where it stands, as any island does; and an item moving, as the strip
  // scrolls, it follows at once. Another item somewhere else, or a corner's
  // island giving way to a strip item's or back, it shrinks away where it
  // stood and comes out again where it is going, rather than flying across
  // the window with what it held. Kept by key: what is shown is made afresh
  // on every render.
  const [standing, setStanding] = useState<{
    at: number | null;
    key: string;
  } | null>(shown ? { at, key: shown.key } : null);
  const [held, setHeld] = useState<EditorIslandContent | null>(shown);
  // A corner and the strip are never the same place, even for one item
  // whose place and island reach the window a message apart.
  const isMoving =
    shown !== null &&
    standing !== null &&
    standing.at !== at &&
    (standing.key !== shown.key || (standing.at === null) !== (at === null));
  if (shown && !isMoving && (standing?.at !== at || standing.key !== shown.key))
    setStanding({ at, key: shown.key });
  if (shown && !isMoving && shown.key !== held?.key) setHeld(shown);
  const isOut = shown !== null && !isMoving;
  const displayed = isOut ? shown : held;
  const placeAt = standing ? standing.at : at;
  // Out again where it is going once the black has closed where it stood,
  // rather than when it is done with the fade that follows (`onHidden`),
  // which has nothing left to show; and out afresh (`trip`), as an island
  // comes out, rather than growing round what it held and then changing.
  const reduce = useReducedMotion() ?? false;
  const [trip, setTrip] = useState(0);
  const goingRef = useRef({ at, shown });
  goingRef.current = { at, shown };
  const arrive = () => {
    const going = goingRef.current;
    if (!going.shown) return;
    setStanding({ at: going.at, key: going.shown.key });
    setHeld(going.shown);
    setTrip((count) => count + 1);
  };
  useEffect(() => {
    if (!isMoving) return;
    const arrives = window.setTimeout(
      arrive,
      reduce ? 0 : (closesAfter + glide.duration) * 1000,
    );
    return () => {
      window.clearTimeout(arrives);
    };
  }, [isMoving, reduce]);
  const holderRef = useRef<HTMLDivElement>(null);
  /** How far the middle of an island `width` wide over `point` stands from
   * the holder's edge, the window's left running left to right, kept over
   * the picture and, standing `top` to `bottom` in the window, clear of
   * `clearOf`, toward the end. */
  const fromSide = (
    point: number,
    width: number,
    [top, bottom]: [number, number],
  ) => {
    const least = inset.left + EDGE_PX + width / 2;
    const most = window.innerWidth - inset.right - EDGE_PX - width / 2;
    const over = Math.max(least, Math.min(point, most));
    const clashes = (clear: Bounds) =>
      top < clear.bottom + EDGE_PX &&
      bottom > clear.top - EDGE_PX &&
      over - width / 2 < clear.end + EDGE_PX &&
      over + width / 2 > clear.start - EDGE_PX;
    const middle =
      clearOf === null || !clashes(clearOf)
        ? over
        : fromRight
          ? Math.min(most, clearOf.end + EDGE_PX + width / 2)
          : Math.max(least, clearOf.start - EDGE_PX - width / 2);
    return fromRight ? window.innerWidth - middle : middle;
  };

  // Moved directly rather than through state: the island reports its shape
  // on every frame it changes. In a corner, its end or start edge stays
  // `EDGE_PX` in from the picture's, and its top `EDGE_PX` down, or at the
  // foot its foot `EDGE_PX` up; over a strip item, its middle over `at`
  // while that keeps it over the picture, and its foot `EDGE_PX` up from the
  // picture's. Placed from where its parts are from its own middle
  // (`bounds`), as its shape's values have them: placed from where they
  // were measured on the page, a frame behind the values, it chased its own
  // move and rocked as it grew or shrank.
  // Returns where the island's middle and top now stand in the window.
  const place = (bounds: Bounds | null) => {
    const holder = holderRef.current;
    // A top corner's island may reach the window's foot; one at the foot
    // stands on the picture.
    const onPicture = placeAt !== null || edge === "bottom";
    holder?.style.setProperty(
      ISLAND_ROOM,
      `${String(window.innerHeight - inset.top - (onPicture ? inset.bottom : 0) - 2 * EDGE_PX)}px`,
    );
    if (!holder || !bounds) return null;
    const width = bounds.end - bounds.start;
    const height = bounds.bottom - bounds.top;
    // The holder centres the island's black, but a column of choices opened
    // past one side of it (`IslandAside`) reaches out on that side alone, so
    // the parts' middle leans off the black's; the lean is kept as it
    // moves, so the whole of it, column and all, stays inside the window.
    const lean = (bounds.start + bounds.end) / 2;
    const endInset = fromRight ? inset.right : inset.left;
    const startInset = fromRight ? inset.left : inset.right;
    const y =
      placeAt === null && edge === "top"
        ? EDGE_PX + inset.top
        : Math.max(
            EDGE_PX + inset.top,
            window.innerHeight - inset.bottom - height - EDGE_PX,
          );
    const middle =
      placeAt !== null
        ? fromSide(placeAt, width, [y, y + height])
        : corner === "end"
          ? EDGE_PX + endInset + width / 2
          : window.innerWidth - EDGE_PX - startInset - width / 2;
    const x = middle + (fromRight ? lean : -lean);
    holder.style.transform = `translate(${String(fromRight ? -x : x)}px, ${String(y)}px)`;
    return { left: fromRight ? window.innerWidth - x : x, top: y };
  };

  // The parts from the island's middle as last reported, to be placed
  // again without changing shape.
  const partsRef = useRef<PopupPanelPart[]>([]);
  const report = (fromMiddle: PopupPanelPart[]) => {
    partsRef.current = fromMiddle;
    const origin = place(boundsOf(fromMiddle));
    onParts(
      origin
        ? fromMiddle.map((part) => ({
            ...part,
            x: part.x + origin.left,
            y: part.y + origin.top,
          }))
        : [],
    );
  };

  // Another place, a picture lying elsewhere in the window, or a window of
  // another height under a strip item's island, moves the island without
  // it changing shape.
  const reportRef = useRef(report);
  reportRef.current = report;
  useLayoutEffect(() => {
    reportRef.current(partsRef.current);
  }, [
    placeAt,
    inset.bottom,
    inset.left,
    inset.right,
    inset.top,
    clearOf?.bottom,
    clearOf?.end,
    clearOf?.start,
    clearOf?.top,
  ]);
  useEffect(() => {
    const replace = () => {
      reportRef.current(partsRef.current);
    };
    window.addEventListener("resize", replace);
    return () => {
      window.removeEventListener("resize", replace);
    };
  }, []);

  return (
    <div
      className="absolute top-0 flex w-0 justify-center"
      data-island-room
      ref={holderRef}
      style={{ insetInlineEnd: 0 }}
    >
      {displayed ? (
        <EditorIsland
          content={displayed}
          isShown={isOut}
          key={trip}
          onHidden={() => {
            if (shown) {
              arrive();
              return;
            }
            setStanding(null);
            setHeld(null);
            report([]);
          }}
          onShapeChange={(_onPage, fromMiddle) => {
            report(fromMiddle);
          }}
        />
      ) : null}
    </div>
  );
}
