// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../base/pill-group/pill-group";

const items: { id: "auto" | "manual"; label: string }[] = [
  { id: "auto", label: "Auto" },
  { id: "manual", label: "Manual" },
];

/** Whether a highlight fits itself to the text under it or is laid by hand
 * over the box its drag spans, for a picture the fitting misreads. */
export function AnnotationFitGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (manual: boolean) => void;
  value: boolean;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label="Fit"
      display="label"
      isDisabled={isDisabled}
      items={items}
      onSelectionChange={(id) => {
        onChange(id === "manual");
      }}
      selected={value ? "manual" : "auto"}
    />
  );
}
