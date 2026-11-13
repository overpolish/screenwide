// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { t } from "../../../../i18n/i18n";
import { SearchField } from "../../../../primitives/input-fields/search-field";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";

import { type LibraryKind, useLibraryKind } from "./library-kind";
import { LibraryPictureGrid } from "./library-picture-grid";
import { LibrarySoundList, type SoundAddition } from "./library-sound-list";
import { OpenverseResults } from "./openverse-results";

import type { ImageArt } from "../../annotations/annotations";

// What the Library island holds: the songs, sound effects and pictures
// added to any recording or screenshot, the most recent first, one kind at
// a time where it offers more than one, found by name.

export type { SoundAddition };

/** How wide the island's contents stand, in px, whatever the list holds, so
 * the island keeps its width as the search narrows it. Its height follows
 * the list, up to `max-h-80` (320px), past which the list scrolls. */
const PANEL_PX = 416;

const KIND_LABELS: Record<LibraryKind, () => string> = {
  effect: () => t("editor-island-sound-library-effects"),
  image: () => t("editor-island-library-images"),
  music: () => t("editor-island-sound-library-music"),
};

export type LibraryPanelProps = {
  /** Whether a song added now would have room after the last song. */
  hasSongRoom: boolean;
  /** What this editor's library offers, in order. */
  kinds: readonly LibraryKind[];
  /** Places a picture from the library as a new image. */
  onAddPicture: (art: ImageArt) => void;
  /** Lays a sound from the library on the timeline. */
  onAddSound: (addition: SoundAddition) => void;
};

export function LibraryPanel({
  hasSongRoom,
  kinds,
  onAddPicture,
  onAddSound,
}: LibraryPanelProps) {
  const { kind, setKind } = useLibraryKind(kinds);
  const [query, setQuery] = useState("");
  // What was downloaded from the online search stands in the library's own
  // whatever the search, since it was found by more than its title.
  const [fetched, setFetched] = useState<ReadonlySet<string>>(() => new Set());
  return (
    <div className="flex flex-col gap-inline" style={{ width: PANEL_PX }}>
      <div className="flex items-center gap-separate">
        {kinds.length > 1 ? (
          <PillGroup
            aria-label={t("editor-island-sound-library-kind")}
            className="shrink-0"
            items={kinds.map((id) => ({ id, label: KIND_LABELS[id]() }))}
            onSelectionChange={(id) => {
              const next = kinds.find((each) => each === id);
              if (next) setKind(next);
            }}
            selected={kind}
          />
        ) : null}
        <SearchField
          aria-label={t("editor-island-sound-library-search")}
          className="min-w-0 grow"
          onChange={setQuery}
          placeholder={t("editor-island-sound-library-search")}
          value={query}
        />
      </div>
      <ScrollArea constrainHeight rootClassName="max-h-80">
        <div className="flex flex-col gap-separate">
          {kind === "image" ? (
            <LibraryPictureGrid
              alsoShown={fetched}
              onAdd={onAddPicture}
              query={query}
            />
          ) : (
            <LibrarySoundList
              alsoShown={fetched}
              hasSongRoom={hasSongRoom}
              kind={kind}
              onAdd={onAddSound}
              query={query}
            />
          )}
          <OpenverseResults
            kind={kind}
            onFetched={(name) => {
              setFetched((now) => new Set(now).add(name));
            }}
            query={query}
          />
        </div>
      </ScrollArea>
    </div>
  );
}
