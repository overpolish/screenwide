// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type PointerEvent, useRef } from "react";

import { t } from "../../../../../i18n/i18n";
import { cn } from "../../../../../lib/styling";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import {
  isChosenWord,
  textWordLines,
} from "../../../tool-panels/annotation/text/text-words";

import type { TextSpan } from "../../../../../bindings/TextSpan";

/** The pill under `element`, of those in `group`, by its place among them,
 * which is the word's place in the text; -1 off every pill. */
const pillIndex = (group: HTMLElement, element: Element | null) => {
  const pill = element?.closest("button");
  return pill && group.contains(pill)
    ? [...group.querySelectorAll("button")].indexOf(pill)
    : -1;
};

/** The words of a text as pills, in the order they are said, any of which a
 * press sets apart in the template's second appearance, its accent or its
 * highlight, or takes back. Pressed and dragged on, every word the pointer
 * passes over is set as the pressed one was, never flipped back, so a run
 * of words is set apart, or taken back, in one stroke; the stroke is one
 * edit (`onEditGestureStart`, `onEditGestureEnd`). The keyboard still sets
 * them one at a time. Nothing is set apart until a word is chosen. A chosen
 * word leans as italic does as well as filling, so the pill says what
 * choosing it does to the word, not only that it is chosen. It is skewed
 * rather than set in italic because the interface font ships upright only
 * and the webview does not slant it for us. */
export function TextWordsChoice({
  emphasis,
  onChange,
  onEditGestureEnd,
  onEditGestureStart,
  text,
}: {
  /** The words set apart, by the bytes they take in the text. */
  emphasis: TextSpan[];
  onChange: (emphasis: TextSpan[]) => void;
  text: string;
  onEditGestureEnd?: () => void;
  onEditGestureStart?: () => void;
}) {
  const words = textWordLines(text).flat();
  const key = (span: TextSpan) => String(span.start);
  const chosen = words.map((word) => isChosenWord(emphasis, word));
  // The stroke in hand: the state it sets, and the words as it has left
  // them, kept here since the text's own may not have caught up between
  // two moves.
  const strokeRef = useRef<{ chosen: boolean[]; sets: boolean } | null>(null);

  const set = (index: number) => {
    const stroke = strokeRef.current;
    if (!stroke || index < 0 || stroke.chosen[index] === stroke.sets) return;
    stroke.chosen = stroke.chosen.map((each, at) =>
      at === index ? stroke.sets : each,
    );
    onChange(
      words.filter((_, at) => stroke.chosen[at]).map((word) => word.span),
    );
  };
  const finish = () => {
    if (!strokeRef.current) return;
    strokeRef.current = null;
    onEditGestureEnd?.();
  };

  return (
    <div
      // As wide as the section's other lines make it and no wider
      // (`w-0 min-w-full`), wrapping within that, so a long text adds rows
      // rather than widening the island. A drag on it sets words rather
      // than scrolling (`touch-none`).
      className="w-0 min-w-full touch-none"
      onLostPointerCapture={finish}
      onPointerDownCapture={(event: PointerEvent<HTMLDivElement>) => {
        if (event.button !== 0) return;
        const group = event.currentTarget;
        const index = pillIndex(group, event.target as Element);
        if (index < 0) return;
        // The stroke sets the word itself, so the pill's own press, which
        // would toggle it again on release, is kept from starting; and no
        // text is selected as the pointer moves.
        event.preventDefault();
        event.stopPropagation();
        (event.target as Element)
          .closest("button")
          ?.focus({ preventScroll: true });
        group.setPointerCapture(event.pointerId);
        onEditGestureStart?.();
        strokeRef.current = { chosen, sets: !chosen[index] };
        set(index);
      }}
      onPointerMove={(event) => {
        if (!strokeRef.current) return;
        set(
          pillIndex(
            event.currentTarget,
            document.elementFromPoint(event.clientX, event.clientY),
          ),
        );
      }}
      onPointerUp={(event) => {
        if (event.currentTarget.hasPointerCapture(event.pointerId))
          event.currentTarget.releasePointerCapture(event.pointerId);
        finish();
      }}
    >
      <PillGroup
        aria-label={t("editor-island-text-words")}
        className="flex-wrap"
        items={words.map((word, index) => ({
          id: key(word.span),
          label: word.text,
          labelClassName: cn(
            "inline-block transition-transform",
            chosen[index] && "-skew-x-10",
          ),
        }))}
        onSelectionChange={(ids) => {
          onChange(
            words
              .filter((word) => ids.includes(key(word.span)))
              .map((word) => word.span),
          );
        }}
        selected={words
          .filter((_, index) => chosen[index])
          .map((word) => key(word.span))}
        selectionMode="multiple"
      />
    </div>
  );
}
