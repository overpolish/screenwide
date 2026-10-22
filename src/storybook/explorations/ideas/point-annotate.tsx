// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { useState } from "react";

import { Button } from "../../../primitives/button/button";
import { PillGroup } from "../../../primitives/pill-group/pill-group";

import { IdeaFrame } from "./idea-frame";
import { clock, type Element } from "./ideas-data";
import { MockApp } from "./mock-app";
import {
  appearsAt,
  elementAt,
  H,
  nearestEdge,
  px,
  SNAP,
  spring,
  W,
} from "./point-geometry";
import { Arrow, Callout } from "./point-marks";

// Annotations that know what they point at. The picture is read as the UI it
// shows, so a pointer locks on to a button or card, a counter lands on its
// corner, a spotlight takes its shape, an arrow's head finds its edge, and
// each one appears when that element was used in the recording.

type Tool = "arrow" | "counter" | "spotlight";
/** Each placed annotation carries the moment it was placed, as its key. */
type Placed = { id: number } & (
  | { element: Element; kind: "counter"; number: number }
  | { element: Element; kind: "spotlight" }
  | {
      element: Element;
      from: { x: number; y: number };
      kind: "arrow";
      to: { x: number; y: number };
    }
);

export function PointAnnotate() {
  const [tool, setTool] = useState<Tool>("counter");
  const [hover, setHover] = useState<Element | undefined>();
  const [placed, setPlaced] = useState<Placed[]>([]);
  const [drag, setDrag] = useState<{
    from: { x: number; y: number };
    to: { x: number; y: number };
    snapped?: Element;
  } | null>(null);
  const last = placed[placed.length - 1] as Placed | undefined;
  const lastAction = last ? appearsAt(last.element) : undefined;
  const locked = drag?.snapped ?? hover;
  const lockedRect = locked ? px(locked) : null;
  const spotlight = [...placed]
    .reverse()
    .find((item) => item.kind === "spotlight");

  const point = (event: React.PointerEvent) => {
    const box = event.currentTarget.getBoundingClientRect();
    return { x: event.clientX - box.left, y: event.clientY - box.top };
  };

  return (
    <IdeaFrame
      hint="Pick a tool, then hover the screen: what is under the pointer locks on. Click to place a counter or spotlight; drag for an arrow and let its head find an edge."
      pitch="Annotations that know what they point at. The picture is read as the interface it shows: counters land on corners, spotlights take the element's own shape, arrows find edges, and each appears when that element was used."
      title="Point, don't draw"
    >
      <div className="flex flex-col items-center gap-separate">
        <div className="flex w-full items-center gap-separate">
          <PillGroup
            aria-label="Annotation"
            items={[
              { id: "counter", label: "Counter" },
              { id: "spotlight", label: "Spotlight" },
              { id: "arrow", label: "Arrow" },
            ]}
            onSelectionChange={(id) => {
              setTool(id as Tool);
            }}
            selected={tool}
          />
          <span className="grow text-caption text-fg-tertiary">
            {last
              ? lastAction
                ? `Appears at ${clock(lastAction.at)}, when “${last.element.label}” was ${lastAction.kind === "type" ? "typed in" : "clicked"}`
                : `Appears at the playhead: “${last.element.label}” is never used in the recording`
              : "Nothing placed yet"}
          </span>
          <Button
            onPress={() => {
              setPlaced([]);
            }}
            variant="ghost"
          >
            Clear
          </Button>
        </div>
        <div
          className="relative cursor-crosshair touch-none overflow-hidden rounded-part shadow-lg"
          dir="ltr"
          onPointerDown={(event) => {
            const at = point(event);
            if (tool === "arrow") {
              event.currentTarget.setPointerCapture(event.pointerId);
              setDrag({ from: at, to: at });
              return;
            }
            const element = elementAt(at.x, at.y);
            if (!element) return;
            const id = event.timeStamp;
            setPlaced((current) =>
              tool === "counter"
                ? [
                    ...current,
                    {
                      element,
                      id,
                      kind: "counter",
                      number:
                        current.filter(({ kind }) => kind === "counter")
                          .length + 1,
                    },
                  ]
                : [
                    ...current.filter(({ kind }) => kind !== "spotlight"),
                    { element, id, kind: "spotlight" },
                  ],
            );
          }}
          onPointerLeave={() => {
            setHover(undefined);
          }}
          onPointerMove={(event) => {
            const at = point(event);
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
            setHover(elementAt(at.x, at.y));
          }}
          onPointerUp={(event) => {
            if (drag?.snapped) {
              const { from, snapped, to } = drag;
              const id = event.timeStamp;
              setPlaced((current) => [
                ...current,
                { element: snapped, from, id, kind: "arrow", to },
              ]);
            }
            setDrag(null);
          }}
          style={{ height: H, width: W }}
        >
          <MockApp className="size-full" />
          <svg
            aria-hidden
            className="pointer-events-none absolute inset-0"
            height={H}
            width={W}
          >
            <defs>
              <filter height="200%" id="lift" width="200%" x="-50%" y="-50%">
                <feDropShadow
                  dx={0}
                  dy={1.5}
                  floodOpacity={0.3}
                  stdDeviation={1}
                />
                <feDropShadow
                  dx={0}
                  dy={5}
                  floodOpacity={0.18}
                  stdDeviation={6}
                />
              </filter>
              {spotlight ? (
                <mask id="spot">
                  <rect fill="#fff" height={H} width={W} />
                  <motion.rect
                    animate={{
                      ...px(spotlight.element),
                      height: px(spotlight.element).h + 12,
                      rx: spotlight.element.radius + 6,
                      width: px(spotlight.element).w + 12,
                      x: px(spotlight.element).x - 6,
                      y: px(spotlight.element).y - 6,
                    }}
                    fill="#000"
                    initial={false}
                    transition={spring}
                  />
                </mask>
              ) : null}
            </defs>
            {spotlight ? (
              <motion.rect
                animate={{ opacity: 1 }}
                fill="rgba(0,0,0,0.45)"
                height={H}
                initial={{ opacity: 0 }}
                mask="url(#spot)"
                width={W}
              />
            ) : null}
            {placed.map((item) =>
              item.kind === "counter" ? (
                <Callout
                  element={item.element}
                  key={item.id}
                  number={item.number}
                />
              ) : item.kind === "arrow" ? (
                <Arrow from={item.from} key={item.id} to={item.to} />
              ) : null,
            )}
            {drag ? <Arrow from={drag.from} to={drag.to} /> : null}
          </svg>
          {/* The lock-on: springs from element to element, in its shape. */}
          {lockedRect && locked ? (
            <motion.span
              animate={{
                borderRadius: locked.radius + 3,
                height: lockedRect.h + 6,
                scale: drag?.snapped ? [1, 1.04, 1] : 1,
                width: lockedRect.w + 6,
                x: lockedRect.x - 3,
                y: lockedRect.y - 3,
              }}
              className="pointer-events-none absolute start-0 top-0 border-2 border-emphasis"
              initial={false}
              transition={spring}
            />
          ) : null}
        </div>
      </div>
    </IdeaFrame>
  );
}
