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
export const elementFocusVisible = "data-[focus-visible]:ring-3";

export const groupFocusVisible = "group-data-[focus-visible]:ring-3";
