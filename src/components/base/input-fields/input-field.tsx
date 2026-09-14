// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { focusStyles } from "../../../lib/styling";
import { tv } from "../../../lib/variants";

/**
 * Shared bezel, label, and input styling for text-entry fields. The bezel
 * follows React Aria's focus visibility on itself or its input, so pointer
 * focus does not introduce a keyboard navigation ring.
 */
export const fieldVariants = tv({
  slots: {
    base: "group flex min-w-0 flex-col gap-control",
    field: [
      "relative flex h-control-height flex-row items-center rounded-control bg-fill text-content-fg outline-none transition-colors",
      focusStyles,
      "data-[focus-visible]:ring-3 has-[[data-focus-visible]]:ring-3",
      "group-data-[invalid]:ring-3 group-data-[invalid]:ring-error",
      "group-data-[disabled]:bg-fill-quaternary group-data-[disabled]:text-content-fg-tertiary",
    ],
    input: [
      "w-full min-w-0 bg-transparent text-body text-content-fg outline-none",
      "placeholder:text-content-fg-secondary selection:bg-primary-tint",
      "group-data-[disabled]:text-content-fg-tertiary",
    ],
    inputWrapper:
      "flex min-w-0 flex-1 flex-row items-center justify-between gap-control-inset px-control-inset outline-none",
    label: "text-body text-content-fg",
  },
  variants: {
    centered: { true: { input: "text-center" } },
    rightAligned: { true: { input: "text-right" } },
  },
});
