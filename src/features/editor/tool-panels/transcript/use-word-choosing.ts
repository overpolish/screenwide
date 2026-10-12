// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ClipboardEvent,
  KeyboardEvent,
  MouseEvent,
  PointerEvent,
  useRef,
} from "react";

import { nextCutRun, nextUnsureWord } from "./transcript-cuts";
import { ClientPoint, EditStart } from "./transcript-edit-span";
import {
  spanOf,
  TranscriptUnit,
  WordRange,
  WordSelection,
} from "./transcript-units";
import { transcriptParagraphs } from "./transcript-words";

type Options = {
  isDisabled: boolean;
  onCut: (span: WordSelection) => void;
  /** Edit these units in place, begun as `start` says: the caret at a
   * point, or the chosen words replaced by what was typed. */
  onEdit: (span: WordSelection, start: EditStart) => void;
  onMenu: (span: WordSelection, point: ClientPoint) => void;
  /** Cut every unit in `span` where `cut`, or restore every one. */
  onPaint: (span: WordSelection, cut: boolean) => void;
  /** The units a Command-drag under way reaches, to show as it will leave
   * them; null once it ends. */
  onPaintChange: (paint: (WordRange & { cut: boolean }) | null) => void;
  onRedo: () => void;
  /** Restore every struck unit in `span`. */
  onRestore: (span: WordSelection) => void;
  onSeek: (ms: number) => void;
  onSelectionChange: (selection: WordRange | null) => void;
  onTogglePlayback: () => void;
  onUndo: () => void;
  /** The unit nearest `from` one line up (-1) or down (1), by its own
   * element geometry; `from` itself where there is no such line. */
  onVerticalMove: (from: number, direction: -1 | 1) => number;
  /** The unit being said, where the keys start from with nothing chosen. */
  playingIndex: number;
  selection: WordRange | null;
  units: readonly TranscriptUnit[];
};

/** The unit an event reached, leaving out the one being edited, whose own
 * letters take the pointer. */
const unitAt = (target: EventTarget | null) => {
  const span =
    target instanceof Element
      ? target.closest<HTMLElement>("[data-unit]")
      : null;
  return span && !span.isContentEditable ? Number(span.dataset.unit) : null;
};

/** A key that types a character, as opposed to one that moves, deletes or
 * runs a shortcut: typing it over chosen words replaces them. A key held for
 * an accent or an input method still composing types nothing yet. */
const isTyped = (event: KeyboardEvent<HTMLElement>) =>
  /^.$/u.test(event.key) &&
  !event.metaKey &&
  !event.ctrlKey &&
  !event.nativeEvent.isComposing;

const textOf = (units: readonly TranscriptUnit[], span: WordSelection) => {
  const slice = units.slice(span.first, span.last + 1);
  return transcriptParagraphs(slice)
    .map(({ end, start }) =>
      slice
        .slice(start, end)
        .filter((unit) => unit.kind === "word")
        .map((unit) => unit.text)
        .join(" "),
    )
    .join("\n\n");
};

/** Calls `onReach` with each unit the pointer reaches until it is released,
 * then `onEnd`. Followed on the window rather than captured, which would
 * hand the click, and so the double-click, to the transcript instead of the
 * word. */
function follow(onReach: (index: number) => void, onEnd: () => void) {
  const onMove = (move: globalThis.PointerEvent) => {
    const reached = unitAt(
      document.elementFromPoint(move.clientX, move.clientY),
    );
    if (reached !== null) onReach(reached);
  };
  const onUp = () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    onEnd();
  };
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
}

/**
 * Choosing words the way text is chosen, a whole word at a time: a click
 * picks one and moves the playhead to it, Shift-click and dragging reach
 * further, and the arrow keys do the same from the keyboard.
 * What is chosen is cut with Delete, edited with Enter or a double-click,
 * copied, and offered in a menu on a right click. Command-click cuts or
 * restores a word, and Command-drag does the same to every word it reaches,
 * each left as the first one was made.
 */
