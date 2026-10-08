// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";
import { Button } from "react-aria-components";

import { Switch } from "../../../../components/base/switch/switch";
import { NativeTooltipTrigger } from "../../../../components/shared/native-tooltip/native-tooltip-trigger";
import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";

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

/**
 * The label column of one timeline row: what the row is, and whether it is
 * carried into the export.
 *
 * Every row kind uses this - the two video tracks, each audio track and the
 * shortcuts lane - so they share the band's row height and the fill that
 * marks the selected track. The shortcuts lane is the row with nothing to
 * include or exclude, so it comes without a switch. A `note` says something
 * about how the track is exported: an icon beside the name, spelled out in
 * the row's tooltip and accessible name.
 */
export function TimelineTrackHeader({
  icon,
  inclusion,
  isSelected = false,
  label,
  note,
  onSelect,
}: {
  icon: ReactNode;
  label: string;
  inclusion?: TimelineTrackInclusion;
  isSelected?: boolean;
  note?: { icon: ReactNode; label: string };
  onSelect?: () => void;
}) {
  const selectButton = (
    <Button
      aria-label={
        note
          ? t("editor-timeline-track-note", { note: note.label, track: label })
          : label
      }
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
      {note ? (
        <span
          aria-hidden
          className="flex shrink-0 items-center text-content-fg-secondary [&_svg]:size-icon"
        >
          {note.icon}
        </span>
      ) : null}
    </Button>
  );

  const row = (
    <div
      className={cn(
        "flex h-control-height w-timeline-gutter shrink-0 items-center",
        // A lane of stacked rows can be taller than the band shows. Pinned to
        // both edges, the label stays centred while the lane fits and holds
        // to whichever edge cuts it off otherwise, never leaving its lane.
        // Rows wrapped in a tooltip are single-row tracks, which never need it.
        "sticky top-0 bottom-0",
        "rounded-control transition-[background-color,opacity]",
        // Selection is the accent tint the app marks a chosen row with; the
        // label keeps its own colour so the switch beside it stays readable.
        isSelected && "bg-primary/15",
        inclusion && !inclusion.isIncluded && "opacity-50",
      )}
    >
      {selectButton}
      {inclusion ? (
        // The switch acts on the track, not on the row: the bubbling press is
        // stopped here so no ancestor treats it as a press on the row.
        <span
          className="flex shrink-0 items-center pr-control-inset"
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
        >
          <Switch
            aria-label={
              inclusion.isRequired
                ? t("editor-timeline-track-required", { track: label })
                : inclusion.isIncluded
                  ? t("editor-timeline-track-exclude", { track: label })
                  : t("editor-timeline-track-include", { track: label })
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
  // attaches the trigger to. The last track says why it cannot be switched
  // off here rather than on the switch: a disabled control receives no
  // pointer events, so a tooltip hung on it would never open.
  const tooltip = inclusion?.isRequired
    ? t("editor-timeline-track-required-tooltip")
    : note?.label;
  return tooltip ? (
    <NativeTooltipTrigger tooltip={tooltip}>{row}</NativeTooltipTrigger>
  ) : (
    row
  );
}
