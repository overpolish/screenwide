// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

export const compactButtonControlStyles =
  "h-6 rounded-lg px-control-inset text-xs [&_svg]:size-icon-compact [&_svg]:shrink-0";

export const defaultButtonControlStyles =
  "rounded-xl px-section py-control-inset text-sm [&_svg]:size-icon-default [&_svg]:shrink-0";

export const buttonVariants = tv({
  base: [
    "gap-control-inset inline-flex items-center justify-center font-semibold transition select-none",
    focusStyles,
    elementFocusVisible,
  ],
  defaultVariants: {
    color: "neutral",
    size: "default",
    variant: "solid",
  },
  variants: {
    color: {
      neutral: [
        "text-content-fg bg-neutral",
        "aria-pressed:bg-neutral-hover",
        "data-[hovered]:bg-neutral-hover",
        "data-[pressed]:bg-neutral-pressed",
      ],
      primary: [
        "text-primary-fg bg-primary-surface",
        "data-[hovered]:bg-primary-surface-hover",
        "data-[pressed]:bg-primary-surface-pressed",
      ],
    },
    isDisabled: {
      true: "cursor-not-allowed! bg-neutral-subtle text-neutral-disabled-fg",
    },
    size: {
      compact: compactButtonControlStyles,
      default: defaultButtonControlStyles,
    },
    variant: {
      ghost: [
        "bg-transparent cursor-pointer",
        "data-[hovered]:bg-neutral",
        "data-[pressed]:bg-neutral-hover",
      ],
      solid: "border-none",
    },
  },
});
