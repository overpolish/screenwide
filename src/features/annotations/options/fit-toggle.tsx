// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

/** How a highlight covers what it is drawn over, in the stroke glyph's
 * solid bands: fitted, a band to each line of text cut to that line's own
 * length; covering the box, one box over the area dragged. Turned on, the
 * first band grows down over the second into the box; turned off, the box
 * parts back into its lines. */
function FitGlyph({ coversBox }: { coversBox: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      <motion.rect
        animate={{ opacity: coversBox ? 0 : 1 }}
        height={5}
        initial={false}
        rx={1}
        transition={motionOf.fade}
        width={16}
        x={3.5}
        y={13}
      />
      <motion.rect
        animate={
          coversBox ? { height: 12, width: 17 } : { height: 5, width: 11 }
        }
        initial={false}
        rx={1}
        transition={motionOf.move}
        x={3.5}
        y={6}
      />
    </svg>
  );
}

/** Whether a highlight covers the box dragged rather than fitting the text
 * it is drawn over. Live, the drag decides, Option held laying the box; a
 * drawn highlight can change its mind. */
export function FitToggle({
  coversBox,
  onChange,
}: {
  coversBox: boolean;
  onChange: (coversBox: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <FitGlyph coversBox={isOn} />}
      isOn={coversBox}
      label={t("annotation-fit-box")}
      onChange={onChange}
    />
  );
}
