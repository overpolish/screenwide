// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref } from "react";
import {
  Button as AriaButton,
  type ButtonProps as AriaButtonProps,
} from "react-aria-components";

import { cn } from "../../lib/styling";
import { Button } from "../../primitives/button/button";
import {
  swatchStyles,
  type SwatchSize,
} from "../../primitives/swatch/swatch-styles";

// Without a label the button is the colour itself, on Swatch's circle; with
// one it is a neutral button with the colour beside its name.

type ColorButtonProps = {
  onPress: AriaButtonProps["onPress"];
  /** `#rrggbb` or `#rrggbbaa`. */
  value: string;
  "aria-expanded"?: boolean;
  isDisabled?: boolean;
  ref?: Ref<HTMLButtonElement>;
  /** The swatch-only form's size, as a swatch's. */
  size?: SwatchSize;
} & (
  | {
      /** Leave out, giving `aria-label`, for the swatch-only form. */
      label: string;
      "aria-label"?: never;
    }
  | { "aria-label": string; label?: never }
);

export function ColorButton({
  "aria-expanded": isExpanded,
  "aria-label": ariaLabel,
  isDisabled,
  label,
  onPress,
  ref,
  size,
  value,
}: ColorButtonProps) {
  if (label === undefined)
    return (
      <AriaButton
        aria-expanded={isExpanded}
        aria-label={ariaLabel}
        className={swatchStyles(true, size)}
        isDisabled={isDisabled}
        onPress={onPress}
        ref={ref}
        style={{ background: value }}
      />
    );

  return (
    <Button
      aria-expanded={isExpanded}
      isDisabled={isDisabled}
      onPress={onPress}
      ref={ref}
    >
      <span
        aria-hidden="true"
        className={cn(
          "size-icon shrink-0 rounded-full",
          "inset-ring-1 inset-ring-divider",
        )}
        style={{ backgroundColor: value }}
      />
      {label}
    </Button>
  );
}
