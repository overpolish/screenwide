// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { GlidePickerScreen } from "./glide-picker-arrange";

export type GlidePickerState = {
  /** Whether a pick is under way; it ends with the window sent. */
  isShown: boolean;
  /** The screen the window is on, by index. */
  origin: number;
  /** The screen the window will go to, by index. */
  selected: number;
};

export type GlidePickerProps = GlidePickerState & {
  /** The glided app's icon, once it resolves. */
  iconSrc: null | string;
  kind: "displays" | "spaces";
  /** Counts the rests completed; each one plays the ready breath. */
  pulse: number;
  screens: GlidePickerScreen[];
};

/** A screen in the picker, in px: the gap keeps the destination clear of its
 * edge and its corner concentric with the screen's. 16:10, as a screen is. */
export const SLOT = { gap: 3, height: 30, radius: 8, width: 48 };

/** The destination on a screen. */
export const TILE = {
  height: SLOT.height - SLOT.gap * 2,
  radius: SLOT.radius - SLOT.gap,
  width: SLOT.width - SLOT.gap * 2,
};

/** The app's icon on its destination. */
export const PICKER_ICON = 18;

/** Between a card's edge and the screen it holds. */
const CARD_INSET = 5;

/** A card: one screen in its own island. The native cards are this size
 * (`picker_cards/arrange.rs`). */
export const CARD = {
  height: SLOT.height + CARD_INSET * 2,
  radius: SLOT.radius + CARD_INSET,
  width: SLOT.width + CARD_INSET * 2,
};

/** Between neighbouring cards, as the native layout leaves
 * (`core/monitors.rs`). */
export const CARD_GAP = 8;

/** Names where the window is going, for assistive technology. `index` counts
 * from 0. */
export const describePick = (
  kind: GlidePickerProps["kind"],
  index: number,
  count: number,
) => {
  const place = { count, index: index + 1 };
  return kind === "spaces"
    ? t("glide-picker-space", place)
    : t("glide-picker-display", place);
};
