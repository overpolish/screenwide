// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ZoomIn } from "lucide-react";

import { NumberField } from "../../../components/base/input-fields/number-field";
import { t } from "../../../i18n/i18n";

/**
 * The zoom field, at the left of a workspace's closing row. The native surface
 * owns the transform: a pinch on it publishes the zoom back here, and typing or
 * scrubbing a zoom here sets it.
 */
export function PreviewZoomField({
  onChange,
  zoomPercent,
}: {
  onChange: (zoomPercent: number) => void;
  zoomPercent: number;
}) {
  return (
    <NumberField
      aria-label={t("editor-zoom")}
      className="w-24"
      leftSection={<ZoomIn />}
      maxValue={800}
      minValue={10}
      onChange={onChange}
      rightSection="%"
      showSteppers={false}
      step={1}
      value={zoomPercent}
    />
  );
}
