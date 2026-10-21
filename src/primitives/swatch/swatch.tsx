// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CSSProperties, MouseEventHandler, ReactNode } from "react";
import { ToggleButton } from "react-aria-components";

import { cn } from "../../lib/styling";

import { swatchStyles } from "./swatch-styles";

type SwatchProps = {
  "aria-label": string;
  children?: ReactNode;
  className?: string;
  /** CSS colour. */
  color?: string;
  /** Key within a group of swatches. */
  id?: string;
  isDisabled?: boolean;
  isSelected?: boolean;
  onContextMenu?: MouseEventHandler<HTMLDivElement>;
  onHoverEnd?: () => void;
  onPress?: () => void;
  /** Paint beyond a plain colour, such as an image. */
  style?: CSSProperties;
};

// The chosen swatch takes an outline rather than a ring, held off the
// square, so it reads over any background and keeps the swatch's own edge.
export function Swatch({
  "aria-label": label,
  children,
  className,
  color,
  id,
  isDisabled,
  isSelected,
  onContextMenu,
  onHoverEnd,
  onPress,
  style,
}: SwatchProps) {
  const isPainted = color !== undefined || style !== undefined;
  return (
    <ToggleButton
      aria-label={label}
      className={cn(
        swatchStyles(isPainted),
        "data-[selected]:outline-2 data-[selected]:outline-primary data-[selected]:outline-solid",
        className,
      )}
      id={id}
      isDisabled={isDisabled}
      isSelected={isSelected}
      onContextMenu={onContextMenu}
      onHoverEnd={onHoverEnd}
      onPress={onPress}
      style={color === undefined ? style : { background: color, ...style }}
    >
      {children}
    </ToggleButton>
  );
}
