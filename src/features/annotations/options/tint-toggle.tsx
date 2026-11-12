// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { useId } from "react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

// Two lines of text under the band, the second shorter, as a line of a page
// ends short.
const LINES = [
  { width: 12, y: 9.25 },
  { width: 8, y: 13 },
];

/** A highlight's band over two lines of text. Recoloured, the band is solid
 * and the text is cut out of it, read in the band's own ink; tinted, the
 * band fades to a wash and the text stands dark through it. */
function TintGlyph({ isTinted }: { isTinted: boolean }) {
  const motionOf = useGlyphMotion();
  const maskId = useId();
  const lines = LINES.map(({ width, y }) => (
    <rect height={1.75} key={y} rx={0.875} width={width} x={6} y={y} />
  ));
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      <defs>
        <mask id={maskId}>
          <rect fill="white" height={24} width={24} />
          <motion.g
            animate={{ opacity: isTinted ? 0 : 1 }}
            fill="black"
            initial={false}
            transition={motionOf.fade}
          >
            {lines}
          </motion.g>
        </mask>
      </defs>
      <motion.rect
        animate={{ opacity: isTinted ? 0.35 : 1 }}
        height={10}
        initial={false}
        mask={`url(#${maskId})`}
        rx={1.5}
        transition={motionOf.fade}
        width={17}
        x={3.5}
        y={6.5}
      />
      <motion.g
        animate={{ opacity: isTinted ? 1 : 0 }}
        initial={false}
        transition={motionOf.fade}
      >
        {lines}
      </motion.g>
    </svg>
  );
}

/** Whether a highlight tints what it covers, as a marker would, rather than
 * recolouring the page under it: a fallback for pages the recolour
 * misreads. */
export function TintToggle({
  isLabelHidden,
  isTinted,
  onChange,
}: {
  isTinted: boolean;
  onChange: (isTinted: boolean) => void;
  isLabelHidden?: boolean;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <TintGlyph isTinted={isOn} />}
      isLabelHidden={isLabelHidden}
      isOn={isTinted}
      label={t("annotation-highlight-tint")}
      onChange={onChange}
    />
  );
}
