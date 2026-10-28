// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { annotationSizeLabel } from "../annotation-text";

/** A block size as a square in the text's colour, larger for each step, as
 * the weights are dots: secure and classic pixelation offer different sizes,
 * so the square follows the step rather than the points. */
function BlockGlyph({ step }: { step: number }) {
  const side = 5 + step * 2;
  return (
    <span
      className="rounded-[1.5px] bg-current"
      style={{ height: side, width: side }}
    />
  );
}

/** How big a pixelated redaction's blocks are, from `sizes`, its style's own
 * steps: the larger, the less of what is under it shows. */
export function BlockSizeChoice({
  onChange,
  sizes,
  value,
}: {
  onChange: (points: number) => void;
  /** The block sizes offered, in points, smallest first. */
  sizes: number[];
  /** The block size in points. */
  value: number;
}) {
  return (
    <PillGroup
      aria-label={annotationSizeLabel("blockSize")}
      items={sizes.map((points, step) => ({
        ariaLabel: t("annotate-island-points", { points }),
        icon: <BlockGlyph step={step} />,
        id: String(points),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}
