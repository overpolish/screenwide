// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ListBox, ListBoxItem } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import { Text } from "../../../primitives/text/text";

import type { SettingsSearch } from "./use-settings-search";

// The results in the box a settings group uses, each laid out as a settings
// row: the setting's name over its description, and where it lives centred
// at the row's end. The list primitive is a menu's single line of text, so
// the results are their own list; the highlight follows the pointer and the
// keyboard alike, as in a menu.
export function SettingsSearchResults({ search }: { search: SettingsSearch }) {
  // With nothing to show, a way into each pane instead of an empty page.
  const isEmpty = search.results.length === 0;
  const shown = isEmpty ? search.suggestions : search.results;
  return (
    <div className="flex flex-col gap-inline">
      {isEmpty ? (
        <Text variant="caption">
          {t("settings-search-no-results", { query: search.query.trim() })}
        </Text>
      ) : null}
      <ListBox
        aria-label={t("settings-search-results")}
        className="flex flex-col rounded-panel bg-fill p-inset-sm outline-none"
        items={shown}
        onAction={(id) => {
          const entry = shown.find((result) => result.id === id);
          if (entry) search.open(entry);
        }}
        ref={search.resultsRef}
      >
        {(entry) => (
          <ListBoxItem
            className="flex cursor-default items-center gap-separate rounded-part p-inset-sm outline-none data-[focused]:bg-fill data-[hovered]:bg-fill data-[pressed]:bg-fill-strong"
            id={entry.id}
            textValue={entry.title}
          >
            <div className="flex min-w-0 flex-1 flex-col">
              <Text>{entry.title}</Text>
              {entry.description ? (
                <Text variant="caption">{entry.description}</Text>
              ) : null}
            </div>
            <Text className="shrink-0" variant="caption">
              {entry.location}
            </Text>
          </ListBoxItem>
        )}
      </ListBox>
    </div>
  );
}
