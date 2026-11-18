// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Reveal } from "../../../composites/reveal/reveal";
import { t } from "../../../i18n/i18n";
import { SensitiveBadges } from "../sensitive/sensitive-badges";
import { SpeakerAvatar } from "../speakers/speaker-avatar";

import { SpeakerLabel } from "./speaker-label";
import { TurnField } from "./turn-field";

import type { FoundRange } from "./marked-pieces";
import type { SensitiveFind } from "../../../bindings/SensitiveFind";
import type { NotesDocument, NotesRun } from "../document/notes-document";
import type { SensitiveGroup } from "../sensitive/sensitive-edits";
import type { SensitiveLookup } from "../sensitive/use-sensitive";

// One run of a speaker's turns: a line of their avatar and name, then each
// turn a paragraph edited in place, under badges naming what in it may be
// private. A run of no one's has no such line.

export function NotesRunSection({
  document,
  found,
  isReadOnly,
  onChooseSpeaker,
  onEditTurn,
  onRemove,
  onRename,
  onSensitiveBadge,
  onSensitiveWord,
  run,
  sensitive,
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
  /** Opens what can be done about `group` in turn `turn`, under `anchor`. */
  onSensitiveBadge: (
    turn: number,
    group: SensitiveGroup,
    anchor: DOMRect,
  ) => void;
  /** Opens what can be done about `find` in turn `turn`, right clicked at
   * `point`. */
  onSensitiveWord: (
    turn: number,
    find: SensitiveFind,
    point: { x: number; y: number },
  ) => void;
  run: NotesRun;
  /** What may be private in each turn. */
  sensitive: SensitiveLookup;
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
          const finds = isReadOnly ? [] : sensitive(turnIndex, turn.text);
          return (
            <div className="flex flex-col" key={turnIndex}>
              <Reveal className="pb-inline">
                {finds.length > 0 ? (
                  <SensitiveBadges
                    finds={finds}
                    onOpen={(group, anchor) => {
                      onSensitiveBadge(turnIndex, group, anchor);
                    }}
                  />
                ) : null}
              </Reveal>
              <TurnField
                found={found.get(turnIndex)}
                isReadOnly={isReadOnly}
                label={
                  turn.speaker === null
                    ? t("notes-view-paragraph")
                    : t("notes-view-turn", { speaker: turn.speaker })
                }
                onChange={(text) => {
                  onEditTurn(turnIndex, text);
                }}
                onSensitiveMenu={(find, point) => {
                  onSensitiveWord(turnIndex, find, point);
                }}
                sensitive={finds}
                text={turn.text}
              />
            </div>
          );
        })}
      </div>
    </section>
  );
}
