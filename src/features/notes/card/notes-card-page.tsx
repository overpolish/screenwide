// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { parseNotes, runsOf } from "../document/notes-document";
import { SpeakerAvatar } from "../speakers/speaker-avatar";

// A notes project's card shows the start of what was said, as the notes
// page does, small: each speaker's avatar beside their words, running off
// the card's foot under a fade. It stands where a recording's still would,
// so opening the notes grows the page out of it.

/** How many runs of the opening are drawn; more never show on a card. */
const RUNS = 4;

export function NotesCardPage({ markdown }: { markdown: string }) {
  const notes = parseNotes(markdown);
  return (
    <span
      aria-hidden
      className="absolute inset-0 flex flex-col gap-inline overflow-hidden px-inset py-inset-sm text-start mask-b-from-60% mask-b-to-95%"
    >
      {runsOf(notes)
        .slice(0, RUNS)
        .map((run) => (
          <span className="flex gap-inline" key={run.start}>
            {run.speaker === null ? (
              <span className="size-5 shrink-0" />
            ) : (
              <SpeakerAvatar name={run.speaker} size="card" />
            )}
            <span
              className="line-clamp-3 min-w-0 pt-0.5 text-caption text-fg-secondary"
              dir="auto"
            >
              {notes.turns
                .slice(run.start, run.end)
                .map(({ text }) => text)
                .join(" ")}
            </span>
          </span>
        ))}
    </span>
  );
}
