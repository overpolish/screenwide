// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type MouseEventHandler, type ReactNode } from "react";

import { ColorButton } from "../../../../../composites/color-button/color-button";
import { cn } from "../../../../../lib/styling";
import {
  useColorPanel,
  usesSystemColorPanel,
} from "../../../../../primitives/input-fields/use-color-panel";

import type { SwatchSize } from "../../../../../primitives/swatch/swatch-styles";

/**
 * A colour that is picked by pressing it. The Mac app opens the system
 * Colours panel. Elsewhere the platform's colour input lies over the swatch
 * at zero opacity, so the press lands on the input itself, as on the colour
 * grid's Custom tile: Safari opens no picker for an input asked from out of
 * sight (`showPicker` on a hidden input). A secondary click is heard on the
 * wrapper, so it reaches past that input; `children` lie over both.
 */
export function ColorPickerSwatch({
  "aria-label": ariaLabel,
  children,
  className,
  isDisabled,
  onChange,
  onContextMenu,
  size,
  value,
}: {
  "aria-label": string;
  onChange: (value: string) => void;
  /** `#rrggbb`. */
  value: string;
  children?: ReactNode;
  className?: string;
  isDisabled?: boolean;
  onContextMenu?: MouseEventHandler<HTMLSpanElement>;
  size?: SwatchSize;
}) {
  const systemPanel = usesSystemColorPanel();
  const panel = useColorPanel({ onChange });
  return (
    <span
      className={cn("relative inline-flex", className)}
      onContextMenu={onContextMenu}
    >
      <ColorButton
        aria-label={ariaLabel}
        isDisabled={isDisabled}
        onPress={() => {
          if (systemPanel) panel.open(value);
        }}
        size={size}
        value={value}
      />
      {systemPanel ? null : (
        <input
          aria-label={ariaLabel}
          className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
          disabled={isDisabled}
          onChange={(event) => {
            onChange(event.currentTarget.value.toUpperCase());
          }}
          type="color"
          // The input takes `#rrggbb` in lower case only.
          value={value.slice(0, 7).toLowerCase()}
        />
      )}
      {children}
    </span>
  );
}
