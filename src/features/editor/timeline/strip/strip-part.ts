// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What runs across a jump is drawn as parts, one each side, each with an
// id of its own made from the item's (`strip-split.ts`).

/** A part's id: the first keeps the item's, so what is chosen stays chosen;
 * each after it takes its own. */
export const partId = (id: string, index: number) =>
  index === 0 ? id : `${id}:${String(index)}`;

/** What a part was split from: its id before the part's number. */
export const baseOf = (id: string) => id.split(":")[0];
