// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Stand in for elements while a message is translated: characters from the
 * private use area, which no translation writes. */
const marks = (count: number) =>
  Array.from({ length: count }, (_, index) =>
    String.fromCharCode(0xe000 + index),
  );

/**
 * A message with elements set in it, such as keycaps, split round where the
 * language puts them: `format` translates the message with a mark given for
 * each element's placeable. Text comes back as strings and each element as
 * its index among the marks, in the order the language reads them, so a
 * translation can move or reorder the elements anywhere in the sentence.
 */
export const splitAtPlaceables = (
  count: number,
  format: (marks: string[]) => string,
): (number | string)[] => {
  const given = marks(count);
  let text = format(given);
  // Fluent isolates a placeable's direction; the element does that itself.
  for (const mark of given) text = text.replace(`\u2068${mark}\u2069`, mark);
  return text
    .split(new RegExp(`([${given.join("")}])`, "u"))
    .filter((part) => part !== "")
    .map((part) => {
      const index = given.indexOf(part);
      return index === -1 ? part : index;
    });
};

/** `splitAtPlaceables` for a message with one element set in it, as the
 * text either side of it. */
export const splitAtPlaceable = (
  format: (mark: string) => string,
): [before: string, after: string] => {
  // One mark, so joining them is that mark.
  const parts = splitAtPlaceables(1, (given) => format(given.join("")));
  const at = parts.indexOf(0);
  const text = (from: number, to?: number) =>
    parts
      .slice(from, to)
      .filter((part) => typeof part === "string")
      .join("");
  return at === -1 ? [text(0), ""] : [text(0, at), text(at + 1)];
};
