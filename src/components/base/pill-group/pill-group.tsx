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
  /** Geometry for the label itself, e.g. a truncation cap on a window title. */
  labelClassName?: string;
  /**
   * Fires on every press of the segment, after any selection change it
   * caused, the way a Screen segment opens its display chooser whether it
   * was just chosen or already was. `anchor` is the segment's bounding rect,
   * so a native menu can hang off it; `fromKeyboard` is true when the press
   * came without a pointer, so what opens can take focus rather than leaving
   * it on the segment. Arrow-key selection is not a press.
   */
  onPress?: (anchor: DOMRect, fromKeyboard: boolean) => void;
};

const selectedId = (selection: Selection) => {
  if (selection === "all") return null;
  const first = selection.values().next().value;
  return typeof first === "string" ? first : null;
};

/** The track insets the segments by `--spacing-tight` on every side, so a
 * segment's corner is the track's less that inset. The capture size is a
 * larger control and takes the next radius up. */
const trackRadius = {
  capture: "rounded-panel",
  default: "rounded-control",
};
const segmentRadius = {
  capture: "rounded-[calc(var(--radius-panel)-var(--spacing-tight))]",
  default: "rounded-[calc(var(--radius-control)-var(--spacing-tight))]",
};

// Text colour follows the knob's arrival; the hover fill on a ghost segment
// must be gone by the time the knob lands on it, or the two read as a double
// box, so it takes the feedback duration instead.
const segmentBase =
  "flex cursor-default items-center justify-center text-body text-content-fg outline-none transition-[color,background-color] select-none";

const knobBase =
  "absolute inset-0 transform-gpu rounded-[inherit] backface-hidden will-change-transform";
// Filled is the native segmented control: accent knob under white content.
// Ghost is the capture-toolbar form QuickTime uses: no track, and the
// selected segment sits on a neutral fill in the label colour.
const variantClassName = {
  filled: {
    // A bezeled control does not react to hover.
    hover: "",
    knob: `${knobBase} bg-primary-surface`,
    selectedText: "data-[selected]:text-primary-fg",
    track: "bg-fill",
  },
  ghost: {
    // Ghost segments are toolbar items, which do.
    hover:
      "data-[hovered]:not-data-[selected]:bg-fill-tertiary data-[pressed]:not-data-[selected]:bg-fill",
    knob: `${knobBase} bg-fill`,
    selectedText: "data-[selected]:text-content-fg",
    track: "bg-transparent",
  },
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
  size = "regular",
  variant = "filled",
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
  /** `large` is the 28px track AppKit uses for the large segmented control;
   * `capture` is the recording bar's icon-only control, whose segments are
   * 48 by 40 under 28px glyphs. */
  size?: "regular" | "large" | "capture";
  variant?: "filled" | "ghost";
}) {
  const selectionId = useId();
  const prefersReducedMotion = useReducedMotion();
  // Regular and large are the native control sizes; capture is the icon-only
  // control the recording bar carries.
  const segmentHeight =
    size === "capture" ? "h-10" : size === "large" ? "h-6" : "h-5";
  // An icon-only capture segment is a landscape rectangle, a toolbar item
  // rather than the glyph box the smaller sizes use.
  const iconWidth = size === "capture" ? "w-12" : "w-icon-large";
  const radii = size === "capture" ? "capture" : "default";
  const glyphClassName =
    size === "capture" ? "[&_svg]:size-icon-xl" : "[&_svg]:size-icon";
  const styles = variantClassName[variant];
  const transitionDuration = prefersReducedMotion
    ? "0s, 0s"
    : `${motionDurationCss("state")}, ${motionDurationCss("feedback")}`;
  const knobTransition = {
    duration: prefersReducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };
  const layoutId = `pill-selection-${selectionId}`;

  return (
    <ToggleButtonGroup
      aria-label={ariaLabel}
      // A macOS 26 segmented control, measured from a live capture: a track on
      // the fill, 24px tall with a 2px inset, and the selected segment filled
      // with the accent under white content. The track sizes from its
      // segments so larger geometry, such as the screen recording mode
      // picker, keeps the same construction.
      className={cn(
        "flex items-stretch gap-tight p-tight",
        trackRadius[radii],
        styles.track,
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
      {items.map((item) => {
        const isItemDisabled = disabledIds?.includes(item.id) ?? false;
        const label = item.labelClassName ? (
          <span className={item.labelClassName}>{item.label}</span>
        ) : (
          item.label
        );
        const contents = (
          <span className="relative z-10 flex min-w-0 items-center justify-center gap-control whitespace-nowrap">
            {display !== "label" ? item.icon : null}
            {display !== "icon" ? label : null}
          </span>
        );

        return (
          <ToggleButton
            aria-label={item.ariaLabel ?? item.label}
            className={cn(
              "group relative",
              segmentBase,
              segmentHeight,
              segmentRadius[radii],
              display === "icon" ? iconWidth : "px-control-inset",
              styles.selectedText,
              styles.hover,
              "data-[disabled]:text-content-fg-tertiary data-[disabled]:data-[selected]:text-content-fg-tertiary",
              glyphClassName,
              "[&_svg]:shrink-0 [&_svg]:transform-gpu",
              focusStyles,
              elementFocusVisible,
              itemClassName,
            )}
            id={item.id}
            isDisabled={isItemDisabled}
            key={item.id}
            // The toggle runs before this, so a listener sees the new
            // selection; `disallowEmptySelection` keeps the selected segment
            // put, so its press only opens the source list.
            onPress={(event) => {
              item.onPress?.(
                event.target.getBoundingClientRect(),
                ["keyboard", "virtual"].includes(event.pointerType),
              );
            }}
            style={{ transitionDuration }}
          >
            {({ isSelected }) => (
              <>
                {isSelected ? (
                  <motion.span
                    aria-hidden="true"
                    className={cn(
                      styles.knob,
                      "group-data-[disabled]:bg-fill-quaternary",
                    )}
                    layoutId={layoutId}
                    transition={knobTransition}
                  />
                ) : null}
                {contents}
              </>
            )}
          </ToggleButton>
        );
      })}
    </ToggleButtonGroup>
  );
}
