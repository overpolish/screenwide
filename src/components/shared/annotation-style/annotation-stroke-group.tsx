// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../base/pill-group/pill-group";

const items: { id: "clean" | "hand"; label: string }[] = [
  { id: "clean", label: "Clean" },
  { id: "hand", label: "Hand-drawn" },
];

/** Whether a highlight is drawn as a clean band or as a marker stroke by
 * hand, labelled for a toolbar that has no room for a row heading. */
export function AnnotationStrokeGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (handDrawn: boolean) => void;
  value: boolean;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label="Stroke"
      display="label"
      isDisabled={isDisabled}
      items={items}
      onSelectionChange={(id) => {
        onChange(id === "hand");
      }}
      selected={value ? "hand" : "clean"}
    />
  );
}
