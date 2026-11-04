// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, PresenceContext, useReducedMotion } from "motion/react";
import {
  Children,
  isValidElement,
  type ReactElement,
  type ReactNode,
  use,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { cn } from "../../lib/styling";

import { IslandFollowContext } from "./island-follow";
import { focusIn, glide, leave, reveal } from "./island-motion";

// The island's one transition for what it holds: the parts of a row, of the
// line under it or of a section that come, go and replace each other as
// choices change. A part leaving fades where it stands, keeping its room, so
// nothing beside it moves while it is still seen. Then:
//
// - Where nothing takes its place, it folds its room away on the black's
//   curve, so the parts after it close up as the black does, in the flow,
//   with no part moved over another or out of what clips it. The black
//   follows the fold exactly meanwhile (`IslandFollowContext`).
// - Where others replace it, the room changes hands at once and the black
//   reshapes round the newcomers, which come into focus once it has, so
//   they are never seen riding its moving edge. Parts that stay glide to
//   their new places with the black (`layout`).
//
// Nothing is ever lifted out of the flow, so nothing is placed from an edge
// the island is not anchored to. Parts are the children, identified by key,
// or by place without one, so `a ? <A /> : <B />` needs keys for the swap to
// be seen. `className` lays them out, and its gap goes with the part after
// it; `direction` says which side of a part that gap is.

type Shown = {
  key: string;
  /** What a part leaving last showed, held while it goes. */
  leaving?: ReactElement;
};

/** What is shown next: parts leaving are held until they have gone, and
 * parts arriving wait for them. */
function reconcile({
  holdsLeaving,
  last,
  shown,
  target,
}: {
  holdsLeaving: boolean;
  last: ReadonlyMap<string, ReactElement>;
  shown: readonly Shown[];
  target: readonly string[];
}): Shown[] {
  const wanted = new Set(target);
  const kept = shown.flatMap((entry): Shown[] => {
    if (wanted.has(entry.key)) return [{ key: entry.key }];
    const leaving = holdsLeaving
      ? (entry.leaving ?? last.get(entry.key))
      : undefined;
    return leaving ? [{ key: entry.key, leaving }] : [];
  });
  if (kept.some((entry) => entry.leaving)) return kept;
  return target.map((key) => ({ key }));
}

export function IslandParts({
  children,
  className,
  direction = "column",
  initial = false,
  partClassName,
}: {
  children: ReactNode;
  className?: string;
  /** Which way the parts are laid out. */
  direction?: "column" | "row";
  /** Reveals what it first holds as it mounts, as a part arriving later is;
   * off, what it first holds is revealed with the island. */
  initial?: boolean;
  /** Lays out the box each part is held in, such as a grid's span, where
   * the layout reaches past the parts' own boxes. */
  partClassName?: string;
}) {
  const reduce = useReducedMotion() ?? false;
  // eslint-disable-next-line @eslint-react/no-children-to-array
  const parts = Children.toArray(children).filter(isValidElement);
  const target = parts.map((part) => String(part.key));
  const byKey = new Map(parts.map((part) => [String(part.key), part]));

  // What each part last showed, which a part starting to leave keeps.
  const lastRef = useRef<ReadonlyMap<string, ReactElement>>(byKey);
  const [shown, setShown] = useState<Shown[]>(() =>
    target.map((key) => ({ key })),
  );
  // Worked out while rendering, so a part arriving with the change that
  // sends another away already waits for it.
  const next = reconcile({
    holdsLeaving: !reduce,
    last: lastRef.current,
    shown,
    target,
  });
  const changed =
    next.length !== shown.length ||
    next.some(
      (entry, index) =>
        entry.key !== shown[index]?.key ||
        Boolean(entry.leaving) !== Boolean(shown[index]?.leaving),
    );
  if (changed) setShown(next);
  useLayoutEffect(() => {
    lastRef.current = byKey;
  });
  // Parts there from the start are revealed with the island unless
  // `initial` asks otherwise; parts mounted later come into focus on their
  // own.
  const hasMountedRef = useRef(initial);
  useEffect(() => {
    hasMountedRef.current = true;
  }, []);

  const shownKeys = new Set(next.map((entry) => entry.key));
  // Parts leaving with nothing to take their place fold their room away;
  // with newcomers waiting, they give it up at once.
  const folds = target.every((key) => shownKeys.has(key));
  const leavingKeys = next
    .filter((entry) => entry.leaving)
    .map((entry) => entry.key)
    .join("\n");
  const goneAfter = leave.duration + (folds ? glide.duration : 0);
  const follow = use(IslandFollowContext);
  useEffect(() => {
    if (!leavingKeys) return;
    // The black follows the room folding away rather than trailing it, for
    // a frame or two past the fold so it takes the last of it, along the
    // way the parts are laid: a row's parts fold their width, a column's
    // their height.
    const folding = folds
      ? window.setTimeout(() => {
          follow(
            glide.duration + 0.03,
            direction === "row" ? "across" : "down",
          );
        }, leave.duration * 1000)
      : 0;
    const gone = window.setTimeout(() => {
      setShown((current) => current.filter((entry) => !entry.leaving));
    }, goneAfter * 1000);
    return () => {
      window.clearTimeout(folding);
      window.clearTimeout(gone);
    };
  }, [direction, folds, follow, goneAfter, leavingKeys]);

  // The gap the layout sets between parts (`className`), read once and laid
  // as each part's margin instead, so a part folding away takes the gap
  // before it with it rather than leaving it to close in a jump.
  const rootRef = useRef<HTMLDivElement>(null);
  const [gap, setGap] = useState<number | null>(null);
  useLayoutEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const style = getComputedStyle(root);
    const value = parseFloat(
      direction === "row" ? style.columnGap : style.rowGap,
    );
    setGap(Number.isFinite(value) ? value : 0);
  }, [direction]);

  // A part folds both ways: along the flow its room closes, and across it a
  // part wider or taller than the rest takes the extra with it, so nothing
  // is left to give up at once when it goes.
  const fold = { height: 0, width: 0 };
  const unfolded = { height: "auto", width: "auto" };
  const folding = { ...glide, delay: leave.duration };
  const margin = direction === "row" ? "marginInlineStart" : "marginTop";
  const isFoldingAway = (entry: Shown) => Boolean(entry.leaving) && folds;
  // The first part to stay has no gap before it: what folds above it takes
  // that gap away.
  const firstStaying = next.findIndex((entry) => !isFoldingAway(entry));
  const still = { duration: 0 };
  // A gap closes with the part folding before it; otherwise it is laid at
  // once, as the room changes hands, and the parts glide over it (`layout`).
  // Laid in place at once when first read, too.
  const marginsLiveRef = useRef(false);
  useEffect(() => {
    if (gap !== null) marginsLiveRef.current = true;
  }, [gap]);
  const isFoldingNow = next.some(isFoldingAway);
  const marginTransition =
    marginsLiveRef.current && isFoldingNow
      ? { ...glide, delay: leave.duration }
      : still;
  return (
    <div
      className={cn("relative", className)}
      ref={rootRef}
      style={gap === null ? undefined : { gap: 0 }}
    >
      {/* Motion hands an outer presence's `initial={false}` to everything
          mounted under it for as long as it stays, so a part arriving
          later under the island's row or section would be shown at once.
          The parts here come and go on their own. */}
      <PresenceContext value={null}>
        {next.map((entry, index) => {
          const part = entry.leaving ?? byKey.get(entry.key);
          if (!part) return null;
          const isFolding = isFoldingAway(entry);
          const gapBefore =
            gap === null
              ? {}
              : { [margin]: isFolding || index <= firstStaying ? 0 : gap };
          return (
            <motion.div
              animate={
                entry.leaving
                  ? {
                      ...focusIn.exit,
                      ...gapBefore,
                      ...(folds ? fold : {}),
                    }
                  : { ...reveal.animate, ...gapBefore, ...unfolded }
              }
              className={cn(
                "flex flex-col",
                partClassName,
                isFolding && "overflow-clip",
              )}
              // Out of reach of the pointer and assistive technology as it goes.
              inert={Boolean(entry.leaving)}
              initial={
                hasMountedRef.current
                  ? { ...reveal.initial, ...gapBefore }
                  : false
              }
              key={entry.key}
              layout="position"
              transition={
                reduce
                  ? still
                  : entry.leaving
                    ? {
                        ...leave,
                        height: folding,
                        layout: glide,
                        [margin]: marginTransition,
                        width: folding,
                      }
                    : // Once the black has reshaped round its room, so it is
                      // never seen riding the black's moving edge.
                      {
                        ...reveal.transition,
                        delay: glide.duration,
                        height: still,
                        layout: glide,
                        [margin]: marginTransition,
                        width: still,
                      }
              }
            >
              {part}
            </motion.div>
          );
        })}
      </PresenceContext>
    </div>
  );
}
