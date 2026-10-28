// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { AnnotationColors } from "../../annotations/options/annotation-colors";

import type { SectionProps } from "./section-props";

/** The colours the next annotation is drawn in, every tool's alike. */
export function AnnotateColors({
  onChange,
  ref,
  savedColors,
  settings,
}: SectionProps & { savedColors?: string[] }) {
  return (
    <IslandCrossfade ref={ref}>
      <AnnotationColors
        onChange={(defaultColor) => {
          onChange({ defaultColor });
        }}
        savedColors={savedColors}
        value={settings.defaultColor}
      />
    </IslandCrossfade>
  );
}
