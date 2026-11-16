// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The name the notes give the note taker, as their microphone was heard. */
export const NOTE_TAKER = "Me";

/** What a speaker's avatar shows: a numbered speaker's number, the note
 * taker's own name, otherwise the first letters of the first two words of
 * the name. */
export function speakerInitials(name: string) {
  const numbered = /^Speaker (\d+)$/.exec(name);
  if (numbered) return numbered[1];
  if (name === NOTE_TAKER) return name;
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => Array.from(word)[0].toLocaleUpperCase())
    .join("");
}
