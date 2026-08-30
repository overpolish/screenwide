// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

export const buttonVariants = tv({
  base: [
    "inline-flex items-center justify-center gap-2 font-semibold transition select-none",
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
      compact:
        "h-6 rounded-lg px-2 text-xs [&_svg]:size-icon-compact [&_svg]:shrink-0",
      default:
        "rounded-xl px-3 py-2 text-sm [&_svg]:size-icon-default [&_svg]:shrink-0",
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
