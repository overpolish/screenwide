// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Stands in for an element while the message is translated: a character
 * from the private use area, which no translation writes. */
const MARK = "\uE000";

/**
 * A message with one element set in it, such as a keycap, split either side
 * of where the language puts it: `format` translates the message with the
 * mark given for the element's placeable. The whole sentence stays one
 * message, so a translation can move the element anywhere in it.
 */
export const splitAtPlaceable = (
  format: (mark: string) => string,
): [before: string, after: string] => {
  const [before, after = ""] = format(MARK)
    // Fluent isolates a placeable's direction; the element does that itself.
    .replace(`\u2068${MARK}\u2069`, MARK)
    .split(MARK);
  return [before, after];
};
