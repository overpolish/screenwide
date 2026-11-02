// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode } from "react";

import { TOOL_PICTURE } from "../preview/scenes/scene-kit";
import { clearIn } from "../settings-motion";

import type { PreviewFollow } from "./use-preview-subject";

// A pane's picture of what its settings do, pinned above its rows. The rows
// are the controls and name themselves, so the picture only shows. It is as
// tall as a tool's picture, so moving between rows never changes its size,
// and one row's picture clears into the next's as the glass of an island
// does, rather than cutting.

export function PreviewStage({
  children,
  subject,
}: {
  children: ReactNode;
  /** Changes when the picture moves to another row's. */
  subject: string;
}) {
  return (
    <div
      aria-hidden
      className="relative overflow-hidden rounded-panel bg-recess"
      inert
      style={{ height: TOOL_PICTURE.height }}
    >
      <AnimatePresence initial={false}>
        <motion.div key={subject} {...clearIn} className="absolute inset-0">
          {children}
        </motion.div>
      </AnimatePresence>
    </div>
  );
}

/** Rows that drive the picture: hovering or focusing any of them shows
 * theirs. */
export function PreviewArea({
  children,
  follow,
}: {
  children: ReactNode;
  follow: PreviewFollow;
}) {
  return (
    <div className="flex flex-col gap-separate" {...follow}>
      {children}
    </div>
  );
}
