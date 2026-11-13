// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchX } from "lucide-react";
import { useState } from "react";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { Button } from "../../../../primitives/button/button";
import { Text } from "../../../../primitives/text/text";
import { IslandLineButton } from "../island-line-button";
import { IslandWorkLine } from "../island-work-line";

import {
  fetchOpenversePicture,
  fetchOpenverseSound,
  isSearchedOnline,
  type OpenverseFound,
} from "./openverse-api";
import {
  type FetchProgress,
  FoundPictureTile,
  FoundSoundRow,
} from "./openverse-found";
import { useFoundPreview } from "./use-found-preview";
import { useOpenverseSearch } from "./use-openverse-search";

import type { LibraryKind } from "./library-kind";

// Under the library's own, what Openverse holds for the words searched,
// called a search online, since few know Openverse by name: asked for with
// a press, then shown as the library's own are, a page at a time. A find is
// heard from where it is published; downloaded, credited to who made it, it
// moves up into the library's own, to be added or dragged out from there.
// Only what may be used with no more than a credit is found
// (`workers/openverse`).

export function OpenverseResults({
  kind,
  onFetched,
  query,
}: {
  kind: LibraryKind;
  /** Hands on the library's name for what was downloaded into it. */
  onFetched: (name: string) => void;
  /** The words searched for. */
  query: string;
}) {
  const search = useOpenverseSearch(kind, query);
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
  if (query.trim().length === 0 || !isSearchedOnline(kind)) return null;
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
      <Text className="px-inset-sm text-fg-secondary" variant="caption">
        {t("editor-island-openverse-heading")}
      </Text>
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
      {search.isSearching ? (
        <IslandWorkLine
          label={t("editor-island-openverse-searching")}
          value={null}
        />
      ) : null}
      {search.failure ? (
        <Alert
          action={
            <Button onPress={search.more}>
              {t("editor-island-openverse-retry")}
            </Button>
          }
          color="error"
        >
          {search.failure === "busy"
            ? t("editor-island-openverse-busy")
            : t("editor-island-openverse-unavailable")}
        </Alert>
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
      {!search.isSearching && !search.failure && search.found.length === 0 ? (
        <Alert icon={<SearchX />}>{t("editor-island-openverse-none")}</Alert>
      ) : null}
      {search.hasMore && !search.isSearching && !search.failure ? (
        <IslandLineButton onPress={search.more}>
          {t("editor-island-openverse-more")}
        </IslandLineButton>
      ) : null}
    </section>
  );
}
