// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { CSSProperties, MouseEventHandler, ReactNode } from "react";
import { ToggleButton } from "react-aria-components";

import { motionDurationCss } from "../../lib/motion";
import { cn } from "../../lib/styling";

import { swatchStyles, type SwatchSize } from "./swatch-styles";

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
  size?: SwatchSize;
  /** Paint beyond a plain colour, such as an image. */
  style?: CSSProperties;
};

// The chosen swatch takes an outline rather than a ring, held off the
// circle, so it reads over any background and keeps the swatch's own edge.
// Choosing another, the outline closes in onto the old swatch as it fades and
// opens out from the new one's edge, so the choice is seen to move.
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
  size = "md",
  style,
}: SwatchProps) {
  const isPainted = color !== undefined || style !== undefined;
  const transitionDuration = useReducedMotion()
    ? "0s"
    : motionDurationCss("travel");
  return (
    <ToggleButton
      aria-label={label}
      className={cn(
        swatchStyles(isPainted, size),
        "outline-2 -outline-offset-2 outline-transparent outline-solid transition-[background-color,box-shadow,opacity,outline-color,outline-offset] ease-out data-[selected]:outline-offset-2 data-[selected]:outline-primary",
        className,
      )}
      id={id}
      isDisabled={isDisabled}
      isSelected={isSelected}
      onContextMenu={onContextMenu}
      onHoverEnd={onHoverEnd}
      onPress={onPress}
      style={{
        transitionDuration,
        ...(color === undefined ? style : { background: color, ...style }),
      }}
    >
      {children}
    </ToggleButton>
  );
}
