// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

/**
 * Shared bezel, label, and input styling for text-entry fields. The bezel
 * follows React Aria's focus visibility on itself or its input, so pointer
 * focus does not introduce a keyboard navigation ring.
 *
 * The bezel's colours are the platform tokens. Fluent's text box carries the
 * elevation border with a strong bottom edge, lifts to a solid fill while
 * edited and underlines itself in the accent, two pixels deep; those are
 * no-ops on macOS, whose tokens keep the bezel plain. The second underline
 * pixel is an inset shadow so the box does not grow on focus.
 */
export const fieldVariants = tv({
  slots: {
    base: "group flex min-w-0 flex-col gap-control",
    field: [
      "relative flex h-control-height flex-row items-center rounded-control bg-control-fill text-content-fg outline-none transition-[background-color,box-shadow,border-color] control-stroke border-b-control-alt-stroke",
      // A plain `hover:` because the text field's bezel is a bare div, not a
      // React Aria group.
      "hover:bg-control-fill-hover focus-within:bg-control-fill-active",
      "windows:focus-within:border-b-primary-surface windows:focus-within:inset-shadow-[0_-1px_0_var(--color-primary-surface)]",
      focusStyles,
      "data-[focus-visible]:ring-3 has-[[data-focus-visible]]:ring-3",
      "windows:data-[focus-visible]:ring-2 windows:data-[focus-visible]:ring-offset-1 windows:data-[focus-visible]:ring-offset-focus-ring-inner",
      "windows:has-[[data-focus-visible]]:ring-2 windows:has-[[data-focus-visible]]:ring-offset-1 windows:has-[[data-focus-visible]]:ring-offset-focus-ring-inner",
      "group-data-[invalid]:ring-3 group-data-[invalid]:ring-error",
      "group-data-[disabled]:bg-control-fill-disabled group-data-[disabled]:text-control-fg-disabled group-data-[disabled]:border-b-control-stroke",
    ],
    input: [
      "w-full min-w-0 bg-transparent text-body text-content-fg outline-none",
      "placeholder:text-content-fg-secondary selection:bg-primary-tint",
      "group-data-[disabled]:text-control-fg-disabled",
    ],
    // Fluent insets its text by 10; the macOS bezel by the control inset.
    inputWrapper:
      "flex min-w-0 flex-1 flex-row items-center justify-between gap-control-inset px-control-inset outline-none windows:px-2.5",
    label: "text-body text-content-fg",
  },
  variants: {
    centered: { true: { input: "text-center" } },
    rightAligned: { true: { input: "text-right" } },
  },
});
