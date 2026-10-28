// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { IconToggleButton } from "../../../primitives/button/icon-button";

/** An annotation option with two looks, as one toggle: named by the look it
 * turns on, its glyph drawing the look it has and moving to the other when
 * pressed, so the button needs no fill to say which it is. */
export function OptionToggle({
  glyph,
  isOn,
  label,
  onChange,
}: {
  /** The glyph for the look in hand, animating as `isOn` changes. */
  glyph: (isOn: boolean) => ReactNode;
  isOn: boolean;
  /** The look the toggle turns on. */
  label: string;
  onChange: (isOn: boolean) => void;
}) {
  return (
    <IconToggleButton aria-label={label} isSelected={isOn} onChange={onChange}>
      {({ isSelected }) => glyph(isSelected)}
    </IconToggleButton>
  );
}
