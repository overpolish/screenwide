// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { findInNotes, replaceMatches } from "../document/notes-find";

import type { SensitiveLookup } from "./use-sensitive";
import type { SensitiveFind } from "../../../bindings/SensitiveFind";
import type { SensitiveFindKind } from "../../../bindings/SensitiveFindKind";
import type { NotesDocument } from "../document/notes-document";

// What may be private in a paragraph, by kind, and what is done about it:
// its words redacted in one paragraph or all of them, each a change of its
// own to undo.

/** What redacted words read as in the notes. The notes file stays in
 * English for whatever reads it, as text written into exported files does. */
export const REDACTED = "REDACTED";

/** One kind of thing found in a paragraph, and where each is. */
export type SensitiveGroup = {
  finds: readonly SensitiveFind[];
  kind: SensitiveFindKind;
};

/** What a group of finds is, and how many, as its badge names it. */
export const sensitiveKindLabel = ({ finds, kind }: SensitiveGroup) =>
  t("notes-sensitive-kind", { count: finds.length, kind });

/** A paragraph's finds by kind, each kind where it is first found. */
export function groupByKind(finds: readonly SensitiveFind[]): SensitiveGroup[] {
  const groups = new Map<SensitiveFindKind, SensitiveFind[]>();
  for (const find of finds) {
    const group = groups.get(find.kind);
    if (group) group.push(find);
    else groups.set(find.kind, [find]);
  }
  return [...groups].map(([kind, found]) => ({ finds: found, kind }));
}

/** The notes with the words `finds` cover, all in turn `turn`, redacted. */
export function redactInTurn(
  document: NotesDocument,
  turn: number,
  finds: readonly Pick<SensitiveFind, "end" | "start">[],
): NotesDocument {
  return replaceMatches(
    document,
    finds.map(({ end, start }) => ({ from: start, to: end, turn })),
    REDACTED,
  );
}

/** The notes with `words` redacted wherever they are said, in any case. */
export function redactThroughout(
  document: NotesDocument,
  words: string,
): NotesDocument {
  return replaceMatches(document, findInNotes(document, words), REDACTED);
}

/** The notes with `words` marked as not private, so they are no longer
 * pointed out anywhere in them. */
export function markNotPrivate(
  document: NotesDocument,
  words: readonly string[],
): NotesDocument {
  const marked = new Set(document.notPrivate);
  for (const word of words) marked.add(word.toLocaleLowerCase());
  return { ...document, notPrivate: [...marked].sort() };
}

/** `lookup` without the finds whose words the notes mark as not private,
 * nor those of words already redacted, which the model can take for a
 * company. */
export function withoutNotPrivate(
  lookup: SensitiveLookup,
  notPrivate: readonly string[] = [],
): SensitiveLookup {
  const marked = new Set([...notPrivate, REDACTED.toLocaleLowerCase()]);
  return (turn, text) =>
    lookup(turn, text).filter(
      ({ end, start }) =>
        !marked.has(text.slice(start, end).toLocaleLowerCase()),
    );
}
