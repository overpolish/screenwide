// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  hasMusicFilters,
  type MusicFilters,
  musicFiltersKey,
  NO_MUSIC_FILTERS,
} from "./music-filters";
import {
  failureOf,
  isSearchedOnline,
  type OpenverseFailure,
  type OpenverseFound,
  type OpenverseKind,
  searchOpenverse,
} from "./openverse-api";

// A search of Openverse for what the library's search field holds, asked
// for only when pressed, so the searches everyone shares go only where they
// are wanted, and a page at a time after that. Asked for, it follows the
// kind shown and the music filters until the words change. A filter is only
// ever set by choosing it, which asks for the search itself, so filters with
// no words are searched without a press.

type Searched = {
  failure: OpenverseFailure | null;
  found: OpenverseFound[];
  isSearching: boolean;
  page: number;
  pageCount: number;
};

/** An online search as the Library island shows it. */
export type OpenverseSearch = {
  /** Whether there is anything to search online for. */
  canSearch: boolean;
  failure: OpenverseFailure | null;
  found: OpenverseFound[];
  hasMore: boolean;
  /** Whether a search online has been asked for these words. */
  hasSearched: boolean;
  isSearching: boolean;
  /** Asks for the next page, or again for the one that failed. */
  more: () => void;
  /** Asks for a search online of these words. */
  search: () => void;
};

export function useOpenverseSearch(
  kind: OpenverseKind,
  query: string,
  musicFilters: MusicFilters,
): OpenverseSearch {
  const words = query.trim().toLowerCase();
  const filters = kind === "music" ? musicFilters : NO_MUSIC_FILTERS;
  const isFiltered = hasMusicFilters(filters);
  const key = `${kind}\n${words}\n${musicFiltersKey(filters)}`;
  // The words an online search was asked for: it then follows the kind
  // shown, so turning from sound effects to music finds music for them.
  const [askedWords, setAskedWords] = useState<string | null>(null);
  // Each kind's results for each search, kept while the island is, so
  // turning back to a kind shows what it found without asking again.
  const [searches, setSearches] = useState<ReadonlyMap<string, Searched>>(
    () => new Map(),
  );
  const shown = searches.get(key) ?? null;
  const canSearch = (words.length > 0 || isFiltered) && isSearchedOnline(kind);
  const isAsked =
    canSearch && (askedWords === words || (words.length === 0 && isFiltered));
  const settle = (asked: string, searched: Searched) => {
    setSearches((now) => new Map(now).set(asked, searched));
  };
  const ask = (page: number, before: Searched | null) => {
    const asked = key;
    const searching = {
      failure: null,
      found: before?.found ?? [],
      isSearching: true,
      page: page - 1,
      pageCount: before?.pageCount ?? 0,
    };
    settle(asked, searching);
    searchOpenverse({ filters, kind, page, query })
      .then(({ found, pageCount }) => {
        settle(asked, {
          failure: null,
          found: [...searching.found, ...found],
          isSearching: false,
          page,
          pageCount,
        });
      })
      .catch((cause: unknown) => {
        settle(asked, {
          ...searching,
          failure: failureOf(cause),
          isSearching: false,
        });
      });
  };
  const askRef = useRef(ask);
  askRef.current = ask;
  const isMissing = isAsked && shown === null;
  useEffect(() => {
    if (isMissing) askRef.current(1, null);
  }, [isMissing, key]);
  return {
    canSearch,
    failure: shown?.failure ?? null,
    found: shown?.found ?? [],
    hasMore: shown !== null && shown.page < shown.pageCount,
    hasSearched: isAsked,
    isSearching: shown?.isSearching ?? isAsked,
    more: () => {
      ask((shown?.page ?? 0) + 1, shown);
    },
    search: () => {
      setAskedWords(words);
    },
  };
}
