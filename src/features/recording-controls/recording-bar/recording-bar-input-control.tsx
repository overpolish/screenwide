// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { Button } from "../../../primitives/button/button";
import { IconToggleButton } from "../../../primitives/button/icon-button";

type RecordingBarInputControlProps = {
  /** The glyph shown while the input is on. */
  children: ReactNode;
  /** The device the input is set to; pressing it opens the choices. */
  label: string;
  /** Names the part that opens the choices. */
  menuLabel: string;
  /** The glyph shown while the input is off. */
  off: ReactNode;
  /** `fromKeyboard` says the menu was opened without a pointer, so what
   * opens can take focus rather than leaving it on the button. */
  onMenuPress: (anchor: DOMRect, fromKeyboard: boolean) => void;
  /** Names the toggle. */
  "aria-label"?: string;
  /** Drawn over the toggle's corner, e.g. a warning or a lock. */
  badge?: ReactNode;
  /** Drawn under the label, e.g. a level meter. */
  detail?: ReactNode;
  isMenuDisabled?: boolean;
  isSelected?: boolean;
  isToggleDisabled?: boolean;
  onChange?: (isSelected: boolean) => void;
  /** Wraps the toggle, as for a tooltip in a window of its own. */
  renderToggle?: (toggle: ReactNode) => ReactNode;
};

// A plain stand-in for the bar's input controls, an icon toggle beside a
// button naming the device, kept until the new recording panel replaces the
// bar. It is not designed: it only keeps the bar working.
export function RecordingBarInputControl({
  "aria-label": ariaLabel,
  badge,
  children,
  detail,
  isMenuDisabled = false,
  isSelected = false,
  isToggleDisabled = false,
  label,
  menuLabel,
  off,
  onChange,
  onMenuPress,
  renderToggle,
}: RecordingBarInputControlProps) {
  const toggle = (
    <span className="relative inline-flex">
      <IconToggleButton
        aria-label={ariaLabel}
        isDisabled={isToggleDisabled}
        isSelected={isSelected}
        off={off}
        onChange={onChange}
      >
        {children}
      </IconToggleButton>
      {badge}
    </span>
  );

  return (
    <span className="inline-flex items-center">
      {renderToggle ? renderToggle(toggle) : toggle}
      <span className="flex flex-col">
        <Button
          aria-label={menuLabel}
          isDisabled={isMenuDisabled}
          onPress={(event) => {
            onMenuPress(
              event.target.getBoundingClientRect(),
              event.pointerType === "keyboard" ||
                event.pointerType === "virtual",
            );
          }}
          variant="ghost"
        >
          <span className="max-w-32 truncate">{label}</span>
        </Button>
        {detail}
      </span>
    </span>
  );
}
