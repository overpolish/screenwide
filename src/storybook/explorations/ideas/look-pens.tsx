// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";

import { cn } from "../../../lib/styling";

import { IdeaFrame } from "./idea-frame";
import { type Element } from "./ideas-data";
import {
  anchorOf,
  corner,
  type Mark,
  newMark,
  type Point,
  SEEDED,
  swatchOf,
} from "./look-pens-model";
import { LookPicker } from "./look-picker";
import { lookById, type LookId, type Pen } from "./looks";
import { MockApp } from "./mock-app";
import { MorePanel } from "./more-panel";
import { PenToolbar } from "./pen-toolbar";
import { PlacedMarks } from "./placed-marks";
import { elementAt, H, nearestEdge, px, SNAP, W } from "./point-geometry";
import { ScreenRipple } from "./screen-ripple";
import { SelectionBar } from "./selection-bar";

// Styling without an inspector: the project's look decides how every mark is
// drawn; a pen is ready to use the moment it is picked; a selected mark
// offers its three common choices beside it, and everything else is behind
// More. Pens place by pointing, as in Point, don't draw, and a counter lands
// with a ripple through the picture under it.

export function LookPens() {
  const [lookId, setLookId] = useState<LookId>("product");
  const [marks, setMarks] = useState<Mark[]>(SEEDED);
  const [pen, setPen] = useState<Pen | "select">("select");
  const [selected, setSelected] = useState<number | null>(3);
  const [hover, setHover] = useState<Element | undefined>();
  const [drag, setDrag] = useState<{
    from: Point;
    to: Point;
    snapped?: Element;
  } | null>(null);
  const [ripple, setRipple] = useState<{ centre: Point; id: number } | null>(
    null,
  );
  const [isMoreOpen, setIsMoreOpen] = useState(false);
  const chosen = marks.find(({ id }) => id === selected);
  const lockOn = drag?.snapped ?? hover;
  const look = lookById(lookId);

  const update = (change: Partial<Mark>) => {
    setMarks((current) =>
      current.map((mark) =>
        mark.id === selected ? { ...mark, ...change } : mark,
      ),
    );
  };
  const place = (mark: Mark) => {
    setMarks((current) => [...current, mark]);
    setSelected(mark.id);
    setPen("select");
    if (mark.pen === "counter")
      setRipple({ centre: corner(mark.element).at, id: mark.id });
  };
  const pointer = (event: React.PointerEvent) => {
    const box = event.currentTarget.getBoundingClientRect();
    return { x: event.clientX - box.left, y: event.clientY - box.top };
  };

  return (
    <IdeaFrame
      hint="Change the look and everything restyles. Pick a pen and point at the screen to place it. Click a mark to see its three common choices; More holds the rest."
      pitch="Styling without an inspector. The project's look decides how every mark is drawn, a pen is ready the moment it is picked, and a selected mark offers its colour, size and look beside it, with everything else one press away."
      title="Looks and pens"
    >
      <LookPicker onChange={setLookId} value={lookId} />
      <div className="flex items-start gap-separate">
        <PenToolbar
          lookId={lookId}
          onChange={(next) => {
            setPen(next);
            setSelected(null);
          }}
          value={pen}
        />
        <div
          className="relative shrink-0"
          dir="ltr"
          style={{ height: H, width: W }}
        >
          <div
            className={cn(
              "relative touch-none overflow-hidden rounded-part shadow-lg",
              pen === "select" ? "cursor-default" : "cursor-crosshair",
            )}
            dir="ltr"
            onPointerDown={(event) => {
              const at = pointer(event);
              if (pen === "select") {
                setSelected(null);
                return;
              }
              if (pen === "arrow") {
                event.currentTarget.setPointerCapture(event.pointerId);
                setDrag({ from: at, to: at });
                return;
              }
              const element = elementAt(at.x, at.y);
              if (element)
                place(newMark({ element, id: event.timeStamp, pen }));
            }}
            onPointerLeave={() => {
              setHover(undefined);
            }}
            onPointerMove={(event) => {
              const at = pointer(event);
              if (drag) {
                const near = nearestEdge(at.x, at.y);
                const snapped = near.distance < SNAP ? near : undefined;
                setDrag({
                  ...drag,
                  snapped: snapped?.element,
                  to: snapped ? snapped.point : at,
                });
                return;
              }
              setHover(pen === "select" ? undefined : elementAt(at.x, at.y));
            }}
            onPointerUp={(event) => {
              if (drag?.snapped) {
                place(
                  newMark({
                    element: drag.snapped,
                    from: drag.from,
                    id: event.timeStamp,
                    pen: "arrow",
                    to: drag.to,
                  }),
                );
              }
              setDrag(null);
            }}
            style={{ height: H, width: W }}
          >
            <MockApp className="size-full" />
            {ripple ? (
              <ScreenRipple centre={ripple.centre} key={ripple.id}>
                <MockApp className="size-full" />
              </ScreenRipple>
            ) : null}
            <PlacedMarks
              draft={drag}
              isSelecting={pen === "select"}
              lookId={lookId}
              marks={marks}
              onSelect={setSelected}
            />
            {lockOn ? (
              <motion.span
                animate={{
                  borderRadius: lockOn.radius + 3,
                  height: px(lockOn).h + 6,
                  width: px(lockOn).w + 6,
                  x: px(lockOn).x - 3,
                  y: px(lockOn).y - 3,
                }}
                className="pointer-events-none absolute start-0 top-0 border-2 border-emphasis"
                initial={false}
                transition={{ bounce: 0.2, duration: 0.3, type: "spring" }}
              />
            ) : null}
          </div>
          <AnimatePresence>
            {chosen ? (
              <div
                className="absolute z-10"
                key={chosen.id}
                style={{
                  // Kept clear of the picture's sides, and below the mark
                  // when there is no room above it.
                  insetInlineStart: Math.min(
                    Math.max(anchorOf(chosen).x, 230),
                    W - 230,
                  ),
                  top: anchorOf(chosen).y,
                  transform:
                    anchorOf(chosen).y < 80
                      ? "translate(-50%, 44px)"
                      : "translate(-50%, calc(-100% - 14px))",
                }}
              >
                <SelectionBar
                  look={look}
                  lookOverride={chosen.lookOverride}
                  onLook={(id) => {
                    update({ lookOverride: id });
                  }}
                  onMore={() => {
                    setIsMoreOpen(true);
                  }}
                  onSize={(size) => {
                    update({ size });
                  }}
                  onSwatch={(swatch) => {
                    update({ swatch });
                  }}
                  size={chosen.size}
                  swatch={swatchOf(chosen, lookId)}
                />
              </div>
            ) : null}
          </AnimatePresence>
        </div>
        <AnimatePresence>
          {isMoreOpen && chosen ? (
            <MorePanel
              onClose={() => {
                setIsMoreOpen(false);
              }}
              pen={chosen.pen}
            />
          ) : null}
        </AnimatePresence>
      </div>
    </IdeaFrame>
  );
}
