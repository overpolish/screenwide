// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { SpeakerAvatar } from "../speakers/speaker-avatar";

import { SpeakerLabel } from "./speaker-label";
import { TurnField } from "./turn-field";

import type { NotesDocument, NotesRun } from "../document/notes-document";
import type { FoundRange } from "../find/found-pieces";

// One run of a speaker's turns: a line of their avatar and name, then each
// turn a paragraph edited in place. A run of no one's has no such line.

export function NotesRunSection({
  document,
  found,
  isReadOnly,
  onChooseSpeaker,
  onEditTurn,
  onRemove,
  onRename,
  run,
}: {
  document: NotesDocument;
  /** Where a search found something, by turn. */
  found: ReadonlyMap<number, FoundRange[]>;
  isReadOnly: boolean;
  /** Opens the speakers to give this run to, under `anchor`. */
  onChooseSpeaker: (anchor: DOMRect) => void;
  onEditTurn: (turn: number, text: string) => void;
  onRemove: () => void;
  onRename: (name: string) => void;
  run: NotesRun;
}) {
  return (
    <section className="group flex flex-col gap-inline">
      {run.speaker === null ? null : (
        <div className="flex items-center gap-inline">
          <SpeakerAvatar name={run.speaker} />
          <SpeakerLabel
            isReadOnly={isReadOnly}
            name={run.speaker}
            onChooseSpeaker={onChooseSpeaker}
            onRemove={onRemove}
            onRename={onRename}
          />
        </div>
      )}
      <div className="flex flex-col gap-separate">
        {document.turns.slice(run.start, run.end).map((turn, offset) => {
          const turnIndex = run.start + offset;
          return (
            <TurnField
              found={found.get(turnIndex)}
              isReadOnly={isReadOnly}
              key={turnIndex}
              label={
                turn.speaker === null
                  ? t("notes-view-paragraph")
                  : t("notes-view-turn", { speaker: turn.speaker })
              }
              onChange={(text) => {
                onEditTurn(turnIndex, text);
              }}
              text={turn.text}
            />
          );
        })}
      </div>
    </section>
  );
}
