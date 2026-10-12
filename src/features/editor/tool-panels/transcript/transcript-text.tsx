// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Fragment,
  useCallback,
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { ScrollArea } from "../../../../components/base/scroll-area/scroll-area";
import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";

import {
  ClientPoint,
  EditStart,
  TranscriptEditSpan,
} from "./transcript-edit-span";
import { foundRanges, foundText } from "./transcript-found-text";
import { nearestVerticalUnit, UnitRect } from "./transcript-line-geometry";
import { TranscriptPauseChip } from "./transcript-pause-chip";
import { TranscriptMatch } from "./transcript-search";
import { pauseClass, unitClass } from "./transcript-unit-styles";
import {
  isWithin,
  spanOf,
  TranscriptUnit,
  WordRange,
  WordSelection,
} from "./transcript-units";
import { transcriptParagraphs, wordAt } from "./transcript-words";
import { usePlayingWord } from "./use-playing-word";
import { useWordChoosing } from "./use-word-choosing";

import type { TranscriptMark } from "./use-transcript-marks";

export type TranscriptTextProps = {
  /** The units being edited in place, or null while words are chosen. */
  editing: WordSelection | null;
  isDisabled: boolean;
  /** What each unit is marked as needing a look for, where it is. */
  marks: ReadonlyMap<number, TranscriptMark>;
  /** What a search found: the units each match covers, and where in their
   * text it starts and ends. */
  matches: readonly TranscriptMatch[];
  onCut: (span: WordSelection) => void;
  /** Start editing these units, or stop with what was typed, or with null
   * to put them back. */
  onEditEnd: (span: WordSelection, text: string | null) => void;
  onEditStart: (span: WordSelection) => void;
  onMenu: (span: WordSelection, point: ClientPoint) => void;
  /** Cut every unit in `span` where `cut`, or restore every one. */
  onPaint: (span: WordSelection, cut: boolean) => void;
  onRedo: () => void;
  /** Restore every struck unit in `span`. */
  onRestore: (span: WordSelection) => void;
  onSeek: (ms: number) => void;
  onSelectionChange: (selection: WordRange | null) => void;
  onTogglePlayback: () => void;
  onUndo: () => void;
  playheadMs: number;
  /** A Remove hovered or focused, and the units it would cut. */
  preview: { isActive: boolean; units: ReadonlySet<number> };
  selection: WordRange | null;
  units: readonly TranscriptUnit[];
};

/**
 * The transcript as words you choose, cut and correct. Words are chosen a
 * whole word at a time, and cut words stay where they were, struck through,
 * to be restored. A double-click or Enter edits the chosen words in place.
 * The word being said is marked and kept in view.
 */
export function TranscriptText(props: TranscriptTextProps) {
  const {
    editing,
    isDisabled,
    marks,
    matches,
    playheadMs,
    preview,
    selection,
    units,
  } = props;
  const id = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const spansRef = useRef<HTMLElement[]>([]);
  const latestRef = useRef(props);
  latestRef.current = props;
  const [editStart, setEditStart] = useState<EditStart>({ point: null });
  // A Command-drag under way: the units it reaches, shown as it will leave
  // them until it ends.
  const [paint, setPaint] = useState<(WordRange & { cut: boolean }) | null>(
    null,
  );
  const span = selection ? spanOf(selection) : null;
  const chosenFirst = span?.first ?? -1;
  const chosenLast = span?.last ?? -1;
  const painted = paint ? spanOf(paint) : null;
  const paintFirst = painted?.first ?? -1;
  const paintLast = painted?.last ?? -1;
  const paintCut = paint?.cut ?? false;
  const current = wordAt(units, playheadMs);
  const unitId = useCallback(
    (index: number) => `${id}-${index.toString()}`,
    [id],
  );

  // Paragraphs follow what was heard, so correcting a word's punctuation
  // never moves a break.
  const paragraphs = useMemo(() => transcriptParagraphs(units), [units]);
  const finishEdit = useCallback((text: string | null) => {
    const { editing: edited, onEditEnd } = latestRef.current;
    if (edited) onEditEnd(edited, text);
    rootRef.current?.focus({ preventScroll: true });
  }, []);
  const content = useMemo(() => {
    const chosen =
      chosenFirst < 0 ? null : { first: chosenFirst, last: chosenLast };
    const reached =
      paintFirst < 0 ? null : { first: paintFirst, last: paintLast };
    const found = foundRanges(units, matches);
    return paragraphs.map(({ end, start }) => (
      // Cut words stay where they were, struck through, to be read in place
      // and restored.
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
          const shown = isWithin(reached, index)
            ? { ...unit, removed: paintCut }
            : unit;
          return (
            <Fragment key={unit.startMs}>
              {offset > 0 ? (
                <span
                  className={cn(
                    isFound &&
                      found.owners[index - 1] === found.owners[index] &&
                      "bg-warning/35",
                    isChosen &&
                      isWithin(chosen, index - 1) &&
                      "bg-primary-tint",
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
                  )}
                  data-unit={index}
                  id={unitId(index)}
                  role="option"
                >
                  {unit.kind === "pause" ? (
                    <TranscriptPauseChip
                      durationMs={unit.endMs - unit.startMs}
                    />
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
  }, [
    chosenFirst,
    chosenLast,
    editStart,
    editing,
    finishEdit,
    paragraphs,
    marks,
    preview,
    unitId,
    units,
    matches,
    paintCut,
    paintFirst,
    paintLast,
  ]);

  useLayoutEffect(() => {
    const spans: HTMLElement[] = [];
    rootRef.current
      ?.querySelectorAll<HTMLElement>("[data-unit]")
      .forEach((element) => {
        spans[Number(element.dataset.unit)] = element;
      });
    spansRef.current = spans;
  }, [content]);

  const markScrolled = usePlayingWord(spansRef, {
    content,
    current,
    isChoosing: selection !== null,
  });

  // The word the keys moved to is kept in view.
  const focus = selection?.focus;
  useEffect(() => {
    if (focus !== undefined)
      spansRef.current[focus]?.scrollIntoView({ block: "nearest" });
  }, [focus]);

  const handlers = useWordChoosing({
    ...props,
    onEdit: (chosen, start) => {
      setEditStart(start);
      props.onEditStart(chosen);
    },
    onPaintChange: setPaint,
    onVerticalMove: (from, direction) => {
      const rects: (UnitRect | undefined)[] = spansRef.current.map((element) =>
        element.getBoundingClientRect(),
      );
      return nearestVerticalUnit(rects, from, direction);
    },
    playingIndex: current,
  });

  return (
    <ScrollArea
      className="pe-control-inset"
      constrainHeight
      rootClassName="max-h-64"
    >
      <div
        {...handlers}
        aria-activedescendant={
          focus !== undefined && !editing ? unitId(focus) : undefined
        }
        aria-disabled={isDisabled || undefined}
        aria-label={t("editor-panels-transcript")}
        aria-multiselectable
        className="flex cursor-default flex-col gap-section text-body text-content-fg outline-none select-none"
        data-owns-text-keys
        onTouchMove={markScrolled}
        onWheel={markScrolled}
        ref={rootRef}
        role="listbox"
        tabIndex={0}
      >
        {content}
      </div>
    </ScrollArea>
  );
}
