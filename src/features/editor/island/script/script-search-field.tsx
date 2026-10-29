// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReplaceAll } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { SearchField } from "../../../../primitives/input-fields/search-field";

import { useScriptSearch } from "./script-search";

/**
 * The search over the script, at the end of its row, and the toggle that
 * opens Replace under the row. Enter and Down go to the next match,
 * Shift+Enter and Up to the one before; Escape clears the search, and once
 * it is empty hands the keys back to the words. The field keeps one width,
 * so typing never resizes the island.
 */
export function ScriptSearchField({ isDisabled }: { isDisabled: boolean }) {
  const query = useScriptSearch((state) => state.query);
  const isReplacing = useScriptSearch((state) => state.isReplacing);
  const step = (by: -1 | 1) => {
    useScriptSearch.getState().words?.step(by);
  };
  return (
    <>
      <SearchField
        aria-label={t("editor-island-script-search")}
        className="w-44"
        isDisabled={isDisabled}
        onChange={(next) => {
          useScriptSearch.setState({ query: next });
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === "ArrowDown") {
            event.preventDefault();
            step(event.key === "Enter" && event.shiftKey ? -1 : 1);
          } else if (event.key === "ArrowUp") {
            event.preventDefault();
            step(-1);
          } else if (event.key === "Escape" && query === "")
            useScriptSearch.getState().words?.focus();
        }}
        placeholder={t("editor-island-script-search")}
        value={query}
      />
      <IconButton
        aria-expanded={isReplacing}
        aria-label={t("editor-island-script-replace")}
        className="text-fg-secondary enabled:hover:text-fg"
        isDisabled={isDisabled}
        onPress={() => {
          useScriptSearch.setState({ isReplacing: !isReplacing });
        }}
      >
        <ReplaceAll aria-hidden />
      </IconButton>
    </>
  );
}
