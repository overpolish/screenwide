// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { boundsAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import {
  assignRun,
  editTurn,
  type NotesDocument,
  type NotesRun,
  renameSpeaker,
  runsOf,
  speakerName,
  speakerShares,
} from "../document/notes-document";
import { SpeakerAvatar } from "../speakers/speaker-avatar";
import { SpeakersStrip } from "../speakers/speakers-strip";

import { SpeakerLabel } from "./speaker-label";
import { TurnField } from "./turn-field";

const MENU_PREFIX = "notes-speaker:";
const MENU_WIDTH = 200;

// The notes as a page: who was on the call across its head, then each run
// of one speaker's turns under a line of their avatar and name, the words
// starting where the avatar does, each turn a paragraph edited in place. A
// paragraph of no one's has no such line. Changes come back whole, with the
// turn typed into named so a run of typing undoes as one step.

export function NotesPage({
  document,
  isReadOnly,
  onChange,
  onHearVoice,
  playingVoice,
  voices,
}: {
  document: NotesDocument;
  isReadOnly: boolean;
  /** The notes changed; `typing` names the turn typed into, if that was
   * the change. */
  onChange: (document: NotesDocument, typing?: string) => void;
  onHearVoice: (number: number) => void;
  playingVoice: number | null;
  voices: readonly number[];
}) {
  const runs = runsOf(document);
  const speakers = speakerShares(document);
  const rename = (from: string, name: string) => {
    const to = speakerName(name);
    if (to && to !== from) onChange(renameSpeaker(document, from, to));
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
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-group pt-separate">
      {speakers.length > 0 ? (
        <SpeakersStrip
          isReadOnly={isReadOnly}
          onHearVoice={onHearVoice}
          onRename={rename}
          playingVoice={playingVoice}
          speakers={speakers}
          voices={voices}
        />
      ) : null}
      {runs.map((run, index) => (
        <section className="group flex flex-col gap-inline" key={run.start}>
          {run.speaker === null ? null : (
            <div className="flex items-center gap-inline">
              <SpeakerAvatar name={run.speaker} />
              <SpeakerLabel
                isReadOnly={isReadOnly}
                name={run.speaker}
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
                onRename={(name) => {
                  if (run.speaker !== null) rename(run.speaker, name);
                }}
              />
            </div>
          )}
          <div className="flex flex-col gap-separate">
            {document.turns.slice(run.start, run.end).map((turn, offset) => {
              const turnIndex = run.start + offset;
              return (
                <TurnField
                  isReadOnly={isReadOnly}
                  key={turnIndex}
                  label={
                    turn.speaker === null
                      ? t("notes-view-paragraph")
                      : t("notes-view-turn", { speaker: turn.speaker })
                  }
                  onChange={(text) => {
                    onChange(
                      editTurn(document, turnIndex, text),
                      String(turnIndex),
                    );
                  }}
                  text={turn.text}
                />
              );
            })}
          </div>
        </section>
      ))}
    </div>
  );
}
