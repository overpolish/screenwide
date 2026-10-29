// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Fragment } from "react";

import { cn } from "../../../../lib/styling";

import { EditStart, TranscriptEditSpan } from "./transcript-edit-span";
import { foundRanges, foundText } from "./transcript-found-text";
import { TranscriptPauseChip } from "./transcript-pause-chip";
import { TranscriptMatch } from "./transcript-search";
import { pauseClass, unitClass } from "./transcript-unit-styles";
import { isWithin, TranscriptUnit, WordSelection } from "./transcript-units";

import type { TranscriptMark } from "./use-transcript-marks";

/** What the transcript's words are drawn as, beside the words themselves. */
export type TranscriptParagraphsState = {
  /** The units a drag is carrying, drawn faded where they are. */
  carried: WordSelection | null;
  chosen: WordSelection | null;
  editStart: EditStart;
  editing: WordSelection | null;
  finishEdit: (text: string | null) => void;
  marks: ReadonlyMap<number, TranscriptMark>;
  matches: readonly TranscriptMatch[];
  /** The units a Command-drag reaches, shown cut or kept as it will leave
   * them. */
  painted: (WordSelection & { cut: boolean }) | null;
  /** Each paragraph, as `[start, end)` unit indices. */
  paragraphs: readonly { end: number; start: number }[];
  preview: { isActive: boolean; units: ReadonlySet<number> };
  unitId: (index: number) => string;
  units: readonly TranscriptUnit[];
};

/**
 * The transcript's paragraphs as elements: each unit an option of the
 * listbox, marked as chosen, found, previewed or carried, and the units
 * being edited as one editable span. Cut words stay where they were, struck
 * through, to be read in place and restored.
 */
export function transcriptParagraphElements({
  carried,
  chosen,
  editStart,
  editing,
  finishEdit,
  marks,
  matches,
  painted,
  paragraphs,
  preview,
  unitId,
  units,
}: TranscriptParagraphsState) {
  const found = foundRanges(units, matches);
  return paragraphs.map(({ end, start }) => (
    <p
      className="m-0"
      dir="auto"
      key={units[start].startMs}
      role="presentation"
    >
      {units.slice(start, end).map((unit, offset) => {
        const index = start + offset;
        const isEdited = isWithin(editing, index);
        if (isEdited && index !== editing?.first) return null;
        const isChosen = !isEdited && isWithin(chosen, index);
        const isFound = found.owners[index] >= 0;
        const shown =
          painted && isWithin(painted, index)
            ? { ...unit, removed: painted.cut }
            : unit;
        return (
          <Fragment key={unit.startMs}>
            {offset > 0 ? (
              <span
                className={cn(
                  isFound &&
                    found.owners[index - 1] === found.owners[index] &&
                    "bg-warning/35",
                  isChosen && isWithin(chosen, index - 1) && "bg-selected",
                )}
              >
                {" "}
              </span>
            ) : null}
            {isEdited && editing ? (
              <TranscriptEditSpan
                className="outline-none select-text"
                id={unitId(index)}
                index={index}
                onFinish={finishEdit}
                start={editStart}
                text={units
                  .slice(editing.first, editing.last + 1)
                  .map((each) => each.text)
                  .join(" ")}
              />
            ) : (
              <span
                aria-selected={isChosen}
                className={cn(
                  shown.kind === "pause"
                    ? pauseClass(shown, isChosen, preview.units.has(index))
                    : unitClass(shown, isChosen, {
                        isPreviewed: preview.units.has(index),
                        isPreviewing: preview.isActive,
                        mark: marks.get(index),
                      }),
                  // Words being dragged stay in place, faded, until dropped.
                  isWithin(carried, index) && "opacity-50",
                )}
                data-unit={index}
                id={unitId(index)}
                role="option"
              >
                {unit.kind === "pause" ? (
                  <TranscriptPauseChip durationMs={unit.endMs - unit.startMs} />
                ) : (
                  foundText(
                    unit.text,
                    isChosen ? undefined : found.ranges.get(index),
                  )
                )}
              </span>
            )}
          </Fragment>
        );
      })}
    </p>
  ));
}
