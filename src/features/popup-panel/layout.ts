// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export const popupPanelMaxHeight = 150;
export const emptyPopupPanelHeight = 64;

const compactItemHeight = 24;
const itemGap = 4;
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
