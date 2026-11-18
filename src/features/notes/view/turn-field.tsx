// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef } from "react";
import { TextArea, TextField } from "react-aria-components";

import { Text } from "../../../primitives/text/text";

import { type FoundRange, markedPieces, sensitiveAt } from "./marked-pieces";

import type { SensitiveFind } from "../../../bindings/SensitiveFind";

// A turn's words, edited where they are read, as the Script transcript's
// are: plain body text with no field drawn round it, growing with what it
// holds so the notes read as one page and scroll as one. Notes that cannot
// be edited yet, as they are still being taken, are only text, with no
// caret to place in them. What a search found is tinted under the words,
// and what may be private underlined: a field's text cannot be marked
// itself, so the same text is laid out beneath it, unseen but for its
// marks. A right click on a private find opens what can be done about it;
// anywhere else it is the field's own.

export function TurnField({
  found = [],
  isReadOnly,
  label,
  onChange,
  onSensitiveMenu,
  sensitive = [],
  text,
}: {
  isReadOnly: boolean;
  /** Says whose words they are, for assistive technology. */
  label: string;
  onChange: (text: string) => void;
  /** Opens what can be done about `find`, right clicked at `point`. */
  onSensitiveMenu: (
    find: SensitiveFind,
    point: { x: number; y: number },
  ) => void;
  text: string;
  /** Where a search found something in the text. */
  found?: readonly FoundRange[];
  /** What in the text may be private. */
  sensitive?: readonly SensitiveFind[];
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
  const marksRef = useRef<HTMLDivElement>(null);
  const currentRef = useRef<HTMLSpanElement>(null);
  const current = found.find(({ isCurrent }) => isCurrent);
  // As tall as its text at the width it is given, measured again as the
  // text changes; a window resized wraps it anew, which the observer sees.
  useLayoutEffect(() => {
    const field = ref.current;
    if (!field) return;
    const fit = () => {
      field.style.height = "0px";
      field.style.height = `${String(field.scrollHeight)}px`;
    };
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(field);
    return () => {
      observer.disconnect();
    };
  }, [isReadOnly, text]);
  useEffect(() => {
    currentRef.current?.scrollIntoView({ block: "nearest" });
  }, [current?.from, current?.to]);
  if (isReadOnly) {
    return (
      <Text className="break-words whitespace-pre-wrap" dir="auto">
        {markedPieces(text, { current: currentRef, found, sensitive })}
      </Text>
    );
  }
  return (
    <div
      className="relative"
      onContextMenu={(event) => {
        const point = { x: event.clientX, y: event.clientY };
        const index = marksRef.current
          ? sensitiveAt(marksRef.current, point)
          : null;
        const find =
          index === null
            ? undefined
            : (sensitive[index] as SensitiveFind | undefined);
        if (!find) return;
        event.preventDefault();
        onSensitiveMenu(find, point);
      }}
    >
      {found.length > 0 || sensitive.length > 0 ? (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 text-body break-words whitespace-pre-wrap text-transparent [&_span]:text-transparent"
          dir="auto"
          ref={marksRef}
        >
          {markedPieces(text, { current: currentRef, found, sensitive })}
        </div>
      ) : null}
      <TextField
        aria-label={label}
        className="flex"
        onChange={onChange}
        value={text}
      >
        {/* The browser's own text-rendering for a field is not the page's,
            which sets its words a touch narrower, so the marks beneath
            would drift short of them along a line. */}
        <TextArea
          className="relative w-full resize-none overflow-hidden bg-transparent text-body text-fg outline-none [text-rendering:inherit]"
          dir="auto"
          ref={ref}
          rows={1}
        />
      </TextField>
    </div>
  );
}
