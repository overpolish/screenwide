// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { KeyboardEvent, useRef } from "react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

export type PositionGridItem<T extends string> = { id: T; label: string };

const COLUMNS = 3;

/**
 * One of nine places on a picture, picked by clicking where it is: a small
 * picture of three rows of three cells, the chosen one filled. Its items
 * are in reading order, the top row first. The picture keeps its own left
 * and right in every language, so the grid is laid out left to right.
 *
 * A radio group, so the arrow keys move the choice; up and down move it a
 * row, where a group of radios would otherwise step through them in order.
 */
export function PositionGrid<T extends string>({
  "aria-labelledby": ariaLabelledBy,
  isDisabled,
  items,
  onChange,
  value,
}: {
  items: readonly PositionGridItem<T>[];
  onChange: (id: T) => void;
  value: T;
  "aria-labelledby"?: string;
  isDisabled?: boolean;
}) {
  const frameRef = useRef<HTMLDivElement>(null);

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
        className="grid w-fit grid-cols-3 gap-tight rounded-control bg-fill-tertiary p-tight"
        isDisabled={isDisabled}
        onChange={(id) => {
          onChange(id as T);
        }}
        orientation="horizontal"
        value={value}
      >
        {items.map((item) => (
          <RadioField aria-label={item.label} key={item.id} value={item.id}>
            <RadioButton
              className={cn(
                "group flex h-4 w-7 cursor-default items-center justify-center rounded-[calc(var(--radius-control)-var(--spacing-tight))] bg-fill-secondary data-[disabled]:opacity-50 data-[hovered]:bg-fill data-[selected]:bg-primary-surface",
                focusStyles,
                elementFocusVisible,
              )}
            >
              {/* A caption's line where it is chosen, a point where it could
                  go. */}
              <span className="size-1 rounded-full bg-content-fg-tertiary group-data-[selected]:w-4 group-data-[selected]:bg-primary-fg" />
            </RadioButton>
          </RadioField>
        ))}
      </RadioGroup>
    </div>
  );
}
