// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useReducedMotion } from "motion/react";
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
} from "../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { IslandFill } from "../island/island-fill";
import { useSkin } from "../skin/skin";

// A segment shows what its item gives: its icon, its label, or both, so a
// group can mix them. One with no label shows its icon alone and is named by
// `ariaLabel`, which a labelled segment may also give to say more than its
// label.
export type PillGroupItem = {
  id: string;
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
} & (
  | { label: string; ariaLabel?: string; icon?: ReactNode }
  | { ariaLabel: string; icon: ReactNode; label?: never }
);

const selectedId = (selection: Selection) => {
  if (selection === "all") return null;
  const first = selection.values().next().value;
  return typeof first === "string" ? first : null;
};

// A segmented control: a track on the fill, a control's height so it lines
// up with buttons and fields, with the chosen segment filled in the primary
// colour on a knob that slides between segments, so what is chosen can be
// read at a glance across a panel of them. The track insets its segments by
// 2px, a value only a segmented track needs, so a segment's corner is the
// track's less that inset and the two run parallel.
const trackClassName =
  "flex h-control-height items-stretch gap-0.5 rounded-control bg-fill p-0.5";

// Unselected segments are a step back and come up under the pointer; the
// chosen one takes the on-primary colour over its knob. The text colour
// follows the knob's arrival.
const segmentClassName = [
  "group relative flex cursor-default items-center justify-center rounded-[calc(var(--radius-control)-0.125rem)] text-label text-fg-secondary outline-none transition-colors select-none",
  "data-[hovered]:not-data-[selected]:text-fg data-[selected]:text-on-primary",
  "data-[disabled]:opacity-40",
  "[&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
  focusStyles,
  elementFocusVisible,
];

const knobClassName =
  "absolute inset-0 transform-gpu rounded-[inherit] bg-primary primary-raised backface-hidden will-change-transform";

// On the island the segments are its circles, with no track: each has the
// island's fill, which is its hover and its choice, so a circle chosen under
// the pointer keeps the fill it already showed, one chosen from the keyboard
// grows its own, and the one left behind shrinks away. A segment with a
// label is a pill the circle's height, as wide as its label.
const islandSegmentClassName = [
  "group relative isolate flex h-island-control min-w-island-control shrink-0 cursor-default items-center justify-center rounded-full text-label outline-none select-none",
  "data-[disabled]:opacity-40",
  "[&_svg]:size-icon [&_svg]:shrink-0 [&_svg]:transform-gpu",
  focusStyles,
  elementFocusVisible,
];

/** One segment chosen, the way a picker's is; or any of them on together,
 * the way a set of independent options is. */
type PillGroupSelection =
  | {
      onSelectionChange: (id: string) => void;
      selected: string;
      selectionMode?: "single";
    }
  | {
      onSelectionChange: (ids: string[]) => void;
      selected: string[];
      selectionMode: "multiple";
    };

export function PillGroup({
  "aria-label": ariaLabel,
  className,
  disabledIds,
  isDisabled,
  itemClassName,
  items,
  ...selection
}: PillGroupSelection & {
  "aria-label": string;
  items: PillGroupItem[];
  className?: string;
  /** Items that stay visible but cannot be picked. */
  disabledIds?: string[];
  isDisabled?: boolean;
  /** Applies shared geometry overrides to every item. */
  itemClassName?: string;
}) {
  const isIsland = useSkin() === "island";
  const isMultiple = selection.selectionMode === "multiple";
  const selectionId = useId();
  const prefersReducedMotion = useReducedMotion();
  const transitionDuration = prefersReducedMotion
    ? "0s"
    : motionDurationCss("state");
  const knobTransition = {
    duration: prefersReducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };
  // One knob slides from segment to segment; with several on, each keeps
  // its own where it is, growing in as its segment turns on and shrinking
  // away as it turns off, or only fading where motion is reduced.
  const layoutId = isMultiple ? undefined : `pill-selection-${selectionId}`;
  const knobOff = {
    opacity: 0,
    scale: prefersReducedMotion ? 1 : 0.85,
  };
  const knobPresence = isMultiple
    ? {
        animate: { opacity: 1, scale: 1 },
        exit: knobOff,
        initial: knobOff,
        transition: {
          duration: motionDurations.state,
          ease: motionEasings.out,
        },
      }
    : { transition: knobTransition };

  return (
    <ToggleButtonGroup
      aria-label={ariaLabel}
      className={cn(isIsland ? "flex gap-1" : trackClassName, className)}
      disallowEmptySelection={!isMultiple}
      isDisabled={isDisabled}
      onSelectionChange={(keys) => {
        if (selection.selectionMode === "multiple") {
          selection.onSelectionChange([...keys].map(String));
          return;
        }
        const id = selectedId(keys);
        if (id !== null) selection.onSelectionChange(id);
      }}
      selectedKeys={
        new Set(
          selection.selectionMode === "multiple"
            ? selection.selected
            : [selection.selected],
        )
      }
      selectionMode={isMultiple ? "multiple" : "single"}
    >
      {items.map((item) => {
        const isItemDisabled = disabledIds?.includes(item.id) ?? false;
        const label =
          item.label !== undefined && item.labelClassName ? (
            <span className={item.labelClassName}>{item.label}</span>
          ) : (
            item.label
          );

        return (
          <ToggleButton
            aria-label={item.ariaLabel ?? item.label}
            className={cn(
              isIsland
                ? [
                    islandSegmentClassName,
                    item.label === undefined ? "w-island-control" : "px-inset",
                  ]
                : [
                    segmentClassName,
                    // A segment showing only its icon is a square the
                    // track's height.
                    item.label === undefined ? "aspect-square" : "px-inset",
                  ],
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
            {({ isHovered, isSelected }) =>
              isIsland ? (
                <>
                  <IslandFill isChosen={isSelected} isHovered={isHovered} />
                  {item.icon ?? label}
                </>
              ) : (
                <>
                  <AnimatePresence initial={false}>
                    {isSelected ? (
                      <motion.span
                        aria-hidden="true"
                        className={knobClassName}
                        key="knob"
                        layoutId={layoutId}
                        {...knobPresence}
                      />
                    ) : null}
                  </AnimatePresence>
                  {/* Above the knob in every segment, not just its own:
                      while the knob slides it is drawn from the segment it
                      is moving to, which comes later in the page than the
                      ones it crosses, and would otherwise cover their
                      text. */}
                  <span className="relative z-10 flex min-w-0 items-center justify-center gap-inline whitespace-nowrap">
                    {item.icon}
                    {label}
                  </span>
                </>
              )
            }
          </ToggleButton>
        );
      })}
    </ToggleButtonGroup>
  );
}
