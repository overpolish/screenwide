// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Droplet, Eraser, Grid2x2, PaintBucket } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../base/pill-group/pill-group";

import { AnnotationRedaction } from "./types";

/** How a redaction covers what is under it. Pressing Pixelate or Blur while
 * it is already chosen lays the blocks out, or nudges the cells, again. Both
 * pixelation styles light Pixelate; the classic one has no layout to shuffle.
 * Without `onShuffle` - no redaction drawn yet - the press only chooses.
 */
export function AnnotationRedactionGroup({
  isDisabled,
  onChange,
  onShuffle,
  value,
}: {
  onChange: (redaction: AnnotationRedaction) => void;
  value: AnnotationRedaction;
  isDisabled?: boolean;
  onShuffle?: () => void;
}) {
  return (
    <PillGroup
      aria-label={t("annotation-redaction")}
      isDisabled={isDisabled}
      items={[
        {
          icon: <Eraser aria-hidden="true" />,
          id: "erase",
          label: t("annotation-redaction-erase"),
        },
        {
          icon: <PaintBucket aria-hidden="true" />,
          id: "color",
          label: t("annotation-redaction-color"),
        },
        {
          // The press lands after the selection has moved, but `value` is the
          // one this render was given: only a press on the mode already
          // chosen shuffles.
          ariaLabel:
            value === "pixelate" && onShuffle
              ? t("annotation-redaction-shuffle-pixelation")
              : t("annotation-redaction-pixelate"),
          icon: <Grid2x2 aria-hidden="true" />,
          id: "pixelate",
          label: t("annotation-redaction-pixelate"),
          onPress: value === "pixelate" ? onShuffle : undefined,
        },
        {
          ariaLabel:
            value === "blur" && onShuffle
              ? t("annotation-redaction-shuffle-blur")
              : t("annotation-redaction-blur"),
          icon: <Droplet aria-hidden="true" />,
          id: "blur",
          label: t("annotation-redaction-blur"),
          onPress: value === "blur" ? onShuffle : undefined,
        },
      ]}
      onSelectionChange={(id) => {
        onChange(id as AnnotationRedaction);
      }}
      selected={value === "pixelateClassic" ? "pixelate" : value}
    />
  );
}
