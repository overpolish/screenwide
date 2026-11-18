// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Notes as the editor works on them: their title and each turn, who said it
// and what, read from the Markdown file and written back to it. The file is
// what assistants read, so it stays plain: a `# Title` heading, then a
// paragraph a turn, each opening on its speaker in bold ("**Me:** ...").
// A paragraph with no speaker, such as one typed into the file by hand, is
// kept as it is, a turn of no one's.

type NotesTurn = {
  /** Who said it, as the notes name them; null for a paragraph of no one's. */
  speaker: string | null;
  text: string;
};

export type NotesDocument = {
  /** The heading's text; null when the file has none. */
  title: string | null;
  turns: readonly NotesTurn[];
};

/** A run of turns one speaker said one after another, shown under their
 * name once, as `[start, end)` turn indices. */
export type NotesRun = { end: number; speaker: string | null; start: number };

const HEADING = /^# (.*)$/;
const TURN = /^\*\*([^*\n]+?):\*\*[ \t]*([\s\S]*)$/;

export function parseNotes(markdown: string): NotesDocument {
  const paragraphs = markdown
    .split(/\n[ \t]*\n/)
    .map((paragraph) => paragraph.trim())
    .filter((paragraph) => paragraph.length > 0);
  const heading = paragraphs.length > 0 ? HEADING.exec(paragraphs[0]) : null;
  const body = heading ? paragraphs.slice(1) : paragraphs;
  return {
    title: heading ? heading[1].trim() : null,
    turns: body.map((paragraph) => {
      const turn = TURN.exec(paragraph);
      return turn
        ? { speaker: turn[1].trim(), text: turn[2] }
        : { speaker: null, text: paragraph };
    }),
  };
}

/** A turn's text as one paragraph: a blank line typed into it would start
 * another, so lines it holds are kept but never left blank between. */
const asParagraph = (text: string) => text.replace(/\n(?:[ \t]*\n)+/g, "\n");

/** The notes as their Markdown file. A turn whose words were all cleared is
 * left out, so clearing a turn removes it once the notes are saved. */
export function writeNotes({ title, turns }: NotesDocument): string {
  const paragraphs = turns
    .filter(({ text }) => text.trim() !== "")
    .map(({ speaker, text }) => {
      const paragraph = asParagraph(text);
      return speaker === null
        ? paragraph
        : `**${speaker}:** ${paragraph}`.trimEnd();
    });
  return `${[...(title === null ? [] : [`# ${title}`]), ...paragraphs].join("\n\n")}\n`;
}

/** Each speaker, in the order they first speak. */
export function speakersOf({ turns }: NotesDocument): string[] {
  const speakers: string[] = [];
  for (const { speaker } of turns)
    if (speaker !== null && !speakers.includes(speaker)) speakers.push(speaker);
  return speakers;
}

/** Each speaker, in the order they first speak, with their share of the
 * words said, from 0 to 1. */
export function speakerShares(document: NotesDocument) {
  const words = new Map<string, number>();
  let total = 0;
  for (const { speaker, text } of document.turns) {
    if (speaker === null) continue;
    const said = text.split(/\s+/).filter(Boolean).length;
    words.set(speaker, (words.get(speaker) ?? 0) + said);
    total += said;
  }
  return speakersOf(document).map((name) => ({
    name,
    share: total === 0 ? 0 : (words.get(name) ?? 0) / total,
  }));
}

/** The turns as runs of one speaker each. */
export function runsOf({ turns }: NotesDocument): NotesRun[] {
  const runs: NotesRun[] = [];
  turns.forEach(({ speaker }, index) => {
    const last = runs.length > 0 ? runs[runs.length - 1] : null;
    if (last && last.speaker === speaker && speaker !== null)
      last.end = index + 1;
    else runs.push({ end: index + 1, speaker, start: index });
  });
  return runs;
}

/** A name as a speaker can be written: without the asterisks and colons
 * that would end their bold label early, on one line. */
export const speakerName = (name: string) =>
  name
    .replace(/[*:\n]/g, "")
    .replace(/\s+/g, " ")
    .trim();

/** Every turn of `from`'s given to `to`. */
export function renameSpeaker(
  document: NotesDocument,
  from: string,
  to: string,
): NotesDocument {
  return {
    ...document,
    turns: document.turns.map((turn) =>
      turn.speaker === from ? { ...turn, speaker: to } : turn,
    ),
  };
}

/** The run of turns `run` covers given to `speaker`, and no other. */
export function assignRun(
  document: NotesDocument,
  run: NotesRun,
  speaker: string,
): NotesDocument {
  return {
    ...document,
    turns: document.turns.map((turn, index) =>
      index >= run.start && index < run.end ? { ...turn, speaker } : turn,
    ),
  };
}

export function editTurn(
  document: NotesDocument,
  index: number,
  text: string,
): NotesDocument {
  return {
    ...document,
    turns: document.turns.map((turn, at) =>
      at === index ? { ...turn, text } : turn,
    ),
  };
}

/** The notes without the turns `[start, end)`. */
export function removeTurns(
  document: NotesDocument,
  start: number,
  end: number,
): NotesDocument {
  return {
    ...document,
    turns: document.turns.filter((_, index) => index < start || index >= end),
  };
}
