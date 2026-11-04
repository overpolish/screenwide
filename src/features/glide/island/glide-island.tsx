// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock, Move } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import { t } from "../../../i18n/i18n";
import { GlideAppIcon } from "../glide-app-icon";
import { describeDestination } from "../glide-region-label";

import { GlideIslandBody } from "./glide-island-body";
import { GlideIslandMap } from "./glide-island-map";
import { swap } from "./glide-island-motion";
import { GLYPH_SHAPE, MAP, MAP_ICON, MAP_SHAPE } from "./glide-island-shapes";
import { type GlideIslandProps, hasDestination } from "./glide-island-types";

// The Glide island: only as big as what it has to say. It comes out round,
// holding the app it has caught, and opens into a small screen, the app on
// its destination, once there is one. It follows a hand, so it fades in
// quickly with what it has to show and changes look at once
// (`glide-island-motion.ts`). A window that will not move gets the lock, and
// a shake. Sent, it goes at once.

export function GlideIsland(props: GlideIslandProps) {
  const { fit, iconSrc, pending, phase, pulse, region } = props;
  const isShown = phase === "gliding" || phase === "locked";
  const isOpen = phase === "gliding" && hasDestination(props);
  const look = isOpen
    ? "map"
    : phase === "locked"
      ? "lock"
      : iconSrc
        ? "app"
        : "move";
  return (
    <GlideIslandBody
      isShown={isShown}
      label={
        phase === "locked"
          ? t("glide-locked")
          : describeDestination(region, pending)
      }
      pulse={pulse}
      refuses={phase === "locked"}
      shape={look === "map" ? MAP_SHAPE : GLYPH_SHAPE}
    >
      <AnimatePresence initial={false}>
        <motion.div
          key={look}
          {...swap}
          className="absolute inset-0 flex items-center justify-center"
        >
          <Look
            fit={fit}
            iconSrc={iconSrc}
            look={look}
            pending={pending}
            region={region}
          />
        </motion.div>
      </AnimatePresence>
    </GlideIslandBody>
  );
}

function Look({
  fit,
  iconSrc,
  look,
  pending,
  region,
}: Pick<GlideIslandProps, "fit" | "iconSrc" | "pending" | "region"> & {
  look: "app" | "lock" | "map" | "move";
}) {
  if (look === "map") {
    return (
      <GlideIslandMap
        fit={fit}
        iconSize={MAP_ICON}
        iconSrc={iconSrc}
        pending={pending}
        region={region}
        size={MAP}
      />
    );
  }
  if (look === "app" && iconSrc) {
    return (
      <div className="relative" style={{ height: MAP_ICON, width: MAP_ICON }}>
        <GlideAppIcon
          className="absolute inset-0 size-full object-contain"
          src={iconSrc}
        />
      </div>
    );
  }
  if (look === "lock") {
    return <Lock aria-hidden className="size-icon text-fg-secondary" />;
  }
  return <Move aria-hidden className="size-icon text-fg-secondary" />;
}
