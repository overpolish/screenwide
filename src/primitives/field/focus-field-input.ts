// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { MouseEvent } from "react";

/**
 * Makes the whole field a target for the caret: a press on its padding or
 * its glyphs focuses the input, as a native text field does, rather than
 * landing on a frame that takes no focus. A press on the input itself or on
 * a button inside the field, such as a clear button, is left alone. The
 * default is prevented so the press does not blur the input it focuses.
 */
export const focusFieldInput = (event: MouseEvent<HTMLElement>) => {
  if ((event.target as HTMLElement).closest("input, button")) return;
  const input = event.currentTarget.querySelector("input");
  if (!input || input.disabled) return;
  event.preventDefault();
  input.focus();
};
