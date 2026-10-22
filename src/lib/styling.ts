// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { clsx, ClassValue } from "clsx";

import { twMerge } from "./tailwind-merge";

export function cn(...classes: ClassValue[]) {
  return twMerge(clsx(classes));
}

// The focus ring's transition, which every control shares and which comes
// last in their classes, so it is the transition they keep. It carries
// opacity too, so a control fades into and out of its disabled state.
export const focusStyles =
  "outline-none ring-focus transition-[box-shadow,background-color,color,border-color,opacity]";

// Interactive elements where focus is not required on non-keyboard
// interaction, e.g. buttons: a 3px halo from the control's edge, no gap.
// React Aria's keyboard-modality tracking decides visibility on its own:
// WebKit's native :focus-visible does not reliably follow focus that a key
// handler moves programmatically, e.g. arrow keys inside a toolbar.
export const elementFocusVisible = "data-[focus-visible]:ring-3";

export const groupFocusVisible = "group-data-[focus-visible]:ring-3";
