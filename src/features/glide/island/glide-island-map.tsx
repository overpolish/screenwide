// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { AnimatePresence, motion } from "motion/react";

import { GlideAppIcon } from "../glide-app-icon";

import {
  type GlideMapBox,
  type GlideMapSize,
  layOutMap,
} from "./glide-island-map-layout";
import { follow } from "./glide-island-motion";

import type { GlideIslandState } from "./glide-island-types";

// The map's root is left to right, so start is left and end right.
const boxStyle = ({ height, radius, width, x, y }: GlideMapBox) => ({
  borderEndEndRadius: `${String(radius.bottomEnd)}px`,
  borderEndStartRadius: `${String(radius.bottomStart)}px`,
  borderStartEndRadius: `${String(radius.topEnd)}px`,
  borderStartStartRadius: `${String(radius.topStart)}px`,
  height,
  width,
  x,
  y,
});

// A small screen in the black: the screen a light well, and the destination
// a primary tile that follows the gesture on a spring. The screen keeps its
// own geometry in a right-to-left window, as the picture does.

export function GlideIslandMap({
  fit,
  iconSize = 0,
  iconSrc = null,
  pending,
  region,
  size,
}: Pick<GlideIslandState, "fit" | "pending" | "region"> & {
  size: GlideMapSize;
  iconSize?: number;
  /** The app's icon, laid on its destination; leave it out to draw it
   * elsewhere, or not at all. */
  iconSrc?: null | string;
}) {
  const layout = layOutMap(size, { fit, pending, region });
  const { height, radius, width } = size;

  return (
    <div className="relative" dir="ltr" style={{ height, width }}>
      <div
        className="absolute inset-0 bg-fill"
        style={{ borderRadius: radius }}
      />
      <AnimatePresence>
        {layout?.requested ? (
          <Neutral box={layout.requested} key="requested" />
        ) : null}
        {layout?.reached ? (
          <Neutral box={layout.reached} key="reached" />
        ) : null}
      </AnimatePresence>
      <motion.div
        animate={
          layout?.tile
            ? { ...boxStyle(layout.tile), opacity: 1 }
            : { opacity: 0 }
        }
        className="absolute start-0 top-0 bg-primary"
        initial={false}
        transition={follow}
      />
      {iconSrc && layout ? (
        <GlideIslandIcon
          at={layout.focus}
          perches={layout.isMinimizing}
          size={iconSize}
          src={iconSrc}
        />
      ) : null}
    </div>
  );
}

function Neutral({ box }: { box: GlideMapBox }) {
  return (
    <motion.div
      animate={{ ...boxStyle(box), opacity: 1 }}
      className="absolute start-0 top-0 bg-fill-strong"
      exit={{ opacity: 0 }}
      initial={{ ...boxStyle(box), opacity: 0 }}
      transition={follow}
    />
  );
}

/**
 * The glided app's icon, centred on the point it names; perched, it stands on
 * that point instead, as an app in the Dock stands on it, a size down.
 */
function GlideIslandIcon({
  at,
  perches,
  size,
  src,
}: {
  at: { x: number; y: number };
  perches: boolean;
  size: number;
  src: string;
}) {
  const scale = perches ? 0.75 : 1;
  return (
    <motion.div
      animate={{
        scale,
        x: at.x - size / 2,
        // Scaled round its centre, it stands on the Dock with a little air.
        y: perches ? at.y - (size * (1 + scale)) / 2 - 5 : at.y - size / 2,
      }}
      className="absolute start-0 top-0"
      initial={false}
      style={{ height: size, width: size }}
      transition={follow}
    >
      <GlideAppIcon
        className="absolute inset-0 size-full object-contain"
        src={src}
      />
    </motion.div>
  );
}
