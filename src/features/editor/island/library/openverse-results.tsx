// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchX } from "lucide-react";
import { useState } from "react";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { IslandSwap } from "../../../../primitives/island/island-swap";
import { ProgressBar } from "../../../../primitives/progress-bar/progress-bar";
import { Text } from "../../../../primitives/text/text";
import { IslandLineButton } from "../island-line-button";

import {
  fetchOpenversePicture,
  fetchOpenverseSound,
  type OpenverseFound,
} from "./openverse-api";
import { OpenverseFailureAlert } from "./openverse-failure-alert";
import {
  type FetchProgress,
  FoundPictureTile,
  FoundSoundRow,
} from "./openverse-found";
import { useFoundPreview } from "./use-found-preview";

import type { LibraryKind } from "./library-kind";
import type { OpenverseSearch } from "./use-openverse-search";

// Under the library's own, what Openverse holds for the words searched and,
// for music, the filters set, called a search online, since few know
// Openverse by name: asked for with a press, then shown as the library's own
// are, a page at a time. A find is heard from where it is published;
// downloaded, credited to who made it, it moves up into the library's own,
// to be added or dragged out from there. Only what may be used with no more
// than a credit is found (`workers/openverse`).

export function OpenverseResults({
  kind,
  onFetched,
  query,
  search,
}: {
  kind: LibraryKind;
  /** Hands on the library's name for what was downloaded into it. */
  onFetched: (name: string) => void;
  /** The words searched for. */
  query: string;
  search: OpenverseSearch;
}) {
  const preview = useFoundPreview();
  const [fetching, setFetching] = useState<ReadonlyMap<string, FetchProgress>>(
    () => new Map(),
  );
  // Downloaded, a find moves up into the library's own, so it leaves here.
  const [fetched, setFetched] = useState<ReadonlySet<string>>(() => new Set());
  // What could not be downloaded, and whether it was because it is no
  // longer offered under a licence the app may use.
  const [failed, setFailed] = useState<{
    found: OpenverseFound;
    isWithdrawn: boolean;
  } | null>(null);
  const fetch = (found: OpenverseFound) => {
    if (fetching.has(found.id)) return;
    setFailed(null);
    const progressed = (progress: FetchProgress | undefined) => {
      setFetching((now) => {
        const next = new Map(now);
        if (progress === undefined) next.delete(found.id);
        else next.set(found.id, progress);
        return next;
      });
    };
    progressed(null);
    const name =
      kind === "image"
        ? fetchOpenversePicture(found, progressed)
        : fetchOpenverseSound(found, kind, progressed).then(
            (sound) => sound.name,
          );
    name
      .then((fetchedName) => {
        preview.stopIf(found);
        setFetched((now) => new Set(now).add(found.id));
        onFetched(fetchedName);
      })
      .catch((cause: unknown) => {
        console.error("Could not download what was found online", cause);
        setFailed({ found, isWithdrawn: cause === "withdrawn" });
      })
      .finally(() => {
        progressed(undefined);
      });
  };
  const shown = search.found.filter((found) => !fetched.has(found.id));
  if (!search.canSearch) return null;
  const isFirstPageComing = search.isSearching && !search.hasMore;
  if (!search.hasSearched)
    return (
      <IslandLineButton onPress={search.search}>
        <span className="min-w-0 truncate">
          {t("editor-island-openverse-search", { query: query.trim() })}
        </span>
      </IslandLineButton>
    );
  return (
    <section
      aria-label={t("editor-island-openverse-heading")}
      className="flex flex-col gap-inline"
    >
      {/* While the first page comes, the heading says so and carries the
          bar, then gives way to Found online as the answer comes; a next
          page's bar comes at the end of More results, as the heading may
          have scrolled away by then. */}
      <IslandParts
        className="flex items-center justify-between gap-separate ps-inset-sm"
        direction="row"
      >
        <IslandSwap
          initial={false}
          key="heading"
          swapKey={isFirstPageComing ? "searching" : "found"}
        >
          <Text className="text-fg-secondary" variant="caption">
            {isFirstPageComing
              ? t("editor-island-openverse-searching-heading")
              : t("editor-island-openverse-heading")}
          </Text>
        </IslandSwap>
        {isFirstPageComing ? <SearchingBar key="bar" /> : null}
      </IslandParts>
      {kind === "image" ? (
        <ul className="grid grid-cols-4 gap-1">
          {shown.map((found) => (
            <FoundPictureTile
              fetching={fetching.get(found.id)}
              found={found}
              key={found.id}
              onFetch={() => {
                fetch(found);
              }}
            />
          ))}
        </ul>
      ) : (
        <ul className="flex flex-col">
          {shown.map((found) => (
            <FoundSoundRow
              fetching={fetching.get(found.id)}
              found={found}
              isPlaying={preview.isPlaying(found)}
              key={found.id}
              onFetch={() => {
                fetch(found);
              }}
              onTogglePlay={() => {
                preview.toggle(found);
              }}
            />
          ))}
        </ul>
      )}
      {search.failure ? (
        <OpenverseFailureAlert failure={search.failure} onRetry={search.more} />
      ) : null}
      {failed ? (
        <Alert color="error">
          {failed.isWithdrawn
            ? t("editor-island-openverse-withdrawn", {
                title: failed.found.title,
              })
            : t("editor-island-openverse-add-failed", {
                title: failed.found.title,
              })}
        </Alert>
      ) : null}
      {/* Standing in for the results, it keeps the room a row would, as
          the library's own Nothing matches does. */}
      {!search.isSearching && !search.failure && search.found.length === 0 ? (
        <Alert className="py-inset" icon={<SearchX />}>
          {t("editor-island-openverse-none")}
        </Alert>
      ) : null}
      {search.hasMore && !search.failure ? (
        <IslandLineButton
          className="justify-between"
          isDisabled={search.isSearching}
          onPress={search.more}
        >
          {t("editor-island-openverse-more")}
          {search.isSearching ? <SearchingBar /> : null}
        </IslandLineButton>
      ) : null}
    </section>
  );
}

function SearchingBar() {
  return (
    <ProgressBar
      aria-label={t("editor-island-openverse-searching")}
      className="w-24"
      isIndeterminate
    />
  );
}