export function useWordChoosing(options: Options) {
  const { isDisabled, selection, units } = options;
  const latestRef = useRef(options);
  latestRef.current = options;
  const span = selection ? spanOf(selection) : null;

  const choose = (index: number, extend: boolean) => {
    const at = Math.min(Math.max(index, 0), units.length - 1);
    options.onSelectionChange({
      anchor: extend && selection ? selection.anchor : at,
      focus: at,
    });
  };

  const onPointerDown = (event: PointerEvent<HTMLElement>) => {
    if (event.button !== 0) return;
    if (event.target instanceof HTMLElement && event.target.isContentEditable)
      return;
    // Taking focus is what saves a word being edited.
    event.currentTarget.focus({ preventScroll: true });
    const index = unitAt(event.target);
    if (isDisabled || index === null) return;
    if (event.metaKey || event.ctrlKey) {
      const cut = !units[index].removed;
      let reach = index;
      options.onPaintChange({ anchor: index, cut, focus: index });
      follow(
        (reached) => {
          if (reached === reach) return;
          reach = reached;
          latestRef.current.onPaintChange({ anchor: index, cut, focus: reach });
        },
        () => {
          const { onPaint, onPaintChange } = latestRef.current;
          onPaintChange(null);
          onPaint(spanOf({ anchor: index, focus: reach }), cut);
        },
      );
      return;
    }
    if (event.shiftKey && selection) {
      choose(index, true);
      return;
    }
    choose(index, false);
    options.onSeek(units[index].startMs);
    follow(
      (reached) => {
        const { onSelectionChange, selection: latest } = latestRef.current;
        if (reached !== latest?.focus)
          onSelectionChange({ anchor: index, focus: reached });
      },
      () => undefined,
    );
  };

  const onDoubleClick = (event: MouseEvent<HTMLElement>) => {
    const index = unitAt(event.target);
    if (isDisabled || index === null || event.metaKey || event.ctrlKey) return;
    options.onEdit(
      { first: index, last: index },
      { point: { x: event.clientX, y: event.clientY } },
    );
  };

  const onContextMenu = (event: MouseEvent<HTMLElement>) => {
    const index = unitAt(event.target);
    if (index === null) return;
    event.preventDefault();
    if (isDisabled) return;
    // A word outside what is chosen is chosen on its own first.
    const chosen =
      span && span.first <= index && index <= span.last
        ? span
        : { first: index, last: index };
    if (chosen !== span) choose(index, false);
    options.onMenu(chosen, { x: event.clientX, y: event.clientY });
  };

  const chooseCut = (at: number, direction: 1 | -1) => {
    const run = nextCutRun(units, at, direction);
    if (!run) return false;
    options.onSelectionChange({ anchor: run.first, focus: run.last });
    options.onSeek(units[run.first].startMs);
    return true;
  };

  const chooseUnsure = (at: number, direction: 1 | -1) => {
    const index = nextUnsureWord(units, at, direction);
    if (index < 0) return false;
    options.onSelectionChange({ anchor: index, focus: index });
    options.onSeek(units[index].startMs);
    return true;
  };

  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.target !== event.currentTarget || isDisabled) return;
    const command = event.metaKey || event.ctrlKey;
    const from = selection?.focus ?? Math.max(options.playingIndex, 0);
    const handled = (() => {
      if (command && event.key.toLowerCase() === "z") {
        if (event.shiftKey) options.onRedo();
        else options.onUndo();
      } else if (command) return false;
      else if (event.altKey && event.key === "ArrowRight")
        return chooseCut(from, 1);
      else if (event.altKey && event.key === "ArrowLeft")
        return chooseCut(from, -1);
      else if (event.key === "F8")
        return chooseUnsure(from, event.shiftKey ? -1 : 1);
      else if (event.key === "ArrowUp")
        choose(options.onVerticalMove(from, -1), event.shiftKey);
      else if (event.key === "ArrowDown")
        choose(options.onVerticalMove(from, 1), event.shiftKey);
      else if (event.key === "ArrowLeft")
        choose(selection ? from - 1 : from, event.shiftKey);
      else if (event.key === "ArrowRight")
        choose(selection ? from + 1 : from, event.shiftKey);
      else if (event.key === " ") options.onTogglePlayback();
      else if (event.key === "Escape") {
        if (selection) options.onSelectionChange(null);
        else event.currentTarget.blur();
      } else if (!span) return false;
      else if (event.key === "Enter" || event.key === "F2")
        options.onEdit(span, { point: null });
      else if (
        (event.key === "Delete" || event.key === "Backspace") &&
        event.shiftKey
      )
        options.onRestore(span);
      else if (event.key === "Delete" || event.key === "Backspace")
        options.onCut(span);
      else if (
        event.key === "ContextMenu" ||
        (event.key === "F10" && event.shiftKey)
      ) {
        const bounds = event.currentTarget
          .querySelector(`[data-unit="${span.first.toString()}"]`)
          ?.getBoundingClientRect();
        if (bounds) options.onMenu(span, { x: bounds.left, y: bounds.bottom });
      } else if (isTyped(event))
        options.onEdit(span, { point: null, typed: event.key });
      else return false;
      return true;
    })();
    if (handled) event.preventDefault();
  };

  const onCopy = (event: ClipboardEvent<HTMLElement>) => {
    if (!span || event.target !== event.currentTarget) return;
    event.preventDefault();
    event.clipboardData.setData("text/plain", textOf(units, span));
  };

  const onCut = (event: ClipboardEvent<HTMLElement>) => {
    if (!span || event.target !== event.currentTarget || isDisabled) return;
    onCopy(event);
    options.onCut(span);
  };

  return {
    onContextMenu,
    onCopy,
    onCut,
    onDoubleClick,
    onKeyDown,
    onPointerDown,
  };
}
