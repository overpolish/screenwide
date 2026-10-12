// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchField } from "../../../../components/base/input-fields/search-field";
import { t } from "../../../../i18n/i18n";

/**
 * The search over the transcript's words. Enter and Down go to the next
 * match, Shift+Enter and Up to the one before; Escape clears the search, and
 * once it is empty hands the keys back to the words.
 */
export function TranscriptSearchField({
  isDisabled,
  onLeave,
  onQueryChange,
  onStep,
  query,
}: {
  isDisabled: boolean;
  onLeave: () => void;
  onQueryChange: (query: string) => void;
  onStep: (step: -1 | 1) => void;
  query: string;
}) {
  return (
    <SearchField
      aria-label={t("editor-panels-transcript-search")}
      className="min-w-0 flex-1"
      isDisabled={isDisabled}
      onChange={onQueryChange}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === "ArrowDown") {
          event.preventDefault();
          onStep(event.key === "Enter" && event.shiftKey ? -1 : 1);
        } else if (event.key === "ArrowUp") {
          event.preventDefault();
          onStep(-1);
        } else if (event.key === "Escape" && query === "") onLeave();
      }}
      placeholder={t("editor-panels-transcript-search")}
      value={query}
    />
  );
}
