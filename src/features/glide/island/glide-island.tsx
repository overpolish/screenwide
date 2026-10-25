// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Lock, Move } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import { t } from "../../../i18n/i18n";
import { describeDestination } from "../glide-region-label";

import { GlideIslandBody } from "./glide-island-body";
import { GlideIslandIcon, GlideIslandMap } from "./glide-island-map";
import { layOutMap } from "./glide-island-map-layout";
import { swap } from "./glide-island-motion";
import {
  GLYPH_SHAPE,
  MAP,
  MAP_ICON,
  MAP_INSET,
  MAP_SHAPE,
} from "./glide-island-shapes";
import { type GlideIslandProps, hasDestination } from "./glide-island-types";

// The Glide island: only as big as what it has to say. It comes out round,
// holding the app it has caught; it opens into a small screen once there is
// a destination, the icon riding across onto it; and as the window is sent
// it closes round a check before it goes. A window that will not move gets
// the lock, and a shake.

export function GlideIsland(props: GlideIslandProps) {
  const { fit, iconSrc, pending, phase, pulse, region } = props;
  const isOpen = phase === "gliding" && hasDestination(props);
  const shape = isOpen ? MAP_SHAPE : GLYPH_SHAPE;
  const layout = isOpen ? layOutMap(MAP, { fit, pending, region }) : null;

  const glyph =
    phase === "locked"
      ? "lock"
      : phase === "committed"
        ? "check"
        : iconSrc
          ? null
          : "move";

  return (
    <GlideIslandBody
      isShown={phase !== "idle"}
      label={
        phase === "locked"
          ? t("glide-locked")
          : describeDestination(region, pending)
      }
      pulse={pulse}
      refuses={phase === "locked"}
      shape={shape}
    >
      <AnimatePresence initial={false}>
        {isOpen ? (
          <motion.div
            key="map"
            {...swap}
            className="absolute inset-0 flex items-center justify-center"
          >
            <GlideIslandMap
              fit={fit}
              pending={pending}
              region={region}
              size={MAP}
            />
          </motion.div>
        ) : null}
      </AnimatePresence>
      <AnimatePresence initial={false} mode="popLayout">
        {glyph && !isOpen ? (
          <motion.div
            key={glyph}
            {...swap}
            className="absolute inset-0 flex items-center justify-center"
          >
            <Glyph glyph={glyph} />
          </motion.div>
        ) : null}
      </AnimatePresence>
      {/* Drawn over the black rather than in the screen, so the one icon
          travels from the middle onto its destination as the island opens.
          The screen's geometry stays left to right, as the map's does. */}
      <div className="absolute inset-0" dir="ltr">
        <AnimatePresence initial={false}>
          {iconSrc && phase === "gliding" ? (
            <motion.div exit={swap.exit} key="icon">
              <GlideIslandIcon
                at={
                  layout
                    ? {
                        x: MAP_INSET + layout.focus.x,
                        y: MAP_INSET + layout.focus.y,
                      }
                    : { x: shape.width / 2, y: shape.height / 2 }
                }
                perches={layout?.isMinimizing ?? false}
                size={MAP_ICON}
                src={iconSrc}
              />
            </motion.div>
          ) : null}
        </AnimatePresence>
      </div>
    </GlideIslandBody>
  );
}

function Glyph({ glyph }: { glyph: "check" | "lock" | "move" }) {
  if (glyph === "check") return <Check aria-hidden className="size-icon" />;
  if (glyph === "lock") {
    return <Lock aria-hidden className="size-icon text-fg-secondary" />;
  }
  return <Move aria-hidden className="size-icon text-fg-secondary" />;
}
