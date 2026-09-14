// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

const iconButtonControlStyles =
  "h-control-height w-control-height rounded-control p-control [&_svg.lucide]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu";

export const iconButtonVariants = tv({
  base: [
    "relative inline-flex origin-center transform-gpu cursor-default items-center justify-center backface-hidden will-change-transform transition select-none",
    "aria-disabled:bg-transparent aria-disabled:text-control-fg-disabled",
    "aria-disabled:data-[selected]:text-control-fg-disabled",
    iconButtonControlStyles,
    focusStyles,
    elementFocusVisible,
  ],
  compoundVariants: [
    {
      class:
        "text-control-fg-disabled data-[disabled]:data-[selected]:bg-control-fill-disabled data-[disabled]:data-[selected]:text-control-fg-disabled",
      isDisabled: true,
      isToggle: true,
    },
  ],
  defaultVariants: {
    color: "neutral",
    size: "regular",
  },
  variants: {
    color: {
      // A borderless control: the subtle hover and press tokens, which are
      // the fill ladder on macOS and Fluent's subtle fills on Windows.
      neutral:
        "bg-transparent text-content-fg data-[hovered]:bg-control-subtle-hover data-[pressed]:bg-control-subtle-pressed",
      primary:
        "bg-primary-surface text-primary-fg primary-stroke windows:data-[hovered]:bg-primary-surface-hover data-[pressed]:bg-primary-surface-pressed data-[pressed]:text-primary-fg-pressed data-[focus-visible]:ring-offset-2 data-[focus-visible]:ring-offset-content",
    },
    hasSelectedBackground: {
      true: [
        "data-[selected]:bg-fill",
        "data-[selected]:data-[hovered]:bg-fill",
        "data-[selected]:data-[pressed]:bg-fill",
      ],
    },
    isDisabled: {
      true: [
        "bg-transparent text-control-fg-disabled",
        "data-[disabled]:data-[selected]:text-control-fg-disabled",
      ],
    },
    isToggle: {
      true: "text-content-fg-secondary data-[selected]:text-content-fg",
    },
    // Capture is the recording bar's control: a 48 by 40 landscape box under
    // a 28px glyph, on the panel radius a standalone control of that height
    // takes.
    size: {
      capture: "h-10 w-12 rounded-capture [&_svg.lucide]:size-icon-xl",
      regular: "",
    },
  },
});
