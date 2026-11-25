// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export const popupPanelMaxHeight = 250;
export const emptyPopupPanelHeight = 64;
/** A list found by typing opens at one height whatever it holds, so the
 * window keeps still as typing narrows the rows: about a dozen of them under
 * the search field. */
export const searchablePopupPanelHeight = 340;
/** The widest a list grows to fit a row it would otherwise cut short; past
 * this a row's label is truncated, as a native menu's is. */
export const popupPanelMaxWidth = 360;

const compactItemHeight = 24;
/** Items touch, as a native menu's do; only headers stand off their items. */
const itemGap = 0;
const listboxPadding = 8;
const focusSafeInset = 4;
/** A section header is one 14 px line of `text-caption` under
 * `pt-separate` and over `pb-inline` (`popup-panel-list.tsx`). */
const sectionHeaderHeight = 32;
/** The fewest rows a menu's scrolling section shows, however much of the
 * window the part above it takes. */
const scrollingSectionLeastRows = 3;

/** How tall a menu's scrolling section is drawn under a part `fixedHeight`
 * tall that always shows whole: all `naturalHeight` of it where that fits
 * under the list ceiling, else the room left there, but never less than a
 * few rows, so the menu grows past the ceiling rather than hide it. */
export const scrollingSectionHeight = (
  fixedHeight: number,
  naturalHeight: number,
) =>
  Math.min(
    naturalHeight,
    Math.max(
      scrollingSectionLeastRows * compactItemHeight,
      popupPanelMaxHeight - fixedHeight,
    ),
  );

/** `sectionCount` counts the headers drawn above the items, so a grouped
 * panel opens at the height it will settle at rather than growing on show.
 * `showsAllItems` lifts the list ceiling for a list that is never scrolled.
 * `scrollingCount` of the items, the last, scroll on their own under the
 * rest (`scrollingSectionHeight`). */
export const initialPopupPanelHeight = (
  itemCount: number,
  sectionCount = 0,
  {
    scrollingCount = 0,
    showsAllItems = false,
  }: { scrollingCount?: number; showsAllItems?: boolean } = {},
) => {
  if (itemCount === 0) return emptyPopupPanelHeight;
  const rows = itemCount - scrollingCount;
  const height =
    rows * compactItemHeight +
    sectionCount * sectionHeaderHeight +
    Math.max(rows + sectionCount - 1, 0) * itemGap +
    listboxPadding +
    focusSafeInset;
  if (scrollingCount > 0)
    return (
      height +
      scrollingSectionHeight(height, scrollingCount * compactItemHeight)
    );
  return showsAllItems ? height : Math.min(height, popupPanelMaxHeight);
};
