// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PointerEvent as ReactPointerEvent,
  RefObject,
  useEffect,
  useRef,
  useState,
} from "react";

import { RecordingAnnotationClip } from "../recording-annotations";

import { LaneBox, sweptAnnotationClips } from "./recording-annotation-layout";
import { StackedLaneFragment } from "./timed-lane-layout";
import { TimelineViewportState } from "./timeline-viewport";

/** How far a press on empty lane travels before it draws a band rather than
 * clearing the choice; the lane's clip drags wait the same. */
const BAND_SLOP_PX = 4;

/** Whether a press adds to the choice rather than replacing it: the modifier
 * every timeline lane toggles an item with. */
export const togglesChoice = (event: { ctrlKey: boolean; metaKey: boolean }) =>
  event.metaKey || event.ctrlKey;

type Band = {
  /** Whether the band adds to what was chosen when it began. */
  additive: boolean;
  box: LaneBox;
  /** The clips under the band, shown chosen while it is drawn. */
  ids: ReadonlySet<string>;
};

/**
 * A band drawn from a press on empty lane space, choosing every clip it
 * touches on release; Cmd or Ctrl held at the press adds them to the choice,
 * as the same modifier adds a single clip. A press that never travels clears
 * the choice instead, and Escape abandons a band in flight.
 */
export function useRecordingAnnotationBand({
  fragments,
  laneRef,
  onClear,
  onSweep,
  viewport,
}: {
  fragments: StackedLaneFragment<RecordingAnnotationClip & { id: string }>[];
  laneRef: RefObject<HTMLDivElement | null>;
  onClear: () => void;
  onSweep: (ids: string[], additive: boolean) => void;
  viewport: TimelineViewportState;
}) {
  const [band, setBand] = useState<Band | null>(null);
  // The window listeners live for one gesture while these are rebuilt every
  // render, so the listeners read them through a ref.
  const latestRef = useRef({ fragments, onClear, onSweep, viewport });
  latestRef.current = { fragments, onClear, onSweep, viewport };
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
      const { fragments, viewport } = latestRef.current;
      swept = sweptAnnotationClips(fragments, box, {
        laneWidthPx: bounds.width,
        viewport,
      });
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
