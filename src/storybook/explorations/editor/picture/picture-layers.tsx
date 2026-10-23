// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { type Element } from "../../ideas/ideas-data";
import { type Mark } from "../../ideas/look-pens-model";
import { px } from "../../ideas/point-geometry";
import { type Layout, SPEECH } from "../editor-data";

// What is laid over the recorded screen: the selection's handles, what a
// pen would land on, the camera as the step's scene places it, and the
// caption being spoken.

export function Handles({ mark }: { mark: Mark }) {
  const rect = px(mark.element);
  const ends =
    mark.from && mark.to
      ? [mark.from, mark.to]
      : [
          { x: rect.x, y: rect.y },
          { x: rect.x + rect.w, y: rect.y + rect.h },
        ];
  return ends.map(({ x, y }) => (
    <span
      className="pointer-events-none absolute size-3 -translate-1/2 rounded-full border-2 border-emphasis bg-white"
      key={`${String(x)}-${String(y)}`}
      style={{ insetInlineStart: x, top: y }}
    />
  ));
}

export function LockOn({ element }: { element: Element }) {
  const rect = px(element);
  return (
    <motion.span
      animate={{
        borderRadius: element.radius + 3,
        height: rect.h + 6,
        width: rect.w + 6,
        x: rect.x - 3,
        y: rect.y - 3,
      }}
      className="pointer-events-none absolute start-0 top-0 border-2 border-emphasis"
      initial={false}
      transition={{ bounce: 0.2, duration: 0.3, type: "spring" }}
    />
  );
}

/** The presenter, where the step's scene puts them: beside the screen, or
 * small in its corner. */
export function Camera({ layout }: { layout: Layout }) {
  if (layout !== "side" && layout !== "pip") return null;
  const isSide = layout === "side";
  const size = isSide ? 168 : 88;
  return (
    <motion.span
      animate={{ height: size, width: size }}
      className="pointer-events-none absolute end-4 bottom-4 overflow-hidden rounded-full shadow-xl ring-4 ring-white"
      initial={false}
      style={{
        background:
          "radial-gradient(circle at 50% 35%, #f3c9a8 0 18%, transparent 19%), radial-gradient(circle at 50% 100%, #4b6cb7 0 42%, transparent 43%), linear-gradient(#cfd9e6, #a9b8cc)",
      }}
    />
  );
}

export function Caption({ seconds }: { seconds: number }) {
  const line = SPEECH.find(
    ({ end, start }) => seconds >= start && seconds < end,
  );
  if (!line) return null;
  return (
    <span className="pointer-events-none absolute inset-x-0 bottom-5 flex justify-center">
      <span className="max-w-[80%] rounded-[8px] bg-black/75 px-3 py-1.5 text-center text-[15px] font-medium text-white">
        {line.text}
      </span>
    </span>
  );
}
