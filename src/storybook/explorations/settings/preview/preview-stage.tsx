// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode } from "react";

import { PREVIEW_DISPLAYS } from "../../../../features/recording-controls/island/island-preview";
import { motionDurations, motionEasings } from "../../../../lib/motion";
import { cn } from "../../../../lib/styling";
import { Text } from "../../../../primitives/text/text";

import type { PreviewFollow } from "./use-preview-subject";

// A pane's picture of what its settings do. The rows are the controls; the
// stage only shows, and says the value in words under the picture, so the
// picture never has to be decoded.

/** A row that drives the stage, marked while the stage shows it. The mark is
 * the orange of where you are, in the box's inset beside the row. */
export function PreviewRow({
  children,
  follow,
  isActive,
}: {
  children: ReactNode;
  follow: PreviewFollow;
  isActive: boolean;
}) {
  return (
    <div className="relative" {...follow}>
      <span
        aria-hidden
        className={cn(
          "absolute top-1/2 h-4 w-[3px] -translate-y-1/2 rounded-full bg-emphasis transition-opacity",
          isActive ? "opacity-100" : "opacity-0",
        )}
        style={{ insetInlineStart: "calc(var(--spacing-inset) * -1 + 3px)" }}
      />
      {children}
    </div>
  );
}

/** The picture, and what it shows in words: the row's name and its value.
 * The picture crossfades when the row it shows changes. */
export function PreviewStage({
  children,
  sceneKey,
  title,
  value,
}: {
  children: ReactNode;
  /** Changes when the stage moves to another row's picture. */
  sceneKey: string;
  title: string;
  value: string;
}) {
  return (
    <div className="flex flex-col gap-inline">
      <div
        aria-hidden
        className="relative h-28 overflow-hidden rounded-panel bg-recess"
        inert
      >
        <AnimatePresence initial={false}>
          <motion.div
            animate={{ opacity: 1 }}
            className="absolute inset-0"
            exit={{ opacity: 0 }}
            initial={{ opacity: 0 }}
            key={sceneKey}
            transition={{
              duration: motionDurations.travel,
              ease: motionEasings.out,
            }}
          >
            {children}
          </motion.div>
        </AnimatePresence>
      </div>
      <div className="flex items-baseline justify-between gap-separate px-inset-sm">
        <Text variant="label">{title}</Text>
        <Text className="text-end" variant="caption">
          {value}
        </Text>
      </div>
    </div>
  );
}

/** The top of a screen, for the island to hang from: the desktop's
 * wallpaper, as the island is seen over it. */
export function ScreenTop({ children }: { children?: ReactNode }) {
  return (
    <div className="absolute inset-0" dir="ltr">
      <img
        alt=""
        className="absolute inset-0 size-full object-cover opacity-80"
        src={PREVIEW_DISPLAYS[0].thumbnail}
      />
      {children}
    </div>
  );
}
