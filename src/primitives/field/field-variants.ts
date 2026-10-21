// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { focusStyles } from "../../lib/styling";
import { tv } from "../../lib/variants";

/**
 * The field every value control shares: its label, frame, value text and the
 * glyphs or units beside the value. Text, search and number fields type into
 * it; a select is pressed to choose, and extends it.
 *
 * The frame is a control's height and corners, so it lines up with buttons
 * in a row. Its material says how it is used: sunken for a field typed into,
 * the counterpart of a button's raised one; raised for a trigger pressed to
 * choose. A sunken field shows the focus ring whenever its input has focus,
 * by click as by keyboard, as text fields on both platforms mark the one
 * being typed into, through a plain `focus-within` since the frame is a bare
 * div; a raised one leaves focus to its trigger. An invalid field takes the
 * ring in the error colour.
 */
export const fieldVariants = tv({
  defaultVariants: { material: "sunken" },
  slots: {
    base: "group flex min-w-0 flex-col gap-inline data-[disabled]:opacity-40",
    field: [
      "relative flex h-control-height flex-row items-center rounded-control text-fg outline-none",
      focusStyles,
      "group-data-[invalid]:ring-3 group-data-[invalid]:ring-error/50",
    ],
    input:
      "w-full min-w-0 bg-transparent text-body text-fg outline-none placeholder:text-fg-tertiary",
    inputWrapper:
      "flex min-w-0 flex-1 flex-row items-center justify-between gap-inline px-inset outline-none",
    label: "text-label text-fg",
    // Glyphs and units say what the value is rather than being it, so they
    // sit a step back, in the placeholder's colour, at the icon size body
    // text takes.
    section:
      "shrink-0 text-fg-tertiary [&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
  },
  variants: {
    centered: { true: { input: "text-center" } },
    material: {
      raised: { field: "control-raised" },
      sunken: { field: "control-sunken focus-within:ring-3" },
    },
    rightAligned: { true: { input: "text-end" } },
  },
});
