// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef } from "react";
import { TextArea, TextField } from "react-aria-components";

import { Text } from "../../../primitives/text/text";
import { type FoundRange, foundPieces } from "../find/found-pieces";

// A turn's words, edited where they are read, as the Script transcript's
// are: plain body text with no field drawn round it, growing with what it
// holds so the notes read as one page and scroll as one. Notes that cannot
// be edited yet, as they are still being taken, are only text, with no
// caret to place in them. What a search found is tinted under the words: a
// field's text cannot be marked itself, so the same text is laid out
// beneath it, unseen but for its tints.

export function TurnField({
  found = [],
  isReadOnly,
  label,
  onChange,
  text,
}: {
  isReadOnly: boolean;
  /** Says whose words they are, for assistive technology. */
  label: string;
  onChange: (text: string) => void;
  text: string;
  /** Where a search found something in the text. */
  found?: readonly FoundRange[];
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
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
        {foundPieces(text, found, currentRef)}
      </Text>
    );
  }
  return (
    <div className="relative">
      {found.length > 0 ? (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 text-body break-words whitespace-pre-wrap text-transparent [&_span]:text-transparent"
          dir="auto"
        >
          {foundPieces(text, found, currentRef)}
        </div>
      ) : null}
      <TextField
        aria-label={label}
        className="flex"
        onChange={onChange}
        value={text}
      >
        <TextArea
          className="relative w-full resize-none overflow-hidden bg-transparent text-body text-fg outline-none"
          dir="auto"
          ref={ref}
          rows={1}
        />
      </TextField>
    </div>
  );
}
