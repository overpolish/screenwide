// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { type ReactNode, useLayoutEffect, useRef, useState } from "react";

import { motionDurations } from "../../../lib/motion";

import type { PictureSize } from "../preview/settings-preview-api";

// A screen the page draws a picture on, such as the island hanging from its
// top or Glide moving a window, scaled down so its whole subject fits the
// stage: drawn at its real size where there is room, smaller where there is
// not, never larger. The screen fills the stage edge to edge at that scale,
// so the desktop runs out to the stage's corners however much is shown, and
// what is on it is laid out in the screen's own points. The scale is
// Motion's own, so a layout animation on the screen, such as the island's
// panel keeping its place, measures in those points: under a plain CSS
// scale it read the island's sideways shift round a notch at the wrong
// size, saw the panel move on every render and kept sliding it back.
//
// A subject whose size is only known once drawn, such as an island whose
// row depends on the settings and on the display's notch, is measured as it
// is drawn, and the screen makes room for the widest it has been: a fixed
// guess at its size left a wide island under a notch cut off at the stage's
// edges. Holding the widest keeps the picture from zooming in and out as
// the island changes through its loop; while a new arrangement settles, the
// room follows the island down as well as up.

/** Room kept round a measured subject for its shadow, in points. */
const SUBJECT_ROOM = 24;
/** How long after its arrangement changes the subject is followed as it
 * shrinks, in milliseconds: the island's own reshaping is over by then. */
const SETTLES_MS = motionDurations.rearrange * 1000;

/** A subject measured on the screen. */
export type StageSubject = {
  /** Finds the subject among what the screen draws. */
  find: (screen: HTMLElement) => Element | null;
  /** Changes when the subject is arranged otherwise, such as a setting
   * turned off that narrows the island: the room it needs starts over. */
  key: string;
};

type Room = { key: string; since: number; size: PictureSize };

/** The scale that fits `fits` into the stage, never above real size. */
function scaleFor(stage: PictureSize, fits: PictureSize) {
  if (stage.width === 0 || stage.height === 0) return 1;
  return Math.min(1, stage.width / fits.width, stage.height / fits.height);
}

/** The room `found` takes on `screen`, a screen of `width` points: as wide
 * both sides of the centre as its far side reaches, since the screen grows
 * evenly round its centre, and down from the screen's top. */
function roomFor(screen: HTMLElement, found: Element, width: number) {
  const box = found.getBoundingClientRect();
  const frame = screen.getBoundingClientRect();
  if (box.width === 0 || frame.width === 0) return null;
  // Measured as drawn, mid-scale or not.
  const points = width / frame.width;
  const centre = frame.left + frame.width / 2;
  const reach = Math.max(centre - box.left, box.right - centre);
  return {
    height: (box.bottom - frame.top) * points + SUBJECT_ROOM,
    width: 2 * (reach * points + SUBJECT_ROOM),
  };
}

export function StageScreen({
  children,
  fits,
  subject,
}: {
  /** The screen at its scale, in points. */
  children: (screen: PictureSize) => ReactNode;
  /** The room the subject needs, in points, at least. */
  fits: PictureSize;
  /** A subject to measure as it is drawn, for room beyond `fits`. */
  subject?: StageSubject;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const screenRef = useRef<HTMLDivElement>(null);
  const [stage, setStage] = useState<PictureSize>({ height: 0, width: 0 });
  const [room, setRoom] = useState<Room | null>(null);

  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    // The observer reports the first size as soon as it starts watching.
    const observer = new ResizeObserver(() => {
      setStage({ height: element.clientHeight, width: element.clientWidth });
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);

  const held = room !== null && room.key === subject?.key ? room : null;
  const needs = held
    ? {
        height: Math.max(fits.height, held.size.height),
        width: Math.max(fits.width, held.size.width),
      }
    : fits;
  const scale = scaleFor(stage, needs);
  const screen = { height: stage.height / scale, width: stage.width / scale };

  // Measured after each render, before it is painted: a picture on the
  // stage is drawn from a clock, so it renders, and passes its subject
  // afresh, every frame it changes.
  useLayoutEffect(() => {
    const element = screenRef.current;
    const found = subject && element ? subject.find(element) : null;
    if (!subject || !element || !found) return;
    const measured = roomFor(element, found, screen.width);
    if (!measured) return;
    const now = performance.now();
    const since = held?.since ?? now;
    const settling = now - since < SETTLES_MS;
    const size =
      held && !settling
        ? {
            height: Math.max(held.size.height, measured.height),
            width: Math.max(held.size.width, measured.width),
          }
        : measured;
    if (
      held &&
      Math.abs(size.width - held.size.width) < 0.5 &&
      Math.abs(size.height - held.size.height) < 0.5
    )
      return;
    // Measured from the drawn page, which only exists once rendered.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setRoom({ key: subject.key, since, size });
  }, [subject, screen.width, held]);

  return (
    <div className="absolute inset-0 overflow-hidden" dir="ltr" ref={ref}>
      {stage.width > 0 ? (
        <motion.div
          className="absolute start-0 top-0"
          ref={screenRef}
          style={{
            height: screen.height,
            originX: 0,
            originY: 0,
            scale,
            width: screen.width,
          }}
        >
          {children(screen)}
        </motion.div>
      ) : null}
    </div>
  );
}
