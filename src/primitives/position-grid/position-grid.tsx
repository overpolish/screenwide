// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { KeyboardEvent, useId, useRef } from "react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { motionDurations, motionEasings } from "../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";

export type PositionGridItem<T extends string> = { id: T; label: string };

const COLUMNS = 3;

// Drawn as the segmented control is, since it is the same kind of choice:
// cells on a plate on the fill, inset by 2px, the chosen cell filled in the
// primary colour on a fill that slides between them. A cell is 28 by 16, a
// size only this grid needs, its corner the plate's less the inset.
const cellClassName = [
  "group relative flex h-4 w-7 cursor-default items-center justify-center rounded-[calc(var(--radius-control)-0.125rem)] outline-none",
  focusStyles,
  elementFocusVisible,
];

// A dot where the overlay could go, a bar where it sits.
const markClassName =
  "relative z-10 h-1 w-1 rounded-full bg-fg-tertiary transition-[width,background-color] group-data-[hovered]:bg-fg-secondary group-data-[selected]:w-4 group-data-[selected]:bg-on-primary";

// One of nine places on a picture, picked by clicking where it is. The
// picture keeps its own left and right in every language, so the grid is
// laid out left to right. A radio group, so the arrow keys move the choice;
// up and down move it a row, where a group of radios would otherwise step
// through them in order.
export function PositionGrid<T extends string>({
  "aria-labelledby": ariaLabelledBy,
  isDisabled,
  items,
  onChange,
  value,
}: {
  /** The nine places in reading order, the top row first. */
  items: readonly PositionGridItem<T>[];
  onChange: (id: T) => void;
  value: T;
  "aria-labelledby"?: string;
  isDisabled?: boolean;
}) {
  const frameRef = useRef<HTMLDivElement>(null);
  const layoutId = `position-grid-${useId()}`;
  const prefersReducedMotion = useReducedMotion();
  const fillTransition = {
    duration: prefersReducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };

  const moveRow = (event: KeyboardEvent) => {
    const step =
      event.key === "ArrowUp"
        ? -COLUMNS
        : event.key === "ArrowDown"
          ? COLUMNS
          : 0;
    if (step === 0) return;
    // Taken before the radios step through in order, and kept from them.
    event.preventDefault();
    event.stopPropagation();
    const next = items.findIndex((item) => item.id === value) + step;
    if (next < 0 || next >= items.length) return;
    onChange(items[next].id);
    frameRef.current?.querySelectorAll("input")[next]?.focus();
  };

  return (
    <div dir="ltr" onKeyDownCapture={moveRow} ref={frameRef}>
      <RadioGroup
        aria-labelledby={ariaLabelledBy}
        className="grid w-fit grid-cols-3 gap-0.5 rounded-control bg-fill p-0.5 transition-opacity data-[disabled]:opacity-40"
        isDisabled={isDisabled}
        onChange={(id) => {
          onChange(id as T);
        }}
        orientation="horizontal"
        value={value}
      >
        {items.map((item) => (
          <RadioField aria-label={item.label} key={item.id} value={item.id}>
            <RadioButton className={cn(cellClassName)}>
              {({ isSelected }) => (
                <>
                  {isSelected ? (
                    <motion.span
                      aria-hidden="true"
                      className="absolute inset-0 rounded-[inherit] bg-primary primary-raised"
                      layoutId={layoutId}
                      transition={fillTransition}
                    />
                  ) : null}
                  <span className={markClassName} />
                </>
              )}
            </RadioButton>
          </RadioField>
        ))}
      </RadioGroup>
    </div>
  );
}
