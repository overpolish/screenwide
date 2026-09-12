// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

const iconButtonControlStyles =
  "h-control-height w-control-height rounded-control p-control [&_svg.lucide]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu";

export const iconButtonVariants = tv({
  base: [
    "relative inline-flex origin-center transform-gpu cursor-default items-center justify-center backface-hidden will-change-transform transition select-none",
    "aria-disabled:bg-transparent aria-disabled:text-content-fg-tertiary",
    "aria-disabled:data-[selected]:text-content-fg-tertiary",
    iconButtonControlStyles,
    focusStyles,
    elementFocusVisible,
  ],
  compoundVariants: [
    {
      class:
        "text-content-fg-tertiary data-[disabled]:data-[selected]:bg-fill-quaternary data-[disabled]:data-[selected]:text-content-fg-tertiary",
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
      neutral:
        "bg-transparent text-content-fg data-[hovered]:bg-fill-tertiary data-[pressed]:bg-fill",
      primary:
        "bg-primary-surface text-primary-fg data-[pressed]:bg-primary-surface-pressed data-[focus-visible]:ring-offset-2 data-[focus-visible]:ring-offset-content",
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
        "bg-transparent text-content-fg-tertiary",
        "data-[disabled]:data-[selected]:text-content-fg-tertiary",
      ],
    },
    isToggle: {
      true: "text-content-fg-secondary data-[selected]:text-content-fg",
    },
    // Capture is the recording bar's control: a 48 by 40 landscape box under
    // a 28px glyph, on the panel radius a standalone control of that height
    // takes.
    size: {
      capture: "h-10 w-12 rounded-panel [&_svg.lucide]:size-icon-xl",
      regular: "",
    },
  },
});
