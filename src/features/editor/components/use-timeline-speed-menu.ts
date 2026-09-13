// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import { pointerAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";

const TIMELINE_PLAYBACK_RATES = [0.5, 0.75, 1, 1.25, 1.5, 2] as const;

/** Every row is a rate and its tick, so the list stays narrow. */
const SPEED_MENU_WIDTH = 120;

const items: PopupPanelItem[] = TIMELINE_PLAYBACK_RATES.map((rate) => ({
  id: rate.toString(),
  label: `${rate.toString()}×`,
  // The rates head a section of their own, so the menu says what they are.
  section: "Speed",
}));

/**
 * The playback-speed menu a right click opens on a timeline segment or on a
 * selected range.
 *
 * It is a select rather than a menu: the rate in force is ticked, so a right
 * click reads the current speed as well as changing it. A range spanning
 * segments of differing speeds has no one rate, and ticks nothing. The menu hangs off
 * the pointer, and the panel window clamps itself to the monitor, so a click
 * near the bottom of the screen needs no clamping here.
 */
export function useTimelineSpeedMenu(
  scope: "range" | "segment",
  onChange: (playbackRate: number, context: string) => void,
) {
  const openMenu = usePopupMenu({
    idPrefix: `timeline-speed:${scope}:`,
    label: `${scope === "range" ? "Range" : "Segment"} playback speed`,
    mode: "select",
    onSelect: (itemId, context) => {
      onChange(Number(itemId), context);
    },
    width: SPEED_MENU_WIDTH,
  });

  return (
    point: { x: number; y: number },
    playbackRate: number | undefined,
    context?: string,
  ) =>
    openMenu({
      anchor: pointerAnchor(point.x, point.y),
      context,
      items,
      selectedIds: playbackRate === undefined ? [] : [playbackRate.toString()],
    });
}
