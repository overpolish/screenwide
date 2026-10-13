// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
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

import { TranscriptDropCaret } from "./transcript-drop-caret";
import { ClientPoint, EditStart } from "./transcript-edit-span";
import { nearestVerticalUnit, UnitRect } from "./transcript-line-geometry";
import { transcriptParagraphElements } from "./transcript-paragraphs";
import { TranscriptMatch } from "./transcript-search";
import {
  spanOf,
  TranscriptUnit,
  WordRange,
  WordSelection,
} from "./transcript-units";
import { transcriptParagraphs } from "./transcript-words";
import { usePlayingWord } from "./use-playing-word";
import { useWordChoosing } from "./use-word-choosing";
import { useWordDragging } from "./use-word-dragging";

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
  /** The unit being said, or -1 between units. */
  playingIndex: number;
  /** A Remove hovered or focused, and the units it would cut. */
  preview: { isActive: boolean; units: ReadonlySet<number> };
  selection: WordRange | null;
  units: readonly TranscriptUnit[];
  /** Move the units in `span` to stand before unit `before`, the unit count
   * for after the last. Without it, chosen words are not dragged. */
  onMove?: (span: WordSelection, before: number) => void;
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
    playingIndex: current,
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
  const dragging = useWordDragging({
    onMove: (moved, before) => props.onMove?.(moved, before),
    rootRef,
    spansRef,
  });
  const carriedFirst = dragging.drag?.span.first ?? -1;
  const carriedLast = dragging.drag?.span.last ?? -1;
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
  const content = useMemo(
    () =>
      transcriptParagraphElements({
        carried:
          carriedFirst < 0 ? null : { first: carriedFirst, last: carriedLast },
        chosen:
          chosenFirst < 0 ? null : { first: chosenFirst, last: chosenLast },
        editStart,
        editing,
        finishEdit,
        marks,
        matches,
        painted:
          paintFirst < 0
            ? null
            : { cut: paintCut, first: paintFirst, last: paintLast },
        paragraphs,
        preview,
        unitId,
        units,
      }),
    [
      carriedFirst,
      carriedLast,
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
    ],
  );

  useLayoutEffect(() => {
    const spans: HTMLElement[] = [];
    rootRef.current
      ?.querySelectorAll<HTMLElement>("[data-unit]")
      .forEach((element) => {
        spans[Number(element.dataset.unit)] = element;
      });
    spansRef.current = spans;
  }, [content]);

  const markScrolled = usePlayingWord(
    (index) => {
      const span = spansRef.current[index] as HTMLElement | undefined;
      return span ? [span] : [];
    },
    { content, current, isChoosing: selection !== null },
  );

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
    ...(props.onMove ? { onCarry: dragging.begin } : {}),
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
      constrainHeight
      rootClassName="max-h-64 rounded-control bg-fill-tertiary"
    >
      <div
        {...handlers}
        aria-activedescendant={
          focus !== undefined && !editing ? unitId(focus) : undefined
        }
        aria-disabled={isDisabled || undefined}
        aria-label={t("editor-panels-transcript")}
        aria-multiselectable
        className="relative flex cursor-default flex-col gap-section p-control-inset text-body text-content-fg outline-none select-none"
        data-owns-text-keys
        onTouchMove={markScrolled}
        onWheel={markScrolled}
        ref={rootRef}
        role="listbox"
        tabIndex={0}
      >
        {content}
        {dragging.drag?.boundary ? (
          <TranscriptDropCaret
            boundary={dragging.drag.boundary}
            rootRef={rootRef}
            spansRef={spansRef}
          />
        ) : null}
      </div>
    </ScrollArea>
  );
}
