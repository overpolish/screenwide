// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ComponentProps, type Ref } from "react";

import { Button } from "../button/button";

/**
 * A colour beside its name on a button, for choosing that colour from
 * somewhere else, as a pop-up of swatches. The swatch is the ruler loupe's:
 * an icon-sized square with the swatch corner, so a colour reads alike
 * wherever the app shows one.
 */
export function ColorChip({
  "aria-expanded": isExpanded,
  isDisabled,
  label,
  onPress,
  ref,
  value,
}: {
  label: string;
  onPress: ComponentProps<typeof Button>["onPress"];
  /** `#rrggbb` or `#rrggbbaa`. */
  value: string;
  /** Whether what the chip opens is showing. */
  "aria-expanded"?: boolean;
  isDisabled?: boolean;
  ref?: Ref<HTMLButtonElement>;
}) {
  return (
    <Button
      aria-expanded={isExpanded}
      className="gap-control-inset px-control-inset"
      isDisabled={isDisabled}
      onPress={onPress}
      ref={ref}
    >
      <span
        aria-hidden="true"
        className="size-icon shrink-0 rounded-swatch"
        style={{ backgroundColor: value }}
      />
      {label}
    </Button>
  );
}
