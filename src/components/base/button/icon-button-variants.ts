// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

export const compactIconControlStyles =
  "h-6 w-6 rounded-lg p-control [&_svg]:size-icon-compact";

export const defaultIconControlStyles =
  "h-9 w-9 rounded-xl p-control-inset [&_svg]:size-icon-default";

export const iconButtonVariants = tv({
  base: [
    "relative inline-flex origin-center transform-gpu cursor-pointer items-center justify-center backface-hidden will-change-transform transition select-none",
    "aria-disabled:bg-neutral-subtle aria-disabled:text-neutral-disabled-fg",
    "aria-disabled:data-[selected]:text-neutral-disabled-fg",
    focusStyles,
    elementFocusVisible,
  ],
  compoundVariants: [
    {
      class:
        "text-neutral-disabled-fg data-[disabled]:data-[selected]:bg-neutral-subtle data-[disabled]:data-[selected]:text-neutral-disabled-fg",
      isDisabled: true,
      isToggle: true,
    },
    {
      class: "p-tight! [&_svg]:size-icon-prominent! [&_svg]:shrink-0",
      iconSize: "prominent",
      size: "default",
    },
  ],
  defaultVariants: {
    color: "neutral",
    size: "default",
  },
  variants: {
    color: {
      neutral: [
        "bg-transparent text-content-fg",
        "data-[hovered]:bg-neutral",
        "data-[pressed]:bg-neutral-hover",
      ],
      primary: [
        "bg-primary-surface text-primary-fg",
        "data-[hovered]:bg-primary-surface-hover",
        "data-[pressed]:bg-primary-surface-pressed",
      ],
    },
    iconSize: {
      prominent: "",
    },
    isDisabled: {
      true: [
        "cursor-not-allowed! bg-neutral-subtle text-neutral-disabled-fg",
        "data-[disabled]:data-[selected]:text-neutral-disabled-fg",
      ],
    },
    isGrouped: {
      true: ["data-[hovered]:bg-transparent", "data-[pressed]:bg-transparent"],
    },
    isToggle: {
      true: "text-muted data-[selected]:text-content-fg",
    },
    size: {
      compact: compactIconControlStyles,
      default: defaultIconControlStyles,
    },
  },
});
