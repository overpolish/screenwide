// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

/** Swung about the pendulum's pivot: Motion places an SVG element's origin
 * within the element's own box, and the rod starts at the pivot, so the
 * pivot is the middle of the box's top. */
const PIVOT = { originX: 0.5, originY: 0 } as const;

/** A pendulum: its rod from the pivot and the weight at its end. */
function Pendulum() {
  return (
    <>
      <path d="M12 3.5V14" />
      <circle cx={12} cy={17} r={3.5} />
    </>
  );
}

/** Still, a pendulum hanging straight. Turned on, it swings out to one side
 * and a faint copy shows the other end of its swing, the image's slow turn
 * and drift; turned off, it comes back to hang straight. A pendulum rather
 * than the picture itself, so it is not taken for the shadow's picture
 * beside it. */
function SwayGlyph({ sways }: { sways: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <motion.g
        animate={
          sways ? { opacity: 0.35, rotate: -32 } : { opacity: 0, rotate: 0 }
        }
        initial={false}
        style={PIVOT}
        transition={motionOf.move}
      >
        <Pendulum />
      </motion.g>
      <motion.g
        animate={{ rotate: sways ? 32 : 0 }}
        initial={false}
        style={PIVOT}
        transition={motionOf.move}
      >
        <Pendulum />
      </motion.g>
      <circle cx={12} cy={3.5} fill="currentColor" r={1.25} stroke="none" />
    </svg>
  );
}

/** Whether a recording gives a placed image a slight, slow turn and drift,
 * so a picture that points at nothing still feels alive. Each time it is
 * turned on, the image gets a sway of its own. */
export function SwayToggle({
  onChange,
  sways,
}: {
  onChange: (sways: boolean) => void;
  sways: boolean;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <SwayGlyph sways={isOn} />}
      isOn={sways}
      label={t("annotation-sway-on")}
      onChange={onChange}
    />
  );
}
