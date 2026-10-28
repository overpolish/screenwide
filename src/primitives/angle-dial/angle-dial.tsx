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
const FREE: DialSteps = {
  drag: { plain: 1, shifted: EIGHTH_TURN },
  key: { plain: 1, shifted: 10 },
};

/** An eighth of a turn at a time, for an aim of a few directions; held
 * Shift frees a drag or an arrow to the degree. */
const STEPPED: DialSteps = {
  drag: { plain: EIGHTH_TURN, shifted: 1 },
  key: { plain: EIGHTH_TURN, shifted: 1 },
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

// On the island's black the neutral dot is the island's ink.
const islandNotchColors = { ...notchColors, neutral: "bg-fg" } as const;

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
  /** Rests on the eighths of a turn, as a counter's aim does; otherwise it
   * turns to the degree. */
  isStepped?: boolean;
};

export function AngleDial({
  children,
  isStepped = false,
  ...props
}: AngleDialProps) {
  const steps = isStepped ? STEPPED : FREE;
  return useSkin() === "island" ? (
    <IslandAngleDial
      aria-label={props["aria-label"]}
      isDisabled={props.isDisabled ?? false}
      isStepped={isStepped}
      onChange={props.onChange}
      steps={steps}
      value={props.value}
    >
      {children ?? (
        <span className="relative size-4">
          <span
            className={cn(
              "absolute top-0 left-1/2 size-1.5 -translate-x-1/2 rounded-full",
              islandNotchColors[props.color ?? "neutral"],
            )}
          />
        </span>
      )}
    </IslandAngleDial>
  ) : (
    <WindowAngleDial {...props} steps={steps} />
  );
}

function WindowAngleDial({
  "aria-label": label,
  className,
  color = "neutral",
  isDisabled = false,
  onChange,
  steps,
  value,
}: Omit<AngleDialProps, "children" | "isStepped"> & { steps: DialSteps }) {
  const { dialProps, notchRef } = useAngleDial({
    isDisabled,
    label,
    onChange,
    steps,
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
