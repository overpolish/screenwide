// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef } from "react";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

export function EditableWindowTitle({
  onChange,
  title,
}: {
  onChange: (title: string) => void;
  title: string;
}) {
  const titleRef = useRef<HTMLSpanElement>(null);
  const originalRef = useRef(title);
  useLayoutEffect(() => {
    if (titleRef.current && document.activeElement !== titleRef.current) {
      titleRef.current.textContent = title;
    }
  }, [title]);

  return (
    <span
      aria-label="Document name"
      aria-multiline={false}
      className={cn(
        "pointer-events-auto block cursor-text select-text whitespace-nowrap rounded-lg text-left outline-none caret-content-fg focus:selection:bg-content-fg/25",
        focusStyles,
        elementFocusVisible,
      )}
      contentEditable="plaintext-only"
      onBlur={(event) => {
        const next =
          event.currentTarget.textContent.replace(/\s+/g, " ").trim() ||
          originalRef.current;
        event.currentTarget.textContent = next;
        if (next !== title) onChange(next);
      }}
      onFocus={() => {
        originalRef.current = title;
      }}
      onKeyDown={(event) => {
        if (event.nativeEvent.isComposing) return;
        if (event.key === "Enter" || event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          if (event.key === "Escape")
            event.currentTarget.textContent = originalRef.current;
          event.currentTarget.blur();
        }
      }}
      onPaste={(event) => {
        event.preventDefault();
        const selection = window.getSelection();
        if (!selection?.rangeCount) return;
        const range = selection.getRangeAt(0);
        if (!event.currentTarget.contains(range.commonAncestorContainer))
          return;
        const text = document.createTextNode(
          event.clipboardData.getData("text/plain").replace(/\s+/g, " "),
        );
        range.deleteContents();
        range.insertNode(text);
        range.setStartAfter(text);
        range.collapse(true);
        selection.removeAllRanges();
        selection.addRange(range);
      }}
      ref={titleRef}
      role="textbox"
      suppressContentEditableWarning
      tabIndex={0}
    />
  );
}
