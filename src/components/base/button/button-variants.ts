// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

export const buttonControlStyles =
  "h-control-height rounded-control px-section text-body [&_svg.lucide]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu";

export const buttonVariants = tv({
  base: [
    "gap-control inline-flex cursor-default items-center justify-center select-none relative transition",
    focusStyles,
    elementFocusVisible,
    buttonControlStyles,
  ],
  compoundVariants: [
    {
      // Bezeled controls do not react to hover; a press stacks the secondary
      // fill behind the label rather than replacing the bezel. The overlay is
      // always present and only gains its colour while pressed, so the change
      // transitions like any other background.
      class: [
        "isolate after:pointer-events-none after:absolute after:inset-0 after:-z-10",
        "after:rounded-[inherit] after:transition-colors after:content-['']",
        "aria-pressed:after:bg-fill-secondary data-[pressed]:after:bg-fill-secondary",
      ],
      color: "neutral",
      variant: "solid",
    },
    {
      // A ghost button has no bezel to dim.
      class: "bg-transparent",
      isDisabled: true,
      variant: "ghost",
    },
  ],
  defaultVariants: {
    color: "neutral",
    size: "regular",
    variant: "solid",
  },
  variants: {
    color: {
      neutral: "bg-fill text-content-fg",
      primary:
        // The ring is the accent too, so on an accent fill it gets a gap in
        // the window colour to stay readable.
        "bg-primary-surface text-primary-fg data-[pressed]:bg-primary-surface-pressed data-[focus-visible]:ring-offset-2 data-[focus-visible]:ring-offset-content",
    },
    isDisabled: {
      true: "bg-fill-quaternary text-content-fg-tertiary",
    },
    // Capture is the recording bar's own control: 48 by 40 under a 28px
    // glyph, taking the panel radius a standalone control of that height
    // carries rather than the tighter segment corner.
    size: {
      capture:
        "h-10 min-w-12 gap-control-inset rounded-panel px-control-inset text-body [&_svg.lucide]:size-icon-xl",
      regular: "",
    },
    variant: {
      ghost:
        "bg-transparent data-[hovered]:bg-fill-tertiary data-[pressed]:bg-fill",
      solid: "border-none",
    },
  },
});
