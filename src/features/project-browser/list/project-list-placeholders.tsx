// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../lib/motion";
import { cn } from "../../../lib/styling";
import { Text } from "../../../primitives/text/text";

import { CARD_COLUMNS } from "./project-grid";

/** In place of the cards when there are none to show: a glyph in a soft
 * well, what is going on in a few words, and the one thing that would help,
 * if any. */
export function ListMessage({
  action,
  glyph,
  title,
}: {
  glyph: ReactNode;
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
      <Text className="max-w-72" variant="heading">
        {title}
      </Text>
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
      <div className={cn(CARD_COLUMNS, "pt-1")}>
        {Array.from({ length: 6 }, (_, index) => (
          <div
            aria-hidden
            className="flex animate-pulse flex-col gap-inline"
            key={index}
            style={{ animationDelay: `${String(index * 80)}ms` }}
          >
            <div className="aspect-video rounded-panel bg-fill" />
            <div className="h-4" />
            <div className="h-3 w-2/3 rounded-sm bg-fill" />
            <div className="h-2.5 w-1/3 rounded-sm bg-fill" />
          </div>
        ))}
      </div>
    </div>
  );
}
