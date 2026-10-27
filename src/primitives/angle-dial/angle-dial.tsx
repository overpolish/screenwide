// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { useSkin } from "../skin/skin";

import { wrapAngle } from "./angle-geometry";
import { IslandAngleDial } from "./island-angle-dial";
import { type DialSteps, EIGHTH_TURN, useAngleDial } from "./use-angle-dial";

/** A degree at a time, as fine as the dial can be read; held Shift snaps a
 * drag to the eight aims that matter and takes the arrows ten at a time. */
const STEPS: DialSteps = {
  drag: { plain: 1, shifted: EIGHTH_TURN },
  key: { plain: 1, shifted: 10 },
};

// The knob: a disc in the knob material, light in both appearances as
// anything turned or slid by hand is, at a control's height so it sits in a
// row of fields and buttons as one of them. While dragged its bevel flattens,
// as a pressed button's does. The dot is the value, drawn as a rotation of
// the whole span rather than a moved dot, so the turn stays on the
// compositor.
const dialStyles = cn(
  "group relative size-control-height shrink-0 cursor-default touch-none rounded-full select-none",
  "control-knob data-[dragging]:[--tw-shadow:var(--control-bevel-pressed)]",
  "data-[disabled]:opacity-40",
  focusStyles,
  elementFocusVisible,
);

const notchStyles =
  "absolute top-1 left-1/2 size-1.5 -translate-x-1/2 rounded-full transition-colors";

// A neutral dot is the knob's own ink, dark on the light face in both
// appearances; a coloured one names what it aims, such as an axis in its
// colour. Hues reuse the status colours so the app keeps one red, one green
// and one blue.
const notchColors = {
  blue: "bg-info",
  green: "bg-success",
  neutral: "bg-on-knob",
  red: "bg-error",
} as const;

export type AngleDialProps = {
  "aria-label": string;
  onChange: (degrees: number) => void;
  /** Angle in degrees. */
  value: number;
  /** On the island, what turns in place of the notch, drawn pointing at
   * twelve o'clock. */
  children?: ReactNode;
  className?: string;
  color?: keyof typeof notchColors;
  isDisabled?: boolean;
};

export function AngleDial({ children, ...props }: AngleDialProps) {
  return useSkin() === "island" ? (
    <IslandAngleDial
      aria-label={props["aria-label"]}
      isDisabled={props.isDisabled ?? false}
      onChange={props.onChange}
      value={props.value}
    >
      {children ?? (
        <span className="relative size-4">
          <span className="absolute top-0 left-1/2 size-1.5 -translate-x-1/2 rounded-full bg-fg" />
        </span>
      )}
    </IslandAngleDial>
  ) : (
    <WindowAngleDial {...props} />
  );
}

function WindowAngleDial({
  "aria-label": label,
  className,
  color = "neutral",
  isDisabled = false,
  onChange,
  value,
}: Omit<AngleDialProps, "children">) {
  const { dialProps, notchRef } = useAngleDial({
    isDisabled,
    label,
    onChange,
    steps: STEPS,
    value,
  });
  return (
    <div {...dialProps} className={cn(dialStyles, className)}>
      <span
        className="absolute inset-0 transform-gpu"
        style={{ rotate: `${String(wrapAngle(value))}deg` }}
      >
        <span className={cn(notchStyles, notchColors[color])} ref={notchRef} />
      </span>
    </div>
  );
}
