// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export const standaloneListboxMaxHeight = 150;
export const emptyStandaloneListboxHeight = 64;

const compactItemHeight = 24;
const itemGap = 4;
const listboxPadding = 8;
const focusSafeInset = 4;

export const initialStandaloneListboxHeight = (itemCount: number) =>
  itemCount === 0
    ? emptyStandaloneListboxHeight
    : Math.min(
        itemCount * compactItemHeight +
          Math.max(itemCount - 1, 0) * itemGap +
          listboxPadding +
          focusSafeInset,
        standaloneListboxMaxHeight,
      );
