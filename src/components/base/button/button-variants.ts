// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

export const buttonVariants = tv({
  base: [
    "relative inline-flex h-control-height cursor-default items-center justify-center gap-inline rounded-control px-inset text-label transition select-none",
    "[&_svg.lucide]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
    focusStyles,
    elementFocusVisible,
  ],
  compoundVariants: [
    {
      class:
        "control-raised text-fg data-[hovered]:control-raised-hover data-[pressed]:control-raised-pressed",
      color: "neutral",
      variant: "solid",
    },
    {
      // The primary action, filled, with its bevel.
      class:
        "bg-primary text-on-primary primary-raised data-[hovered]:bg-primary-hover data-[pressed]:bg-primary-pressed data-[pressed]:primary-raised-pressed",
      color: "primary",
      variant: "solid",
    },
  ],
  defaultVariants: {
    color: "neutral",
    isDisabled: false,
    variant: "solid",
  },
  variants: {
    // Only a solid button has a colour: the primary action is always filled.
    color: {
      neutral: "",
      primary: "",
    },
    // Disabled keeps its shape and material, only faded, so it reads as the
    // same button rather than a flatter, larger one. React Aria sends no
    // hover or press while disabled.
    isDisabled: {
      false: "",
      true: "opacity-40",
    },
    variant: {
      // No material of its own, for an action that should not compete:
      // a command in a window's toolbar, or one inside a notice.
      ghost:
        "bg-transparent text-fg data-[hovered]:bg-fill data-[pressed]:bg-fill-strong",
      solid: "",
    },
  },
});
