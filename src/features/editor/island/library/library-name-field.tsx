// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { EditableTitle } from "../../../../composites/editable-title/editable-title";

/** A library name being edited where it is shown, as a project's name is:
 * it takes focus with its words chosen, Enter or a click away keeps the
 * name, Escape puts it back, and either way the edit ends. */
export function LibraryNameField({
  className,
  label,
  onDone,
  onRename,
  title,
}: {
  /** The field's accessible name. */
  label: string;
  /** The edit ended, kept or not. */
  onDone: () => void;
  onRename: (title: string) => void;
  title: string;
  className?: string;
}) {
  const wrapRef = useRef<HTMLSpanElement>(null);
  useEffect(() => {
    const field = wrapRef.current?.querySelector<HTMLElement>("[role=textbox]");
    if (!field) return;
    field.focus();
    const selection = window.getSelection();
    selection?.selectAllChildren(field);
  }, []);
  return (
    <span className="min-w-0 grow" onBlur={onDone} ref={wrapRef}>
      <EditableTitle
        className={className}
        label={label}
        onChange={onRename}
        title={title}
      />
    </span>
  );
}
