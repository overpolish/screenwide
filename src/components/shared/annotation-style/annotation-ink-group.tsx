// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../base/pill-group/pill-group";

const items: { id: "recolor" | "tint"; label: string }[] = [
  { id: "recolor", label: "Recolor" },
  { id: "tint", label: "Tint" },
];

/** Whether a highlight recolours the page it reads, turning the page into its
 * colour and the text into ink that reads on it, or tints what it covers the
 * way a felt marker does, for content the recolouring misreads. */
export function AnnotationInkGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (tint: boolean) => void;
  value: boolean;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label="Ink"
      display="label"
      isDisabled={isDisabled}
      items={items}
      onSelectionChange={(id) => {
        onChange(id === "tint");
      }}
      selected={value ? "tint" : "recolor"}
    />
  );
}
