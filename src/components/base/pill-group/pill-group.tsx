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
import { defaultButtonControlStyles } from "../button/button-variants";
import { defaultIconControlStyles } from "../button/icon-button-variants";

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
      className={cn("gap-tight flex items-center", className)}
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
            "group relative flex items-center justify-center font-semibold text-muted outline-none transition-colors select-none",
            display === "icon"
              ? defaultIconControlStyles
              : defaultButtonControlStyles,
            "data-[hovered]:text-content-fg data-[selected]:text-content-fg",
            "data-[disabled]:cursor-not-allowed data-[disabled]:text-neutral-disabled-fg data-[disabled]:data-[hovered]:text-neutral-disabled-fg",
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
                  className="absolute inset-0 transform-gpu rounded-[inherit] bg-neutral backface-hidden group-data-[disabled]:bg-neutral-subtle will-change-transform"
                  layoutId={`pill-selection-${selectionId}`}
                  transition={{
                    duration: prefersReducedMotion ? 0 : motionDurations.travel,
                    ease: motionEasings.out,
                  }}
                />
              ) : null}
              <span className="gap-control-inset relative z-10 flex min-w-0 items-center justify-center whitespace-nowrap">
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
