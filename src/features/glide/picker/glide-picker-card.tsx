// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";

import { GlideAppIcon } from "../glide-app-icon";
import { GlideIslandBody } from "../island/glide-island-body";
import { follow } from "../island/glide-island-motion";

import { CARD, PICKER_ICON, SLOT, TILE } from "./glide-picker-types";

/** Where the destination sits on its screen. */
const TILE_PLACE = {
  borderRadius: TILE.radius,
  height: TILE.height,
  insetInlineStart: SLOT.gap,
  top: SLOT.gap,
  width: TILE.width,
};

// One screen in the picker, its own small island: the screen a light well,
// marked faintly when the window is on it, and holding the destination, a
// primary tile with the app on it, when the window is going there. The
// screen keeps its own geometry in a right-to-left window, as the picture
// does.

export function GlidePickerCard({
  iconSrc,
  isOrigin,
  isSelected,
  isShown,
  label,
  pulse,
}: {
  /** The glided app's icon, once it resolves. */
  iconSrc: null | string;
  /** The window is on this screen. */
  isOrigin: boolean;
  /** The window will go to this screen. */
  isSelected: boolean;
  isShown: boolean;
  /** Counts this card's rests; each one plays the ready breath. */
  pulse: number;
  /** Names the pick, where nothing round the card does. */
  label?: string;
}) {
  return (
    <GlideIslandBody
      isShown={isShown}
      label={label}
      pulse={pulse}
      refuses={false}
      shape={CARD}
    >
      <div
        className="absolute inset-0 flex items-center justify-center"
        dir="ltr"
      >
        <div
          className="relative bg-fill"
          style={{
            borderRadius: SLOT.radius,
            height: SLOT.height,
            width: SLOT.width,
          }}
        >
          {isOrigin ? (
            <div className="absolute bg-fill-strong" style={TILE_PLACE} />
          ) : null}
          <AnimatePresence initial={false}>
            {isSelected ? (
              <Destination iconSrc={iconSrc} key="destination" />
            ) : null}
          </AnimatePresence>
        </div>
      </div>
    </GlideIslandBody>
  );
}

function Destination({ iconSrc }: { iconSrc: null | string }) {
  return (
    <motion.div
      animate={{ opacity: 1, scale: 1 }}
      className="absolute flex items-center justify-center bg-primary"
      exit={{ opacity: 0, scale: 0.6 }}
      initial={{ opacity: 0, scale: 0.6 }}
      style={TILE_PLACE}
      transition={follow}
    >
      {iconSrc ? (
        <span
          className="relative"
          style={{ height: PICKER_ICON, width: PICKER_ICON }}
        >
          <GlideAppIcon
            className="absolute inset-0 size-full object-contain"
            src={iconSrc}
          />
        </span>
      ) : null}
    </motion.div>
  );
}
