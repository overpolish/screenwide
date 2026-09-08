// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { ReactNode, useId } from "react";
import {
  Selection,
  ToggleButton,
  ToggleButtonGroup,
} from "react-aria-components";

import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

export type PillGroupItem = {
  id: string;
  label: string;
  ariaLabel?: string;
  icon?: ReactNode;
};

const selectedId = (selection: Selection) => {
  if (selection === "all") return null;
  const first = selection.values().next().value;
  return typeof first === "string" ? first : null;
};

export function PillGroup({
  "aria-label": ariaLabel,
  className,
  disabledIds,
  display = "icon",
  isDisabled,
  itemClassName,
  items,
  onSelectionChange,
  selected,
}: {
  "aria-label": string;
  items: PillGroupItem[];
  onSelectionChange: (id: string) => void;
  selected: string;
  className?: string;
  /** Items that stay visible but cannot be picked. */
  disabledIds?: string[];
  display?: "icon" | "icon-label" | "label";
  isDisabled?: boolean;
  /** Applies shared geometry overrides to every item. */
  itemClassName?: string;
}) {
  const selectionId = useId();
  const prefersReducedMotion = useReducedMotion();

  return (
    <ToggleButtonGroup
      aria-label={ariaLabel}
      // A macOS 26 segmented control, measured from a live capture: a track on
      // the fill, 24px tall with a 2px inset, and the selected segment filled
      // with the accent under white content. The track sizes from its
      // segments so larger geometry, such as the screen recording mode
      // picker, keeps the same construction.
      className={cn(
        "flex items-stretch gap-tight rounded-control bg-fill p-tight",
        className,
      )}
      disallowEmptySelection
      isDisabled={isDisabled}
      onSelectionChange={(selection) => {
        const id = selectedId(selection);
        if (id !== null) onSelectionChange(id);
      }}
      selectedKeys={new Set([selected])}
      selectionMode="single"
    >
      {items.map((item) => (
        <ToggleButton
          aria-label={item.ariaLabel ?? item.label}
          className={cn(
            "group relative flex h-5 cursor-default items-center justify-center rounded-[calc(var(--radius-control)-var(--spacing-tight))] text-body text-content-fg outline-none transition-colors select-none",
            display === "icon" ? "w-icon-large" : "px-control-inset",
            "data-[selected]:text-primary-fg data-[disabled]:text-content-fg-tertiary data-[disabled]:data-[selected]:text-content-fg-tertiary",
            "[&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
            focusStyles,
            elementFocusVisible,
            itemClassName,
          )}
          id={item.id}
          isDisabled={disabledIds?.includes(item.id)}
          key={item.id}
          style={{
            transitionDuration: prefersReducedMotion
              ? "0s"
              : motionDurationCss("state"),
          }}
        >
          {({ isSelected }) => (
            <>
              {isSelected ? (
                <motion.span
                  aria-hidden="true"
                  className="absolute inset-0 transform-gpu rounded-[inherit] bg-primary-surface backface-hidden group-data-[disabled]:bg-fill-quaternary will-change-transform"
                  layoutId={`pill-selection-${selectionId}`}
                  transition={{
                    duration: prefersReducedMotion ? 0 : motionDurations.travel,
                    ease: motionEasings.out,
                  }}
                />
              ) : null}
              <span className="relative z-10 flex min-w-0 items-center justify-center gap-control whitespace-nowrap">
                {display !== "label" ? item.icon : null}
                {display !== "icon" ? item.label : null}
              </span>
            </>
          )}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
