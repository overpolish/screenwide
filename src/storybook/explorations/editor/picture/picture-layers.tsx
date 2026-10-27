// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { type Element } from "../../ideas/ideas-data";
import { type Mark } from "../../ideas/look-pens-model";
import { H, px, W } from "../../ideas/point-geometry";
import { type ScenePreset } from "../editor-data";
import { lineAt, type Recording } from "../recording";

// What is laid over the recorded screen: the selection's handles, what a
// pen would land on, the camera as the scene places it, and the
// caption being spoken.

export function Handles({
  mark,
}: {
  mark: Pick<Mark, "element" | "from" | "to">;
}) {
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

/** Where each arrangement puts the presenter, in px of the picture. */
const CAMERA_BOX: Partial<
  Record<
    ScenePreset,
    {
      bottom: number;
      end: number;
      height: number;
      radius: number;
      width: number;
    }
  >
> = {
  camera: { bottom: 0, end: 0, height: H, radius: 0, width: W },
  pip: { bottom: 16, end: 16, height: 88, radius: 44, width: 88 },
  side: { bottom: 72, end: 16, height: 306, radius: 14, width: 182 },
  stacked: {
    bottom: 14,
    end: (W - 168) / 2,
    height: 112,
    radius: 14,
    width: 168,
  },
  // A zoom only reframes the screen: the camera stays where the recording
  // put it.
  zoom: { bottom: 16, end: 16, height: 88, radius: 44, width: 88 },
};

/** The presenter, where the scene puts them: beside the screen, under it,
 * in its corner or filling the picture. Changing scene, the frame reshapes
 * from one place to the next. */
export function Camera({ preset }: { preset: ScenePreset }) {
  const box = CAMERA_BOX[preset];
  if (!box) return null;
  return (
    <motion.span
      animate={{
        borderRadius: box.radius,
        bottom: box.bottom,
        height: box.height,
        insetInlineEnd: box.end,
        width: box.width,
      }}
      className="pointer-events-none absolute overflow-hidden shadow-xl ring-4 ring-white"
      initial={false}
      style={{
        background:
          "radial-gradient(circle at 50% 38%, #f3c9a8 0 16%, transparent 17%), radial-gradient(ellipse 60% 40% at 50% 100%, #4b6cb7 0 98%, transparent 100%), linear-gradient(#cfd9e6, #a9b8cc)",
      }}
      transition={{ duration: 0.4, ease: "easeOut" }}
    />
  );
}

export function Caption({
  recording,
  seconds,
}: {
  recording: Recording;
  seconds: number;
}) {
  const line = lineAt(recording, seconds);
  if (!line) return null;
  return (
    <span className="pointer-events-none absolute inset-x-0 bottom-5 flex justify-center">
      <span className="max-w-[80%] rounded-[8px] bg-black/75 px-3 py-1.5 text-center text-[15px] font-medium text-white">
        {line.text}
      </span>
    </span>
  );
}
