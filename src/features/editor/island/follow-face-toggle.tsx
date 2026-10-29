// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";

import { t } from "../../../i18n/i18n";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { focusIn } from "../../../primitives/island/island-motion";
import { useGlyphMotion } from "../../annotations/options/glyph-motion";
import { OptionToggle } from "../../annotations/options/option-toggle";

/** Lucide's scan-face brackets round a face. Off, the face has wandered to
 * one side of them; following, it springs back to their middle, as the
 * camera keeps the speaker's face there. */
function FollowFaceGlyph({ isFollowing }: { isFollowing: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2" />
      <motion.g
        animate={isFollowing ? { x: 0, y: 0 } : { x: -3, y: 2 }}
        initial={false}
        transition={motionOf.move}
      >
        <path d="M8 14s1.5 2 4 2 4-2 4-2M9 9h.01M15 9h.01" />
      </motion.g>
    </svg>
  );
}

/** Whether the camera keeps the speaker's face in the middle of its crop:
 * for the whole recording on the Camera island, for one scene on the Scene
 * island. While the face is first looked for, a spinner beside the toggle
 * comes into focus as the island's parts do. */
export function FollowFaceToggle({
  isFindingFace,
  isFollowing,
  onChange,
}: {
  isFindingFace: boolean;
  isFollowing: boolean;
  onChange: (isFollowing: boolean) => void;
}) {
  return (
    <span className="flex items-center gap-inline">
      <AnimatePresence initial={false}>
        {isFollowing && isFindingFace ? (
          <motion.span
            animate={focusIn.animate}
            className="flex"
            exit={focusIn.exit}
            initial={focusIn.exit}
            key="finding"
            transition={focusIn.transition}
          >
            <CircularProgress
              aria-label={t("editor-island-finding-face")}
              isIndeterminate
              size="small"
            />
          </motion.span>
        ) : null}
      </AnimatePresence>
      <OptionToggle
        glyph={(isOn) => <FollowFaceGlyph isFollowing={isOn} />}
        isOn={isFollowing}
        label={t("editor-island-follow-face")}
        onChange={onChange}
      />
    </span>
  );
}
