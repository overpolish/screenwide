// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { elementFocusVisible, focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

const buttonControlStyles =
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
      // The bezel. Its rest, hover, pressed and stroke colours are all
      // platform tokens: on macOS hover equals rest and the stroke is
      // transparent, on Windows each state is Fluent's. A toggle held down
      // (`aria-pressed`) shows the pressed fill too.
      class: "control-stroke aria-pressed:bg-control-fill-pressed",
      color: "neutral",
      variant: "solid",
    },
    {
      class: "primary-stroke",
      color: "primary",
      variant: "solid",
    },
    {
      // Without a fill the accent moves to the label. The hover fill is the
      // subtle one on both platforms, over the primary colour's Windows
      // hover. Compound classes land after the disabled variant's, so the
      // disabled colour is restated here.
      class:
        "text-primary data-[pressed]:text-primary data-[disabled]:text-control-fg-disabled windows:data-[hovered]:bg-control-subtle-hover",
      color: "primary",
      variant: "ghost",
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
      neutral:
        "bg-control-fill text-content-fg data-[hovered]:bg-control-fill-hover data-[pressed]:bg-control-fill-pressed data-[pressed]:text-control-fg-pressed",
      primary:
        // The macOS ring is the accent too, so on an accent fill it gets a
        // gap in the window colour to stay readable. Only Fluent's accent
        // button reacts to hover.
        "bg-primary-surface text-primary-fg windows:data-[hovered]:bg-primary-surface-hover data-[pressed]:bg-primary-surface-pressed data-[pressed]:text-primary-fg-pressed data-[focus-visible]:ring-offset-2 data-[focus-visible]:ring-offset-content",
    },
    isDisabled: {
      true: "bg-control-fill-disabled text-control-fg-disabled",
    },
    // Capture is the recording bar's own control: 48 by 40 under a 28px
    // glyph, taking the panel radius a standalone control of that height
    // carries rather than the tighter segment corner.
    size: {
      capture:
        "h-10 min-w-12 gap-control-inset rounded-capture px-control-inset text-body [&_svg.lucide]:size-icon-xl",
      regular: "",
    },
    variant: {
      ghost:
        "bg-transparent data-[hovered]:bg-control-subtle-hover data-[pressed]:bg-control-subtle-pressed",
      solid: "border-none",
    },
  },
});
