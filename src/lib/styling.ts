// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx, ClassValue } from "clsx";

import { twMerge } from "./tailwind-merge";

export function cn(...classes: ClassValue[]) {
  return twMerge(clsx(classes));
}

export const focusStyles =
  "outline-none ring-focus-ring transition-[box-shadow,background-color,color,border-color]";

// Interactive elements where focus is not required on non-keyboard interaction, e.g., buttons
// AppKit draws the focus ring as a 3px halo from the control edge, no gap.
// React Aria's keyboard-modality tracking decides visibility on its own:
// WebKit's native :focus-visible does not reliably follow focus that a key
// handler moves programmatically, e.g. arrow keys inside a toolbar.
// Fluent draws two strokes instead: a 1px inner in the opposite colour hugging
// the control, then a 2px outer in the text colour. The ring offset is the
// inner stroke. The `windows:` variant has higher specificity than a plain
// data variant, so it also wins over a component's own ring offset. Written
// out in full because Tailwind only generates classes it finds literally.
export const elementFocusVisible =
  "data-[focus-visible]:ring-3 windows:data-[focus-visible]:ring-2 windows:data-[focus-visible]:ring-offset-1 windows:data-[focus-visible]:ring-offset-focus-ring-inner";

export const groupFocusVisible =
  "group-data-[focus-visible]:ring-3 windows:group-data-[focus-visible]:ring-2 windows:group-data-[focus-visible]:ring-offset-1 windows:group-data-[focus-visible]:ring-offset-focus-ring-inner";
