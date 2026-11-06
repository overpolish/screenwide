// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { AnnotationColors } from "../../annotations/options/annotation-colors";

import type { SectionProps } from "./section-props";

/** The colours the next annotation is drawn in, every tool's alike. */
export function AnnotateColors({
  onChange,
  savedColors,
  settings,
}: SectionProps & { savedColors?: string[] }) {
  return (
    <AnnotationColors
      onChange={(defaultColor) => {
        onChange({ defaultColor });
      }}
      savedColors={savedColors}
      value={settings.defaultColor}
    />
  );
}
