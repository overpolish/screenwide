// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef } from "react";
import { TextArea, TextField } from "react-aria-components";

// A turn's words, edited where they are read, as the Script transcript's
// are: plain body text with no field drawn round it, growing with what it
// holds so the notes read as one page and scroll as one.

export function TurnField({
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
}) {
  const ref = useRef<HTMLTextAreaElement>(null);
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
  }, [text]);
  return (
    <TextField
      aria-label={label}
      className="flex"
      isReadOnly={isReadOnly}
      onChange={onChange}
      value={text}
    >
      <TextArea
        className="w-full resize-none overflow-hidden bg-transparent text-body text-fg outline-none"
        dir="auto"
        ref={ref}
        rows={1}
      />
    </TextField>
  );
}
