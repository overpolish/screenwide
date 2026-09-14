// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock } from "lucide-react";
import { MouseEventHandler } from "react";
import { useFocusRing } from "react-aria";
import { Button } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

import { useColorPanel, usesSystemColorPanel } from "./use-color-panel";

/**
 * A colour well: a square of the colour itself, on the control height and
 * radius its neighbours take. On the Mac app pressing it opens the system
 * Colours panel; everywhere else a native `<input type="color">` sits over it
 * at zero opacity. The well follows that input's keyboard focus visibility.
 *
 * A white colour would otherwise have no edge, so an inset hairline-free ring
 * keeps the well reading as a control whatever it holds.
 */
export function ColorSwatch({
  ariaLabel,
  isDisabled,
  isLocked,
  onChange,
  onContextMenu,
  value,
}: {
  ariaLabel: string;
  value: string;
  isDisabled?: boolean;
  isLocked?: boolean;
  onChange?: (value: string) => void;
  onContextMenu?: MouseEventHandler<HTMLSpanElement>;
}) {
  const systemPanel = usesSystemColorPanel();
  const { focusProps, isFocusVisible } = useFocusRing();
  const colorPanel = useColorPanel({
    onChange: (next) => {
      onChange?.(next);
    },
  });

  return (
    <span
      className={cn(
        "relative inline-block size-control-height shrink-0 cursor-default overflow-hidden rounded-control align-middle",
        "inset-ring-1 inset-ring-content-fg-quaternary",
        focusStyles,
        "has-[[data-focus-visible]]:ring-3",
        isDisabled && "opacity-50",
      )}
      onContextMenu={onContextMenu}
      style={{ backgroundColor: value }}
    >
      {systemPanel ? (
        <Button
          aria-label={ariaLabel}
          className={cn(
            "absolute inset-0 size-full cursor-default bg-transparent",
            focusStyles,
            elementFocusVisible,
          )}
          isDisabled={isDisabled}
          onPress={() => {
            colorPanel.open(value);
          }}
        />
      ) : (
        <input
          {...focusProps}
          aria-label={ariaLabel}
          className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
          data-focus-visible={isFocusVisible || undefined}
          disabled={isDisabled}
          onChange={(event) => {
            onChange?.(event.currentTarget.value.toUpperCase());
          }}
          type="color"
          value={value}
        />
      )}
      {isLocked ? (
        <span className="pointer-events-none absolute inset-0 flex items-center justify-center text-white drop-shadow-[0_0_2px_rgb(0_0_0/0.8)] [&_svg.lucide]:size-icon-mini [&_svg]:shrink-0">
          <Lock aria-hidden="true" />
        </span>
      ) : null}
    </span>
  );
}
