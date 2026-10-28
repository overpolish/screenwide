// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../i18n/i18n";

import { useGlyphMotion } from "./glyph-motion";
import { OptionToggle } from "./option-toggle";

// A highlight's band, clean and by hand, as two paths of the same commands so
// the one can be moved into the other. By hand it is the renderer's marker
// stroke (`highlight_stroke` in `annotation_highlight.wgsl`): its top and
// bottom waving gently and out of step, and both ends cut on the one slant
// of the nib, deeper than the renderer's so it reads at this size.
const BAND = {
  clean:
    "M4.25 9.25C9 9.25 11.5 9.25 14.75 9.25S19.75 9.25 19.75 9.25L19.75 14.75C15.25 14.75 12.5 14.75 9.25 14.75S4.25 14.75 4.25 14.75Z",
  hand: "M5.75 9.25C9 8.25 11.5 10.25 14.75 9.25S18.75 8.5 20.5 9.25L18.25 14.75C15.25 15.75 12.5 13.75 9.25 14.75S5.25 15.5 3.5 14.75Z",
};

// A shape by hand, the renderer's stray (`outline/geometry.rs`): the box a
// touch off square, its sides bowing a little, and the pen running on past
// where it began, finer as it lifts. Deeper than the renderer's, so it reads.
const STRAY = [
  "M7.5 5.6C11 5.1 15 4.9 18.2 4.9C19.1 4.9 19.4 5.5 19.4 6.4C19.6 10 19.6 14.5 19.2 17.9C19.1 18.8 18.5 19.2 17.6 19.2C13.5 19.5 9.5 19.6 6.3 19.3C5.3 19.2 4.8 18.6 4.8 17.6C4.5 13.5 4.6 9.5 4.9 6.3C5 4.8 6 4.2 7.5 4.2",
  "M7.5 4.2C9.2 4.2 10.8 4.6 12.4 5.3",
];

/** A highlight's band, moving between straight and waved. */
function BandGlyph({ isHandDrawn }: { isHandDrawn: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="currentColor"
      stroke="currentColor"
      strokeLinejoin="round"
      strokeWidth={2.5}
      viewBox="0 0 24 24"
    >
      <motion.path
        animate={{ d: isHandDrawn ? BAND.hand : BAND.clean }}
        initial={false}
        transition={motionOf.move}
      />
    </svg>
  );
}

/** A shape's outline: turned on, the clean box undraws as the pen draws
 * the stray round in its place; turned off, the other way. */
function OutlineGlyph({ isHandDrawn }: { isHandDrawn: boolean }) {
  const motionOf = useGlyphMotion();
  const drawn = (shown: boolean, delay: number) => ({
    animate: shown
      ? { opacity: 1, pathLength: 1 }
      : { opacity: 0, pathLength: 0 },
    initial: false as const,
    transition: {
      ...motionOf.draw,
      delay: shown && motionOf.draw.duration !== 0 ? delay : 0,
    },
  });
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
      <motion.rect
        {...drawn(!isHandDrawn, 0.06)}
        height={14}
        rx={2}
        width={14}
        x={5}
        y={5}
      />
      <g transform="rotate(-3 12 12)">
        <motion.path {...drawn(isHandDrawn, 0.06)} d={STRAY[0]} />
        <motion.path
          {...drawn(isHandDrawn, 0.22)}
          d={STRAY[1]}
          strokeWidth={1.25}
        />
      </g>
    </svg>
  );
}

/** Whether a highlight or a shape is drawn by hand rather than clean, its
 * glyph drawn as its own kind would look. Each time it is turned on, the
 * stroke gets a wobble of its own. */
export function HandDrawnToggle({
  isHandDrawn,
  kind,
  onChange,
}: {
  isHandDrawn: boolean;
  kind: "highlight" | "shape";
  onChange: (isHandDrawn: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) =>
        kind === "highlight" ? (
          <BandGlyph isHandDrawn={isOn} />
        ) : (
          <OutlineGlyph isHandDrawn={isOn} />
        )
      }
      isOn={isHandDrawn}
      label={t("annotation-stroke-hand-drawn")}
      onChange={onChange}
    />
  );
}
