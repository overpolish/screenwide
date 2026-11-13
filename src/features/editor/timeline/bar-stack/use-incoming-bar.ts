// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { type Span } from "../timeline-scale";

import { heldLanes, holdStep } from "./stack-hold";
import { BAR_PX, type StackedBar } from "./stack-lanes";

/** A bar brought in from outside a stack, carried with the pointer `y` px
 * down the lanes shown. */
export type IncomingBar<T> = { bar: T; y: number };

/** A bar coming into the stack, held from the moment it arrives with its
 * middle on the pointer, starting from the top lane, as a bar picked up
 * is: where it is drawn, and where every bar goes if it is let go now,
 * which `onLanes` is told whenever that changes. */
export function useIncomingBar<T extends StackedBar>({
  drawn,
  drawnSpan,
  incoming,
  onLanes,
  scrollTop,
  settled,
}: {
  /** The stack's bars as drawn, as shares of the strip. */
  drawn: StackedBar[];
  drawnSpan: (span: Span) => Span;
  incoming: IncomingBar<T> | null;
  /** How far the lanes are scrolled, in px. */
  scrollTop: number;
  /** Every bar's lane with nothing held. */
  settled: Record<string, number>;
  onLanes?: (lanes: Record<string, number> | null) => void;
}) {
  const arriving = incoming && {
    ...incoming.bar,
    ...drawnSpan(incoming.bar),
  };
  const held =
    incoming && arriving
      ? holdStep({
          bars: [...drawn, arriving],
          current: null,
          hold: {
            guide: null,
            isLifted: true,
            span: drawnSpan(incoming.bar),
            startY: BAR_PX / 2,
            y: incoming.y + scrollTop,
          },
          id: arriving.id,
          shown: settled,
        })
      : null;
  const lanes = held && arriving ? heldLanes([...drawn, arriving], held) : null;
  // Told only when the lanes change, not each time its owner remakes it.
  const told = lanes ? JSON.stringify(lanes) : null;
  useEffect(() => {
    onLanes?.(
      told === null ? null : (JSON.parse(told) as Record<string, number>),
    );
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- `onLanes` is remade each render; only new lanes are told.
  }, [told]);
  return { arriving, held, lanes };
}
