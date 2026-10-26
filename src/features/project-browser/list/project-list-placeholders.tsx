// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { Text } from "../../../primitives/text/text";

/** In place of the cards when there are none to show: a glyph in a soft
 * well, what is going on, and the one thing that would help, if any. */
export function ListMessage({
  action,
  glyph,
  text,
  title,
}: {
  glyph: ReactNode;
  text: string;
  title: string;
  action?: ReactNode;
}) {
  const reducedMotion = useReducedMotion() ?? false;
  return (
    <motion.div
      animate={{ opacity: 1, y: 0 }}
      className="flex grow flex-col items-center justify-center gap-separate pe-inset pb-inset text-center"
      initial={{ opacity: 0, y: reducedMotion ? 0 : 6 }}
      transition={{ duration: motionDurations.travel, ease: motionEasings.out }}
    >
      <span className="flex size-12 items-center justify-center rounded-full bg-fill text-fg-secondary [&_svg]:size-icon-lg">
        {glyph}
      </span>
      <div className="flex max-w-72 flex-col gap-1">
        <Text variant="heading">{title}</Text>
        <Text className="text-fg-secondary" variant="body">
          {text}
        </Text>
      </div>
      {action}
    </motion.div>
  );
}

/** Cards' shapes while a source is read, so the window keeps its layout and
 * the real cards land where these were. */
export function LoadingGrid() {
  return (
    <div
      aria-busy
      aria-label={t("project-browser-loading")}
      className="flex flex-col gap-separate pe-inset"
      role="status"
    >
      {/* Where the toolbar will be, so the cards land in their places. */}
      <div className="h-control-height" />
      <div className="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-x-separate gap-y-group pt-1">
        {Array.from({ length: 6 }, (_, index) => (
          <div
            aria-hidden
            className="flex animate-pulse flex-col gap-inline"
            key={index}
            style={{ animationDelay: `${String(index * 80)}ms` }}
          >
            <div className="aspect-video rounded-panel bg-fill" />
            <div className="h-3 w-2/3 rounded-sm bg-fill" />
            <div className="h-2.5 w-1/3 rounded-sm bg-fill" />
          </div>
        ))}
      </div>
    </div>
  );
}
