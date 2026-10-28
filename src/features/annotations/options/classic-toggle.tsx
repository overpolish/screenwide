// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Grid3x3, ShieldCheck } from "lucide-react";
import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

/** Secure pixelation, its blocks laid out at random so nothing of the
 * layout is left, as a shield; classic, each block the average of what it
 * covers, as the plain grid it looks like. Changing shrinks the one away as
 * the other grows in its place. */
function PixelationGlyph({ isClassic }: { isClassic: boolean }) {
  const motionOf = useGlyphMotion();
  const shown = (isShown: boolean) => ({
    animate: isShown
      ? { opacity: 1, rotate: 0, scale: 1 }
      : { opacity: 0, rotate: -20, scale: 0.6 },
    className: "absolute inset-0 flex items-center justify-center",
    initial: false as const,
    transition: motionOf.move,
  });
  return (
    <span aria-hidden className="relative flex size-icon">
      <motion.span {...shown(!isClassic)}>
        <ShieldCheck className="size-icon" />
      </motion.span>
      <motion.span {...shown(isClassic)}>
        <Grid3x3 className="size-icon" />
      </motion.span>
    </span>
  );
}

/** Whether a pixelated redaction uses classic pixelation rather than
 * secure: only a look, which the island says when it is on. */
export function ClassicToggle({
  isClassic,
  onChange,
}: {
  isClassic: boolean;
  onChange: (isClassic: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <PixelationGlyph isClassic={isOn} />}
      isOn={isClassic}
      label={t("annotation-pixelation-classic-on")}
      onChange={onChange}
    />
  );
}
