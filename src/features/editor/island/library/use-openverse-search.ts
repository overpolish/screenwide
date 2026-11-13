// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

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
// kind shown until the words change.

type Searched = {
  failure: OpenverseFailure | null;
  found: OpenverseFound[];
  isSearching: boolean;
  page: number;
  pageCount: number;
};

export function useOpenverseSearch(kind: OpenverseKind, query: string) {
  const words = query.trim().toLowerCase();
  const key = `${kind}\n${words}`;
  // The words an online search was asked for: it then follows the kind
  // shown, so turning from sound effects to music finds music for them.
  const [askedWords, setAskedWords] = useState<string | null>(null);
  // Each kind's results for each search, kept while the island is, so
  // turning back to a kind shows what it found without asking again.
  const [searches, setSearches] = useState<ReadonlyMap<string, Searched>>(
    () => new Map(),
  );
  const shown = searches.get(key) ?? null;
  const isAsked =
    askedWords === words && words.length > 0 && isSearchedOnline(kind);
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
    searchOpenverse(kind, query, page)
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
    failure: shown?.failure ?? null,
    found: shown?.found ?? [],
    hasMore: shown !== null && shown.page < shown.pageCount,
    /** Whether a search online has been asked for these words. */
    hasSearched: isAsked,
    isSearching: shown?.isSearching ?? isAsked,
    /** Asks for the next page, or again for the one that failed. */
    more: () => {
      ask((shown?.page ?? 0) + 1, shown);
    },
    search: () => {
      setAskedWords(words);
    },
  };
}
