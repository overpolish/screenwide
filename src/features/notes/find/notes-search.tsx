// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReplaceAll } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { SearchField } from "../../../primitives/input-fields/search-field";
import { TextField } from "../../../primitives/input-fields/text-field";

/**
 * The search over the notes, as a row at the top of the page, and the
 * toggle at its end that opens Replace under it, as the Script transcript's
 * does. Enter and Down go to the next match, Shift+Enter and Up to the one
 * before; Escape clears it. ⌘F or Ctrl+F comes back to it.
 */
export function NotesSearchRow({
  isReplacing,
  onQueryChange,
  onReplacingChange,
  onStep,
  query,
}: {
  isReplacing: boolean;
  onQueryChange: (query: string) => void;
  onReplacingChange: (isReplacing: boolean) => void;
  onStep: (by: -1 | 1) => void;
  query: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey) || event.key !== "f") return;
      event.preventDefault();
      ref.current?.querySelector("input")?.focus();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);
  return (
    <div className="flex items-center gap-separate" ref={ref}>
      <SearchField
        aria-label={t("notes-view-search")}
        className="min-w-0 flex-1"
        onChange={onQueryChange}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === "ArrowDown") {
            event.preventDefault();
            onStep(event.key === "Enter" && event.shiftKey ? -1 : 1);
          } else if (event.key === "ArrowUp") {
            event.preventDefault();
            onStep(-1);
          }
        }}
        placeholder={t("notes-view-search")}
        value={query}
      />
      <IconButton
        aria-expanded={isReplacing}
        aria-label={t("notes-view-find")}
        onPress={() => {
          onReplacingChange(!isReplacing);
        }}
        variant="ghost"
      >
        <ReplaceAll aria-hidden />
      </IconButton>
    </div>
  );
}

/** What the match gone to, or every match, becomes, and the buttons that
 * make it so, each as one step of the edit history. Replace goes to the
 * first match when none has been gone to yet. */
export function NotesReplaceLine({
  found,
  isDisabled,
  onReplace,
  onReplaceAll,
}: {
  found: number;
  isDisabled: boolean;
  onReplace: (replacement: string) => void;
  onReplaceAll: (replacement: string) => void;
}) {
  const [replacement, setReplacement] = useState("");
  const canReplace = !isDisabled && found > 0;
  return (
    <div className="flex items-center gap-separate">
      <TextField
        aria-label={t("notes-view-replace-field")}
        className="min-w-0 flex-1"
        icon={<ReplaceAll />}
        isClearable
        isDisabled={isDisabled}
        onChange={setReplacement}
        placeholder={t("notes-view-replace-field")}
        value={replacement}
      />
      <Button
        isDisabled={!canReplace}
        onPress={() => {
          onReplace(replacement);
        }}
      >
        {t("notes-view-replace")}
      </Button>
      <Button
        isDisabled={!canReplace}
        onPress={() => {
          onReplaceAll(replacement);
        }}
      >
        {t("notes-view-replace-all")}
      </Button>
    </div>
  );
}
