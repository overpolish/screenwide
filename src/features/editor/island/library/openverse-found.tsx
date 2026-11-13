// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Download } from "lucide-react";
import { useNumberFormatter } from "react-aria";
import { Button } from "react-aria-components";

import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { IconButton } from "../../../../primitives/button/icon-button";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import { Text } from "../../../../primitives/text/text";
import { formatDuration } from "../../duration";
import { IslandLineButton } from "../island-line-button";
import { IslandPlayButton } from "../island-play-button";

import { licenseName, type OpenverseFound } from "./openverse-api";

// What was found online, laid out as the library's own: a sound as a row,
// heard from its play button or its title, and a picture as a tile. Each
// shows who made it and its licence, which says whether an export that uses
// it will credit it. Downloaded, from the row's end or by pressing the tile,
// it shows how far it has come, then moves up into the library's own.

/** Below this, in ms, a length is shown in seconds to a tenth. */
const SHORT_MS = 10_000;

/** How far a download has come, 0 to 1; null before the host says how
 * large the file is. */
export type FetchProgress = number | null;

const fetchLabel = (found: OpenverseFound) =>
  found.creator
    ? t("editor-island-openverse-fetch-by", {
        creator: found.creator,
        license: licenseName(found.license),
        title: found.title,
      })
    : t("editor-island-openverse-fetch", {
        license: licenseName(found.license),
        title: found.title,
      });

function FetchingRing({
  found,
  progress,
}: {
  found: OpenverseFound;
  progress: FetchProgress;
}) {
  return (
    <CircularProgress
      aria-label={t("editor-island-openverse-fetching", { title: found.title })}
      isIndeterminate={progress === null}
      size="small"
      value={(progress ?? 0) * 100}
    />
  );
}

export function FoundSoundRow({
  fetching,
  found,
  isPlaying,
  onFetch,
  onTogglePlay,
}: {
  /** How far its download has come, while it is downloading. */
  fetching: FetchProgress | undefined;
  found: OpenverseFound;
  isPlaying: boolean;
  onFetch: () => void;
  onTogglePlay: () => void;
}) {
  const tenths = useNumberFormatter({
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  });
  const durationMs = found.durationMs ?? 0;
  const playLabel = isPlaying
    ? t("editor-island-sound-library-stop", { title: found.title })
    : t("editor-island-sound-library-play", { title: found.title });
  return (
    <li className="flex items-center gap-inline">
      <IslandPlayButton
        isPlaying={isPlaying}
        label={playLabel}
        onPress={onTogglePlay}
      />
      {/* The play button's twin, a wider target for the pointer; keyboard
          users reach the play button. */}
      <IslandLineButton
        aria-label={playLabel}
        className="min-w-0"
        excludeFromTabOrder
        onPress={onTogglePlay}
      >
        <span className="min-w-0 truncate" dir="auto">
          {found.title}
        </span>
        {found.creator ? (
          <span
            className="min-w-0 shrink truncate text-caption text-fg-tertiary"
            dir="auto"
          >
            {found.creator}
          </span>
        ) : null}
      </IslandLineButton>
      <Badge>{licenseName(found.license)}</Badge>
      <span className="shrink-0 text-caption whitespace-nowrap text-fg-tertiary tabular-nums">
        {durationMs < SHORT_MS
          ? t("editor-island-sound-library-seconds", {
              seconds: tenths.format(durationMs / 1_000),
            })
          : formatDuration(durationMs)}
      </span>
      {/* The ring stands where the button was, a control's size, so the row
          keeps its width as one turns into the other. */}
      <span className="flex size-control-height shrink-0 items-center justify-center">
        {fetching === undefined ? (
          <IconButton
            aria-label={fetchLabel(found)}
            className="text-fg-secondary enabled:hover:text-fg"
            onPress={onFetch}
          >
            <Download aria-hidden className="size-icon" />
          </IconButton>
        ) : (
          <FetchingRing found={found} progress={fetching} />
        )}
      </span>
    </li>
  );
}

export function FoundPictureTile({
  fetching,
  found,
  onFetch,
}: {
  /** How far its download has come, while it is downloading. */
  fetching: FetchProgress | undefined;
  found: OpenverseFound;
  onFetch: () => void;
}) {
  const isFetching = fetching !== undefined;
  return (
    <li className="group relative flex min-w-0 flex-col">
      <Button
        aria-label={fetchLabel(found)}
        className={cn(
          "relative block aspect-square w-full cursor-default rounded-control transition-colors data-[hovered]:bg-fill",
          focusStyles,
          elementFocusVisible,
        )}
        isDisabled={isFetching}
        onPress={onFetch}
      >
        <span
          className={cn(
            "absolute inset-1 transition-opacity",
            isFetching && "opacity-40",
          )}
        >
          <img
            alt=""
            className="size-full object-contain"
            decoding="async"
            draggable={false}
            loading="lazy"
            src={found.thumbnail ?? found.url}
          />
        </span>
        {isFetching ? (
          <span className="absolute inset-0 flex items-center justify-center">
            <FetchingRing found={found} progress={fetching} />
          </span>
        ) : (
          // What pressing the tile does, on the island's black so it reads
          // over any picture, as a library tile's bin does.
          <span
            aria-hidden
            className="absolute end-1 top-1 flex rounded-full bg-island p-1 text-fg-secondary opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
          >
            <Download className="size-icon-sm" />
          </span>
        )}
      </Button>
      <Text
        as="span"
        className="truncate px-1 text-center text-fg-secondary"
        dir="auto"
        variant="caption"
      >
        {found.title}
      </Text>
      {/* Who made it, then its licence, each on its own line, so a long
          name is cut short without hiding the licence. */}
      {found.creator ? (
        <Text
          as="span"
          className="truncate px-1 text-center text-fg-tertiary"
          dir="auto"
          variant="caption"
        >
          {found.creator}
        </Text>
      ) : null}
      <Text
        as="span"
        className="truncate px-1 text-center text-fg-tertiary"
        variant="caption"
      >
        {licenseName(found.license)}
      </Text>
    </li>
  );
}
