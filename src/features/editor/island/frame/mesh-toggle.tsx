// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { t } from "../../../../i18n/i18n";
import { useGlyphMotion } from "../../../annotations/options/glyph-motion";
import { OptionToggle } from "../../../annotations/options/option-toggle";

/** Where each of the three circles sits, and how big it is: stacked into one
 * disc for a single colour, spread into overlapping ones for a mesh. */
const SOLID = [
  { cx: 12, cy: 12, r: 8 },
  { cx: 12, cy: 12, r: 8 },
  { cx: 12, cy: 12, r: 8 },
];
const MESH = [
  { cx: 9, cy: 9.5, r: 5.5 },
  { cx: 15, cy: 9.5, r: 5.5 },
  { cx: 12, cy: 15, r: 5.5 },
];

/** One circle for one colour; turned on, it parts into three overlapping
 * ones, colours blending where they meet, as a mesh paints them. */
function MeshGlyph({ isMesh }: { isMesh: boolean }) {
  const motionOf = useGlyphMotion();
  return (
    <svg
      aria-hidden
      className="size-icon"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      {(isMesh ? MESH : SOLID).map((circle, index) => (
        <motion.circle
          animate={circle}
          initial={false}
          // The circles are named by their place, which never changes.
          // eslint-disable-next-line @eslint-react/no-array-index-key
          key={index}
          transition={motionOf.move}
        />
      ))}
    </svg>
  );
}

/** Whether the background is a mesh of colours rather than one. */
export function MeshToggle({
  isMesh,
  onChange,
}: {
  isMesh: boolean;
  onChange: (isMesh: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <MeshGlyph isMesh={isOn} />}
      isOn={isMesh}
      label={t("background-kind-mesh")}
      onChange={onChange}
    />
  );
}
