// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";

import { type Placement } from "../../../../bindings/Placement";
import { cn } from "../../../../lib/styling";
import { type KeysSettings } from "../model/use-overlays";
import { type Recording } from "../recording";

// A shortcut pressed while recording, drawn on the picture as its keys for a
// moment after it is pressed, where and how the Shortcuts options say: the
// stand-in for the app's keyboard effect, which the compositor draws.

/** How long a shortcut stays on the picture, in seconds. */
const SHOWN_S = 1.6;
const MODIFIERS = "⌘⌥⇧⌃";
const SCALE = {
  "extra-large": 2,
  giant: 5,
  huge: 3,
  large: 1.5,
  standard: 1,
} as const;

/** The keys of a shortcut's label: each modifier it starts with, then the
 * rest as one key. */
const keysOf = (label: string) => {
  const keys: string[] = [];
  let rest = label;
  while (rest && MODIFIERS.includes(rest.charAt(0))) {
    keys.push(rest.charAt(0));
    rest = rest.slice(1);
  }
  return rest ? [...keys, rest] : keys;
};

/** The box's place on the picture, kept a little in from its edges. The
 * picture is laid out left to right, so its start is its left. */
const placeOf = (position: Placement) => {
  const [vertical, horizontal] = position.includes("-")
    ? position.split("-")
    : ["top", "bottom"].includes(position)
      ? [position, "center"]
      : ["center", position];
  return cn(
    "absolute flex",
    vertical === "top"
      ? "top-6"
      : vertical === "bottom"
        ? "bottom-6"
        : "inset-y-0 items-center",
    horizontal === "left"
      ? "start-6"
      : horizontal === "right"
        ? "end-6"
        : "inset-x-0 justify-center",
  );
};

export function KeysOverlay({
  recording,
  seconds,
  settings,
}: {
  recording: Recording;
  seconds: number;
  settings: KeysSettings;
}) {
  const shortcut = recording.actions.find(
    ({ at, kind }) =>
      kind === "keys" && seconds >= at && seconds < at + SHOWN_S,
  );
  const from =
    settings.animation === "pop"
      ? { opacity: 0, scale: 0.6 }
      : settings.animation === "fade"
        ? { opacity: 0 }
        : { opacity: 1 };
  return (
    <span className={cn("pointer-events-none", placeOf(settings.position))}>
      <AnimatePresence>
        {shortcut ? (
          <motion.span
            animate={{ opacity: 1, scale: 1 }}
            className="flex origin-center gap-1"
            exit={settings.animation === "none" ? { opacity: 0 } : from}
            initial={from}
            key={shortcut.at}
            style={{ zoom: SCALE[settings.size] }}
            transition={
              settings.animation === "pop"
                ? { bounce: 0.4, duration: 0.35, type: "spring" }
                : { duration: settings.animation === "none" ? 0 : 0.2 }
            }
          >
            {keysOf(shortcut.label).map((key) => (
              <span
                className={cn(
                  "flex h-9 min-w-9 items-center justify-center rounded-[8px] px-2 text-[17px] font-semibold shadow-lg",
                  settings.appearance === "dark"
                    ? "bg-black/80 text-white"
                    : "bg-white text-black",
                )}
                key={key}
              >
                {key}
              </span>
            ))}
          </motion.span>
        ) : null}
      </AnimatePresence>
    </span>
  );
}
