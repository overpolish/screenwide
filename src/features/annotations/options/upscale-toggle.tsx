// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

/** A disc in stair-stepped pixels, a stretched picture's blocky edge. */
const PIXEL_DISC = "M9 5H15V7H17V9H19V15H17V17H15V19H9V17H7V15H5V9H7V7H9Z";

/** A disc with a smooth edge when on, the picture drawn larger by the
 * model; the same disc in stair-stepped pixels when off, the picture
 * stretched. */
function UpscaleGlyph({ upscales }: { upscales: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <motion.path
        animate={{ opacity: upscales ? 0 : 1 }}
        d={PIXEL_DISC}
        initial={false}
        transition={motionOf.fade}
      />
      <motion.circle
        animate={{ opacity: upscales ? 1 : 0 }}
        cx={12}
        cy={12}
        initial={false}
        r={7}
        transition={motionOf.fade}
      />
    </svg>
  );
}

/** Whether an image drawn larger than its picture is drawn larger by the
 * upscaling model, which keeps flat drawn art sharp, or stretched. */
export function UpscaleToggle({
  onChange,
  upscales,
}: {
  onChange: (upscales: boolean) => void;
  upscales: boolean;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <UpscaleGlyph upscales={isOn} />}
      isOn={upscales}
      label={t("annotation-upscale-on")}
      onChange={onChange}
    />
  );
}
