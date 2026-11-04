// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { IconToggleButton } from "../../../primitives/button/icon-button";

/** An option with two looks, as one toggle: named by the look it turns on,
 * its glyph drawing the look it has and moving to the other when pressed,
 * so the button needs no fill to say which it is. Its name is shown after
 * the glyph, except where there is no room for words, such as among the
 * glyphs at the end of an island's row. */
export function OptionToggle({
  glyph,
  isLabelHidden = false,
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
  /** Names the toggle for assistive technology only. */
  isLabelHidden?: boolean;
}) {
  return (
    <IconToggleButton
      aria-label={isLabelHidden ? label : undefined}
      isSelected={isOn}
      label={isLabelHidden ? undefined : label}
      onChange={onChange}
    >
      {({ isSelected }) => glyph(isSelected)}
    </IconToggleButton>
  );
}
