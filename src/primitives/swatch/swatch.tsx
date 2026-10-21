// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CSSProperties, MouseEventHandler, ReactNode } from "react";
import { ToggleButton } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";

type SwatchProps = {
  "aria-label": string;
  children?: ReactNode;
  className?: string;
  /** The colour it shows, as CSS. */
  color?: string;
  /** Its key when it sits in a group of swatches. */
  id?: string;
  isDisabled?: boolean;
  isSelected?: boolean;
  onContextMenu?: MouseEventHandler<HTMLDivElement>;
  onHoverEnd?: () => void;
  onPress?: () => void;
  /** Paint for anything richer than a colour, such as a rendered picture. */
  style?: CSSProperties;
};

// A choice shown as what it paints: a colour, a picture, or a glyph for one
// that stands for an action. Drawn as Color Button's square is, so the two
// read as one thing: a control's height square on the control radius, a
// hairline edge so white on white or black on black still reads, dimming
// while pressed. The chosen one is checked, so it takes an outline in the
// primary colour, held off the square so its own edge stays; an outline
// rather than a ring, so it reads over any background. A glyph swatch sits
// on the fill and deepens under the pointer.
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
        "flex size-control-height shrink-0 cursor-default items-center justify-center overflow-hidden rounded-control outline-offset-2 transition-[background-color,box-shadow,opacity]",
        "[&_svg]:size-icon [&_svg]:shrink-0",
        isPainted
          ? "inset-ring-1 inset-ring-divider data-[pressed]:opacity-80"
          : "bg-fill text-fg-secondary data-[hovered]:bg-fill-strong data-[pressed]:bg-fill-strong",
        "data-[selected]:outline-2 data-[selected]:outline-primary data-[selected]:outline-solid",
        "data-[disabled]:opacity-40",
        focusStyles,
        elementFocusVisible,
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
