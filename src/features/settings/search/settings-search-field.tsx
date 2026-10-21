// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { t } from "../../../i18n/i18n";
import { SearchField } from "../../../primitives/input-fields/search-field";

import type { SettingsSearch } from "./use-settings-search";

// The field above the sections. Command-F (Control-F on Windows) comes back
// to it from anywhere in the window; Escape clears it; Enter opens the best
// result, and the down arrow moves into the results to choose another.
export function SettingsSearchField({ search }: { search: SettingsSearch }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "f" || event.altKey || event.shiftKey) return;
      if (!(event.metaKey || event.ctrlKey)) return;
      event.preventDefault();
      ref.current?.querySelector("input")?.select();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    // As wide as the sections below it, and never wider: the field's own
    // width would otherwise widen the sidebar.
    <div className="w-0 min-w-full" ref={ref}>
      <SearchField
        aria-label={t("settings-search-label")}
        className="w-full"
        onChange={search.setQuery}
        onKeyDown={(event) => {
          if (event.key !== "ArrowDown" || !search.isActive) {
            event.continuePropagation();
            return;
          }
          search.resultsRef.current?.focus();
        }}
        onSubmit={() => {
          if (search.results.length > 0) search.open(search.results[0]);
        }}
        placeholder={t("settings-search-placeholder")}
        value={search.query}
      />
    </div>
  );
}
