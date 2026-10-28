// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

/** The picture's frame, in the glyph's own 24 units. */
const FRAME = { height: 11, rx: 2.5, width: 13, x: 5.5, y: 6.5 };

/** Where the frame sits in the glyph's box, for the halo behind it. */
const share = (units: number) => `${String((units / 24) * 100)}%`;
const HALO_BOX = {
  height: share(FRAME.height),
  insetInlineStart: share(FRAME.x),
  top: share(FRAME.y),
  width: share(FRAME.width),
};

/** A picture's frame with a hill in it, and with a shadow a soft halo of it
 * behind, centred, spreading past its edges all round. Turned on, the halo
 * grows out from behind the frame as it fades in; turned off, it draws back
 * in and is gone. The halo is a box the frame's size casting an outer
 * shadow in the text's colour: an outer shadow never paints inside its own
 * box, so the frame stays hollow, and it is not cut at an edge the way an
 * SVG filter's region cuts a blur. A picture rather than a box or a ball,
 * so it is not taken for the corner choice beside it, and it is the image's
 * own shadow. */
function ShadowGlyph({ hasShadow }: { hasShadow: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <span aria-hidden className="relative flex size-icon">
      <motion.span
        animate={
          hasShadow
            ? { opacity: 0.9, scale: 1.12 }
            : { opacity: 0, scale: 0.85 }
        }
        className="absolute rounded-[2px] shadow-[0_0_6px_2px_currentColor]"
        initial={false}
        style={HALO_BOX}
        transition={motionOf.move}
      />
      <svg
        className="relative size-icon"
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        viewBox="0 0 24 24"
      >
        <rect {...FRAME} strokeWidth={2} />
        <path
          d="M8 15l2.75-2.75 2.25 2.25 1.75-1.75 1.75 1.75"
          strokeWidth={1.75}
        />
      </svg>
    </span>
  );
}

/** Whether an image casts a shadow onto the picture under it. */
export function ShadowToggle({
  hasShadow,
  onChange,
}: {
  hasShadow: boolean;
  onChange: (hasShadow: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <ShadowGlyph hasShadow={isOn} />}
      isOn={hasShadow}
      label={t("annotation-shadow-drop")}
      onChange={onChange}
    />
  );
}
