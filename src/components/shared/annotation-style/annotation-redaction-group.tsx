// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Droplet, Eraser, Grid2x2, PaintBucket } from "lucide-react";

import { PillGroup } from "../../base/pill-group/pill-group";

import { AnnotationRedaction } from "./types";

/** How a redaction covers what is under it. Pressing Pixelate or Blur while
 * it is already chosen lays the blocks out, or nudges the cells, again. Both
 * pixelation styles light Pixelate; the classic one has no layout to shuffle.
 */
export function AnnotationRedactionGroup({
  isDisabled,
  onChange,
  onShuffle,
  value,
}: {
  onChange: (redaction: AnnotationRedaction) => void;
  onShuffle: () => void;
  value: AnnotationRedaction;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label="Redaction"
      isDisabled={isDisabled}
      items={[
        { icon: <Eraser aria-hidden="true" />, id: "erase", label: "Erase" },
        {
          icon: <PaintBucket aria-hidden="true" />,
          id: "color",
          label: "Colour",
        },
        {
          // The press lands after the selection has moved, but `value` is the
          // one this render was given: only a press on the mode already
          // chosen shuffles.
          ariaLabel: value === "pixelate" ? "Shuffle pixelation" : "Pixelate",
          icon: <Grid2x2 aria-hidden="true" />,
          id: "pixelate",
          label: "Pixelate",
          onPress: value === "pixelate" ? onShuffle : undefined,
        },
        {
          ariaLabel: value === "blur" ? "Shuffle blur" : "Blur",
          icon: <Droplet aria-hidden="true" />,
          id: "blur",
          label: "Blur",
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
