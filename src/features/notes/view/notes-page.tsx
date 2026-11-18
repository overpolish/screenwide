// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Reveal } from "../../../composites/reveal/reveal";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { boundsAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import {
  assignRun,
  editTurn,
  type NotesDocument,
  type NotesRun,
  removeTurns,
  renameSpeaker,
  runsOf,
  speakerName,
  speakerShares,
} from "../document/notes-document";
import { type FoundRange } from "../find/found-pieces";
import { SpeakersStrip } from "../speakers/speakers-strip";

import { NotesRunSection } from "./notes-run";
import { useSpeakerFilter } from "./use-speaker-filter";

import type { NotesMatch } from "../document/notes-find";

const MENU_PREFIX = "notes-speaker:";
const MENU_WIDTH = 200;

// The notes as a page: who was on the call across its head, then each run
// of one speaker's turns. A run is removed from beside its speaker's name;
// a paragraph whose words are all cleared goes when the notes are saved.
// What a search found is tinted where it is. Choosing speakers by their
// share shows only what they said: each stretch of everyone else closes up
// into a line that opens it again, and the match a search goes to shows
// wherever it falls. Changes come back whole, with the turn typed into
// named so a run of typing undoes as one step.

export function NotesPage({
  current,
  document,
  isReadOnly,
  matches,
  onChange,
  onHearVoice,
  playingVoice,
  voices,
}: {
  /** The match gone to, by its place in `matches`; -1 when none is. */
  current: number;
  document: NotesDocument;
  isReadOnly: boolean;
  /** What a search found. */
  matches: readonly NotesMatch[];
  /** The notes changed; `typing` names the turn typed into, if that was
   * the change. */
  onChange: (document: NotesDocument, typing?: string) => void;
  onHearVoice: (number: number) => void;
  playingVoice: number | null;
  voices: readonly number[];
}) {
  const runs = runsOf(document);
  const speakers = speakerShares(document);
  const filter = useSpeakerFilter(speakers.map(({ name }) => name));
  const found = new Map<number, FoundRange[]>();
  matches.forEach(({ from, to, turn }, index) => {
    found.set(turn, [
      ...(found.get(turn) ?? []),
      { from, isCurrent: index === current, to },
    ]);
  });
  const currentTurn = current >= 0 ? matches[current].turn : -1;
  const isShown = (run: NotesRun) =>
    filter.chosen.length === 0 ||
    (run.speaker !== null && filter.chosen.includes(run.speaker)) ||
    (currentTurn >= run.start && currentTurn < run.end);
  // Each stretch of runs not shown, by the index of its first run: where
  // it starts, so it can be opened again, and how many paragraphs it holds.
  const stretches = new Map<number, { count: number; start: number }>();
  let stretch: { count: number; start: number } | null = null;
  runs.forEach((run, index) => {
    if (isShown(run)) {
      stretch = null;
      return;
    }
    if (!stretch) {
      stretch = { count: 0, start: run.start };
      stretches.set(index, stretch);
    }
    stretch.count += run.end - run.start;
  });
  const isOpened = (start: number) => filter.opened.includes(start);
  let openedStart = -1;

  const rename = (from: string, name: string) => {
    const to = speakerName(name);
    if (!to || to === from) return;
    filter.rename(from, to);
    onChange(renameSpeaker(document, from, to));
  };
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("notes-view-said-by-menu"),
    mode: "select",
    onSelect: (speaker, context) => {
      const run = runs[Number(context)] as NotesRun | undefined;
      if (run) onChange(assignRun(document, run, speaker));
    },
    width: MENU_WIDTH,
  });
  return (
    // The space above each run and line opens and closes inside its reveal,
    // with it, so the page does not jump by a gap as one comes or goes.
    <div className="mx-auto flex w-full max-w-2xl flex-col pt-separate">
      {speakers.length > 0 ? (
        <SpeakersStrip
          chosen={filter.chosen}
          isReadOnly={isReadOnly}
          onChoose={filter.toggle}
          onHearVoice={onHearVoice}
          onRename={rename}
          playingVoice={playingVoice}
          speakers={speakers}
          voices={voices}
        />
      ) : null}
      {runs.flatMap((run, index) => {
        const hidden = stretches.get(index);
        if (hidden) openedStart = isOpened(hidden.start) ? hidden.start : -1;
        const isVisible = isShown(run) || openedStart !== -1;
        // Every run and the line that could stand before it keep their
        // place, so each opens and closes as the speakers chosen change.
        return [
          <Reveal
            className="flex flex-col pt-group"
            key={`hidden-${String(run.start)}`}
          >
            {hidden && !isOpened(hidden.start) ? (
              // Hangs its padding into the margin, so its words start where
              // the paragraphs' do, and reads a step back from them.
              <Button
                className="-ms-inset self-start text-fg-secondary data-[hovered]:text-fg"
                onPress={() => {
                  filter.open(hidden.start);
                }}
                variant="ghost"
              >
                {t("notes-view-hidden", { count: hidden.count })}
              </Button>
            ) : null}
          </Reveal>,
          <Reveal className="pt-group" key={run.start}>
            {isVisible ? (
              <NotesRunSection
                document={document}
                found={found}
                isReadOnly={isReadOnly}
                onChooseSpeaker={(bounds) => {
                  void openMenu({
                    anchor: boundsAnchor(bounds),
                    context: String(index),
                    items: speakers.map(({ name }) => ({
                      id: name,
                      label: name,
                      section: t("notes-view-said-by-menu"),
                    })),
                    selectedIds: [run.speaker ?? ""],
                  });
                }}
                onEditTurn={(turn, text) => {
                  onChange(editTurn(document, turn, text), String(turn));
                }}
                onRemove={() => {
                  onChange(removeTurns(document, run.start, run.end));
                }}
                onRename={(name) => {
                  if (run.speaker !== null) rename(run.speaker, name);
                }}
                run={run}
              />
            ) : null}
          </Reveal>,
        ];
      })}
    </div>
  );
}
