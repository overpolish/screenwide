// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export const popupPanelMaxHeight = 150;
export const emptyPopupPanelHeight = 64;

const compactItemHeight = 24;
/** Items touch, as a native menu's do; only headers stand off their items. */
const itemGap = 0;
const listboxPadding = 8;
const focusSafeInset = 4;
/** A section header is one line of `text-section` inside `py-control`. */
const sectionHeaderHeight = 22;

/** `sectionCount` counts the headers drawn above the items, so a grouped
 * panel opens at the height it will settle at rather than growing on show. */
export const initialPopupPanelHeight = (itemCount: number, sectionCount = 0) =>
  itemCount === 0
    ? emptyPopupPanelHeight
    : Math.min(
        itemCount * compactItemHeight +
          sectionCount * sectionHeaderHeight +
          Math.max(itemCount + sectionCount - 1, 0) * itemGap +
          listboxPadding +
          focusSafeInset,
        popupPanelMaxHeight,
      );

/** Tool panels are one fixed width, wide enough for a labelled slider row and
 * narrow enough to sit over a preview without covering it. */
export const toolPanelWidth = 320;

/** `--spacing-section` in logical px. A panel is placed and clamped in window
 * and screen coordinates, where the CSS token cannot be read. */
export const popupPanelSpacing = 12;

/** Windows tool panels use the `--spacing-window-inset` right edge token. */
export const toolPanelSpacing =
  typeof navigator !== "undefined" && /Windows/i.test(navigator.userAgent)
    ? 14
    : popupPanelSpacing;

/** What a tool panel opens at before its own content is measured. */
export const initialToolPanelHeight = 200;
