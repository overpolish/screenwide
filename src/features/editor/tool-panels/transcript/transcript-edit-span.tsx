// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef } from "react";

import { t } from "../../../../i18n/i18n";

/** A point in the panel window, in client pixels. */
export type ClientPoint = { x: number; y: number };

/** How an edit began: where a double-click put the caret, or the letter
 * typed over the chosen words, which replaces them. */
export type EditStart = { point: ClientPoint | null; typed?: string };

/** The caret the platform would put at `point`, where it has one to say. */
function caretAt({ x, y }: ClientPoint) {
  const position = document.caretPositionFromPoint(x, y);
  if (!position) return null;
  const range = document.createRange();
  range.setStart(position.offsetNode, position.offset);
  return range;
}

/**
 * A word, or the words of a phrase, being edited in place: plain text you
 * type into, with the caret where you double-clicked, every letter chosen
 * when editing began from Enter, or only what was typed with the caret after
 * it when typing over the words began it. Enter or leaving it saves what it
 * holds, Escape puts it back. Drawn afresh for each edit, so what the browser
 * did to the text never outlives it.
 */
export function TranscriptEditSpan({
  className,
  id,
  index,
  onFinish,
  start,
  text,
}: {
  className: string;
  id: string;
  /** The first unit being edited. */
  index: number;
  /** The text to save, or null to put the words back as they were. */
  onFinish: (text: string | null) => void;
  start: EditStart;
  text: string;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const isDoneRef = useRef(false);

  useLayoutEffect(() => {
    const span = ref.current;
    const selection = document.getSelection();
    if (!span || !selection) return;
    span.focus({ preventScroll: true });
    const caret = start.point ? caretAt(start.point) : null;
    if (caret && span.contains(caret.startContainer))
      selection.collapse(caret.startContainer, caret.startOffset);
    else selection.selectAllChildren(span);
    if (start.typed !== undefined) selection.collapseToEnd();
  }, [start]);

  const finish = (save: boolean) => {
    if (isDoneRef.current) return;
    isDoneRef.current = true;
    onFinish(save ? (ref.current?.textContent ?? text) : null);
  };

  return (
    <span
      aria-label={t("editor-panels-transcript-editing")}
      className={className}
      contentEditable
      data-unit={index}
      id={id}
      onBlur={() => {
        finish(true);
      }}
      onDrop={(event) => {
        event.preventDefault();
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          finish(true);
        } else if (event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          finish(false);
        }
      }}
      ref={ref}
      role="textbox"
      spellCheck={false}
      suppressContentEditableWarning
    >
      {start.typed ?? text}
    </span>
  );
}
