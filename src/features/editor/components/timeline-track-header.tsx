// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentPropsWithoutRef, ReactNode } from "react";
import { Button } from "react-aria-components";

import { Switch } from "../../../components/base/switch/switch";
import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

/** Whether a track is carried into the export, and whether that can change. */
export type TimelineTrackInclusion = {
  isIncluded: boolean;
  /**
   * The last track still in the export. It cannot be switched off: an export
   * has to carry something.
   */
  isRequired: boolean;
  onChange: (isIncluded: boolean) => void;
};

/** Pointer handling a reorderable row hangs off its header. */
export type TimelineTrackDragProps = Pick<
  ComponentPropsWithoutRef<"div">,
  "onPointerCancel" | "onPointerDownCapture" | "onPointerMove" | "onPointerUp"
>;

/**
 * Marks the switch column for the row's own pointer handling to step around.
 *
 * The switch cannot be recognised by its role: React Aria renders the control
 * as a label around a visually hidden input, and `role="switch"` sits on that
 * input, below the element a press actually lands on. A row that captured the
 * pointer for a press on the switch would take the click with it - a captured
 * pointer sends its compatibility click to the capturing element - and the
 * label would never activate its input.
 */
export const TIMELINE_TRACK_SWITCH_SELECTOR = "[data-timeline-track-switch]";

/**
 * Said on the row rather than on the switch: a disabled control receives no
 * pointer events, so a tooltip hung on it would never open.
 */
const REQUIRED_TOOLTIP = "The export has to keep at least one track";

/**
 * The label column of one timeline row: what the row is, and whether it is
 * carried into the export.
 *
 * Every row kind uses this - the two video tracks, each audio track and the
 * shortcuts lane - so they share the band's row height and the fill that
 * marks the selected track. The shortcuts lane is the row with nothing to
 * include or exclude, so it comes without a switch.
 */
export function TimelineTrackHeader({
  dragProps,
  icon,
  inclusion,
  isDragging = false,
  isSelected = false,
  label,
  onSelect,
}: {
  icon: ReactNode;
  label: string;
  dragProps?: TimelineTrackDragProps;
  inclusion?: TimelineTrackInclusion;
  /** True only while this row is being carried to a new place in the order. */
  isDragging?: boolean;
  isSelected?: boolean;
  onSelect?: () => void;
}) {
  const selectButton = (
    <Button
      aria-label={label}
      className={cn(
        "flex h-full min-w-0 grow cursor-[inherit] items-center gap-control-inset",
        "rounded-control px-control-inset text-left select-none",
        focusStyles,
        elementFocusVisible,
      )}
      excludeFromTabOrder={!onSelect}
      onPress={() => {
        onSelect?.();
      }}
    >
      <span className="flex shrink-0 items-center text-content-fg-secondary [&_svg]:size-icon">
        {icon}
      </span>
      <span className="min-w-0 grow truncate text-subheadline text-content-fg">
        {label}
      </span>
    </Button>
  );

  const row = (
    <div
      className={cn(
        "flex h-control-height w-timeline-gutter shrink-0 items-center",
        "rounded-control transition-[background-color,opacity]",
        // Selection is the accent tint the app marks a chosen row with; the
        // label keeps its own colour so the switch beside it stays readable.
        isSelected && "bg-primary/15",
        inclusion && !inclusion.isIncluded && "opacity-50",
        // At rest the row is an ordinary target: the grabbing cursor belongs
        // to a reorder in progress, not to the possibility of one.
        isDragging && "cursor-grabbing",
      )}
      {...dragProps}
    >
      {selectButton}
      {inclusion ? (
        // The switch acts on the track, not on the row: a press on it must
        // neither select the row nor start a reorder. The row's drag handling
        // reads the marker attribute; the bubbling press is stopped here so
        // no ancestor treats it as a press on the row.
        <span
          className="flex shrink-0 items-center pr-control-inset"
          data-timeline-track-switch=""
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
        >
          <Switch
            aria-label={
              inclusion.isRequired
                ? `${label} must remain included`
                : `${inclusion.isIncluded ? "Exclude" : "Include"} ${label}`
            }
            isDisabled={inclusion.isRequired}
            isSelected={inclusion.isIncluded}
            onChange={inclusion.onChange}
          />
        </span>
      ) : null}
    </div>
  );

  // The tooltip hangs on the whole row, whose button is what React Aria
  // attaches the trigger to.
  return inclusion?.isRequired ? (
    <NativeTooltipTrigger tooltip={REQUIRED_TOOLTIP}>
      {row}
    </NativeTooltipTrigger>
  ) : (
    row
  );
}
