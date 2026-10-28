// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

// Lucide's ArrowRightToLine, played through once to its end, and Repeat,
// going round for as long as the clip lasts.
const ONCE = ["M17 12H3", "m11 18 6-6-6-6", "M21 5v14"];
const LOOP = [
  "m17 2 4 4-4 4",
  "M3 11v-1a4 4 0 0 1 4-4h14",
  "m7 22-4-4 4-4",
  "M21 13v1a4 4 0 0 1-4 4H3",
];

/** Playing once, an arrow running to a line; looping, the arrows going
 * round. Changing undraws the one and draws the other in, a part at a time.
 * It points the way time runs, so it mirrors with the language. */
function LoopGlyph({ loops }: { loops: boolean }) {
  const motionOf = useGlyphMotion();
  const lines = (paths: string[], shown: boolean) =>
    paths.map((d, index) => (
      <motion.path
        animate={
          shown ? { opacity: 1, pathLength: 1 } : { opacity: 0, pathLength: 0 }
        }
        d={d}
        initial={false}
        key={d}
        transition={{
          ...motionOf.draw,
          delay:
            shown && motionOf.draw.duration !== 0 ? 0.08 + index * 0.04 : 0,
        }}
      />
    ));
  return (
    <svg
      aria-hidden
      className="size-icon rtl:-scale-x-100"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      {lines(ONCE, !loops)}
      {lines(LOOP, loops)}
    </svg>
  );
}

/** Whether a recording loops a moving image for as long as its clip lasts,
 * or plays it through once, the clip then lasting exactly one run. */
export function LoopToggle({
  loops,
  onChange,
}: {
  loops: boolean;
  onChange: (loops: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <LoopGlyph loops={isOn} />}
      isOn={loops}
      label={t("annotation-playback-loop")}
      onChange={onChange}
    />
  );
}
