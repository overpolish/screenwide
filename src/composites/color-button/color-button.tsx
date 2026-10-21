// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref } from "react";
import {
  Button as AriaButton,
  type ButtonProps as AriaButtonProps,
} from "react-aria-components";

import { cn } from "../../lib/styling";
import { Button } from "../../primitives/button/button";
import { swatchStyles } from "../../primitives/swatch/swatch-styles";

// Without a label the button is the colour itself, on Swatch's square; with
// one it is a neutral button with the colour beside its name.

type ColorButtonProps = {
  onPress: AriaButtonProps["onPress"];
  /** `#rrggbb` or `#rrggbbaa`. */
  value: string;
  "aria-expanded"?: boolean;
  isDisabled?: boolean;
  ref?: Ref<HTMLButtonElement>;
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
  value,
}: ColorButtonProps) {
  if (label === undefined)
    return (
      <AriaButton
        aria-expanded={isExpanded}
        aria-label={ariaLabel}
        className={swatchStyles(true)}
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
          "size-icon shrink-0 rounded-sm",
          "inset-ring-1 inset-ring-divider",
        )}
        style={{ backgroundColor: value }}
      />
      {label}
    </Button>
  );
}
