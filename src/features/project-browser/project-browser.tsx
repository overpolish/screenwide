// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clock, Folder, Trash2 } from "lucide-react";
import { useState } from "react";

import { WindowHeader } from "../../composites/window-header/window-header";
import { WindowShell } from "../../composites/window-shell/window-shell";
import { t } from "../../i18n/i18n";
import { Button } from "../../primitives/button/button";
import { SearchField } from "../../primitives/input-fields/search-field";

import { ProjectListPane } from "./list/project-list-pane";
import { ListMessage } from "./list/project-list-placeholders";
import { ProjectBrowserSidebar } from "./project-browser-sidebar";
import { createPointedCard } from "./storage/pointed-card";
import { type GoToCard } from "./storage/use-go-to-card";
import {
  DELETED,
  RECENT,
  type CopyingProjects,
  type DeletedProjects,
  type ProjectLocation,
  type ProjectSummary,
  type ScrubStrip,
} from "./types";

import type { ProjectReading } from "./activity/project-reading";
import type { KindFilter } from "./list/project-list";

export type ProjectBrowserProps = {
  error: string | null;
  locations: ProjectLocation[];
  onAddLocation: () => void;
  onClose: () => void;
  /** Sets projects aside in Recently Deleted. */
  onDelete: (files: string[]) => void;
  /** Sends everything in Recently Deleted on to the Trash. */
  onEmpty: () => void;
  onForget: (file: string) => void;
  onMinimize: () => void;
  onOpen: (file: string) => void;
  onOpenFile: () => void;
  onOpenLocation: (path: string) => void;
  onRemoveLocation: (path: string) => void;
  /** Resolves once the project is renamed, and rejects when it is not. */
  onRename: (file: string, title: string) => Promise<void>;
  /** Puts projects in Recently Deleted back where they came from. */
  onRestore: (files: string[]) => void;
  onReveal: (file: string) => void;
  onSelect: (selected: string) => void;
  /** Sends projects in Recently Deleted on to the Trash now. */
  onTrash: (files: string[]) => void;
  /** Null while the selected source is being read. */
  projects: ProjectSummary[] | null;
  /** What happened in each recording, by manifest; null for nothing. */
  readings: Record<string, ProjectReading | null>;
  /** The sidebar's selection: `RECENT`, `DELETED`, or a location's path. */
  selected: string;
  /** Each project's still by its manifest, once there is one. */
  thumbnails: Record<string, string | null>;
  /** A move or duplicate under way, while it copies. */
  copying?: CopyingProjects | null;
  /** Shown while the last delete can still be undone from the island. */
  deleted?: DeletedProjects | null;
  /** The search on opening, for a story. */
  initialQuery?: string;
  /** Absent while projects cannot be duplicated, and then not offered. */
  onDuplicate?: (files: string[]) => void;
  /** Absent while projects cannot be moved, and then not offered. */
  onMove?: (files: string[], location: string) => void;
  /** Asks for a recording's scrub strip when its card is first pointed at. */
  onRequestScrubStrip?: (file: string) => void;
  /** Puts the last projects deleted back. */
  onUndoDelete?: () => void;
  /** Each recording's scrub strip, by manifest: null for one without. */
  scrubStrips?: Record<string, ScrubStrip | null>;
};

/** The projects window: recent, folder and deleted projects, searchable by
 * what happened in them. */
export function ProjectBrowser({
  copying,
  deleted,
  error,
  initialQuery = "",
  locations,
  onAddLocation,
  onClose,
  onDelete,
  onDuplicate,
  onEmpty,
  onForget,
  onMinimize,
  onMove,
  onOpen,
  onOpenFile,
  onOpenLocation,
  onRemoveLocation,
  onRename,
  onRequestScrubStrip,
  onRestore,
  onReveal,
  onSelect,
  onTrash,
  onUndoDelete,
  projects,
  readings,
  scrubStrips,
  selected,
  thumbnails,
}: ProjectBrowserProps) {
  // The search and the kind filter hold across sources; the choice of
  // projects does not.
  const [query, setQuery] = useState(initialQuery);
  // When the query was last typed, which a date in it is read from, so "an
  // hour ago" is the hour up to then.
  const [queriedAt, setQueriedAt] = useState(() => new Date());
  const [kind, setKind] = useState<KindFilter>("all");
  // The project pointed at in the storage strip, which lights its card, and
  // the card pointed at, which lights its segment.
  const [stripPointed, setStripPointed] = useState<string | null>(null);
  const [pointedCard] = useState(createPointedCard);
  const [goTo, setGoTo] = useState<GoToCard | null>(null);
  const location = locations.find(({ path }) => path === selected);
  const title =
    selected === DELETED
      ? t("project-browser-recently-deleted")
      : (location?.name ?? t("project-browser-recent"));

  const empty =
    selected === DELETED ? (
      <ListMessage
        glyph={<Trash2 />}
        title={t("project-browser-empty-deleted-title")}
      />
    ) : location ? (
      <ListMessage
        glyph={<Folder />}
        title={t("project-browser-empty-title")}
      />
    ) : (
      <ListMessage
        action={
          <Button onPress={onOpenFile}>{t("project-browser-open-file")}</Button>
        }
        glyph={<Clock />}
        title={t("project-browser-empty-title")}
      />
    );

  return (
    <WindowShell
      header={
        <WindowHeader
          actions={
            <>
              <SearchField
                aria-label={t("project-browser-search-label")}
                className="w-56"
                onChange={(text) => {
                  setQuery(text);
                  setQueriedAt(new Date());
                }}
                placeholder={t("project-browser-search-placeholder")}
                value={query}
              />
              <Button onPress={onOpenFile} variant="ghost">
                {t("project-browser-open-file")}
              </Button>
            </>
          }
          onClose={onClose}
          onMinimize={onMinimize}
          title={title}
        />
      }
    >
      {/* Only the sidebar keeps the window inset; the projects run to
              the window's end and bottom edges so the scroll fade lands
              there. */}
      <div className="flex min-h-0 grow gap-group ps-inset">
        <ProjectBrowserSidebar
          linked={stripPointed}
          locations={locations}
          onAddLocation={onAddLocation}
          onGoTo={(file) => {
            setGoTo({ file });
          }}
          onLink={setStripPointed}
          onOpenLocation={onOpenLocation}
          onRemoveLocation={onRemoveLocation}
          onSelect={onSelect}
          pointedCard={pointedCard}
          projects={projects ?? []}
          selected={selected}
        />
        <ProjectListPane
          actions={
            selected === DELETED
              ? { onEmpty, onRestore, onTrash }
              : {
                  onDelete,
                  onDuplicate,
                  onForget: selected === RECENT ? onForget : undefined,
                  onMove,
                  onReveal,
                }
          }
          copying={copying ?? null}
          deleted={deleted ?? null}
          empty={empty}
          error={error}
          goTo={goTo}
          isDated={selected !== RECENT && selected !== DELETED}
          key={selected}
          kind={kind}
          linked={stripPointed}
          locations={locations}
          onClearSearch={() => {
            setQuery("");
          }}
          onKindChange={setKind}
          onOpen={onOpen}
          onPoint={pointedCard.set}
          onRename={onRename}
          onRequestScrubStrip={onRequestScrubStrip}
          onUndoDelete={onUndoDelete}
          projects={projects}
          queriedAt={queriedAt}
          query={query}
          readings={readings}
          scrubStrips={scrubStrips}
          thumbnails={thumbnails}
        />
      </div>
    </WindowShell>
  );
}
