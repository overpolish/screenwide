// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../base/pill-group/pill-group";

import { SOFT_SPOTLIGHT_EDGE } from "./widths";

const items: { id: "hard" | "soft"; label: string }[] = [
  { id: "hard", label: "Hard" },
  { id: "soft", label: "Soft" },
];

/** Whether a spotlight's edge is hard or fades in from its box, labelled for
 * a toolbar that has no room for a row heading. `value` is the softness it
 * draws with, any fade at all reading as soft. */
export function AnnotationEdgeGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (softness: number) => void;
  value: number;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label="Edge"
      display="label"
      isDisabled={isDisabled}
      items={items}
      onSelectionChange={(id) => {
        onChange(id === "soft" ? SOFT_SPOTLIGHT_EDGE : 0);
      }}
      selected={value > 0 ? "soft" : "hard"}
    />
  );
}
