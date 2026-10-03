// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PointerEvent as ReactPointerEvent,
  RefObject,
  useEffect,
  useRef,
  useState,
} from "react";

import {
  timelineFractionToX,
  TimelineViewportState,
} from "../timeline-viewport";

import { timedLaneFragmentBox } from "./timed-lane-layout";

/** A rectangle in a lane's own CSS pixels. */
export type LaneBox = {
  bottom: number;
  left: number;
  right: number;
  top: number;
};

/** How far a press on empty lane travels before it draws a band rather than
 * clearing the choice; the lanes' clip drags wait the same. */
const BAND_SLOP_PX = 4;

/** Whether a press adds to the choice rather than replacing it: the modifier
 * every timeline lane toggles an item with. */
export const togglesChoice = (event: { ctrlKey: boolean; metaKey: boolean }) =>
  event.metaKey || event.ctrlKey;

/**
 * The ids of the items a band over `box` touches, as a lane `laneWidthPx`
 * wide draws `fragments` under `viewport`, each at least `minimumWidthPx`
 * wide. An item drawn in several fragments is named once.
 */
export const sweptLaneItems = (
  fragments: readonly {
    item: { id: string };
    outputEnd: number;
    outputStart: number;
    row?: number;
  }[],
  box: LaneBox,
  {
    laneWidthPx,
    minimumWidthPx,
    viewport,
  }: {
    laneWidthPx: number;
    minimumWidthPx: number;
    viewport: TimelineViewportState;
  },
): string[] => {
  const lane = { left: 0, width: laneWidthPx };
  const swept = new Set<string>();
  for (const { item, outputEnd, outputStart, row = 0 } of fragments) {
    const left = timelineFractionToX(outputStart, viewport, lane);
    const right = Math.max(
      timelineFractionToX(outputEnd, viewport, lane),
      left + minimumWidthPx,
    );
    const { height, top } = timedLaneFragmentBox(row);
    if (
      left <= box.right &&
      box.left <= right &&
      top <= box.bottom &&
      box.top <= top + height
    )
      swept.add(item.id);
  }
  return [...swept];
};

type Band = {
  /** Whether the band adds to what was chosen when it began. */
  additive: boolean;
  box: LaneBox;
  /** The items under the band, shown chosen while it is drawn. */
  ids: ReadonlySet<string>;
};

/**
 * A band drawn from a press on empty lane space, choosing every item it
 * touches on release; Cmd or Ctrl held at the press adds them to the choice,
 * as the same modifier adds a single item. A press that never travels clears
 * the choice instead, and Escape abandons a band in flight. `sweep` names the
 * items under a box in the lane's own pixels.
 */
export function useTimelineLaneBand({
  laneRef,
  onClear,
  onSweep,
  sweep,
}: {
  laneRef: RefObject<HTMLDivElement | null>;
  onClear: () => void;
  onSweep: (ids: string[], additive: boolean) => void;
  sweep: (box: LaneBox, laneWidthPx: number) => string[];
}) {
  const [band, setBand] = useState<Band | null>(null);
  // The window listeners live for one gesture while these are rebuilt every
  // render, so the listeners read them through a ref.
  const latestRef = useRef({ onClear, onSweep, sweep });
  latestRef.current = { onClear, onSweep, sweep };
  const detachRef = useRef<() => void>(() => undefined);
  useEffect(
    () => () => {
      detachRef.current();
    },
    [],
  );

  const pressLane = (event: ReactPointerEvent<HTMLDivElement>) => {
    const lane = laneRef.current;
    if (event.button !== 0 || !lane) return;
    // No text selection follows the band across the timeline.
    event.preventDefault();
    const additive = togglesChoice(event);
    const origin = { x: event.clientX, y: event.clientY };
    let swept: string[] | null = null;
    const sample = (clientX: number, clientY: number) => {
      if (
        swept === null &&
        Math.hypot(clientX - origin.x, clientY - origin.y) < BAND_SLOP_PX
      )
        return;
      const bounds = lane.getBoundingClientRect();
      const x = (value: number) =>
        Math.max(0, Math.min(bounds.width, value - bounds.left));
      const y = (value: number) =>
        Math.max(0, Math.min(bounds.height, value - bounds.top));
      const box = {
        bottom: y(Math.max(origin.y, clientY)),
        left: x(Math.min(origin.x, clientX)),
        right: x(Math.max(origin.x, clientX)),
        top: y(Math.min(origin.y, clientY)),
      };
      swept = latestRef.current.sweep(box, bounds.width);
      setBand({ additive, box, ids: new Set(swept) });
    };
    const move = (moved: PointerEvent) => {
      sample(moved.clientX, moved.clientY);
    };
    const release = (released: PointerEvent) => {
      sample(released.clientX, released.clientY);
      detachRef.current();
      if (swept !== null) latestRef.current.onSweep(swept, additive);
      else if (!additive) latestRef.current.onClear();
    };
    const escape = (pressed: KeyboardEvent) => {
      if (pressed.key !== "Escape") return;
      pressed.preventDefault();
      pressed.stopImmediatePropagation();
      detachRef.current();
    };
    const abandon = () => {
      detachRef.current();
    };
    detachRef.current = () => {
      detachRef.current = () => undefined;
      window.removeEventListener("pointermove", move, true);
      window.removeEventListener("pointerup", release, true);
      window.removeEventListener("pointercancel", abandon, true);
      window.removeEventListener("keydown", escape, true);
      setBand(null);
    };
    window.addEventListener("pointermove", move, true);
    window.addEventListener("pointerup", release, true);
    window.addEventListener("pointercancel", abandon, true);
    window.addEventListener("keydown", escape, true);
  };

  return { band, pressLane };
}
