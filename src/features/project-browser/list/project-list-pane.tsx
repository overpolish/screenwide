// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchX } from "lucide-react";
import { type ReactNode, useRef, useState } from "react";

import { t } from "../../../i18n/i18n";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { ProjectCard } from "../card/project-card";
import { ProjectsIsland } from "../island/projects-island";
import { SelectionActions } from "../island/selection-actions";
import { parseDateQuery } from "../search/date-query";
import { dateVocabulary } from "../search/date-reading";
import { searchByQuery } from "../search/project-search";
import { type GoToCard, useGoToCard } from "../storage/use-go-to-card";

import { filterByKind, timeLeft, type KindFilter } from "./project-list";
import { ListMessage, LoadingGrid } from "./project-list-placeholders";
import { ProjectListResults } from "./project-list-results";
import { ProjectListToolbar } from "./project-list-toolbar";
import { useCardKeys } from "./use-card-keys";
import { useProjectMenu, type ProjectActions } from "./use-project-menu";
import { clearingChoice, useProjectSelection } from "./use-project-selection";
import { useSelectionKeys } from "./use-selection-keys";

import type { ProjectReading } from "../activity/project-reading";
import type {
  CopyingProjects,
  DeletedProjects,
  ProjectLocation,
  ProjectSummary,
  ScrubStrip,
} from "../types";

export type ProjectListPaneProps = {
  actions: ProjectActions;
  copying: CopyingProjects | null;
  deleted: DeletedProjects | null;
  /** In place of the list when the source holds no projects at all. */
  empty: ReactNode;
  error: string | null;
  /** A segment pressed in the storage strip, whose card is gone to. */
  goTo: GoToCard | null;
  /** A location's, grouped by when they were recorded; elsewhere (Recent,
   * Recently Deleted) the projects keep the order given, ungrouped. */
  isDated: boolean;
  kind: KindFilter;
  /** The project pointed at in the storage strip, whose card lights up. */
  linked: string | null;
  locations: readonly ProjectLocation[];
  onClearSearch: () => void;
  onKindChange: (kind: KindFilter) => void;
  onOpen: (file: string) => void;
  /** Told the card under the pointer, or null as it leaves the cards. */
  onPoint: (file: string | null) => void;
  onRename: (file: string, title: string) => Promise<void>;
  /** Null while the source is being read. */
  projects: readonly ProjectSummary[] | null;
  /** When the query was typed, which a date in it is read from. */
  queriedAt: Date;
  query: string;
  /** What happened in each recording, by manifest, once read. */
  readings: Readonly<Record<string, ProjectReading | null>>;
  thumbnails: Record<string, string | null>;
  onRequestScrubStrip?: (file: string) => void;
  onUndoDelete?: () => void;
  scrubStrips?: Record<string, ScrubStrip | null>;
};

const NONE: readonly ProjectSummary[] = [];

/**
 * One source's projects: filtered by kind, in a location grouped by when
 * they were recorded, or, while a search is under way, the projects it found
 * with the moments in each; chosen and acted on one at a time or together
 * from the island. Mounted per source, so a choice made in one folder never
 * carries into another. Recent lists them most recently opened first and
 * Recently Deleted newest first, both ungrouped; in Recently Deleted they
 * cannot be opened or renamed: only restored or sent on.
 */
export function ProjectListPane({
  actions,
  copying,
  deleted,
  empty,
  error,
  goTo,
  isDated,
  kind,
  linked,
  locations,
  onClearSearch,
  onKindChange,
  onOpen,
  onPoint,
  onRename,
  onRequestScrubStrip,
  onUndoDelete,
  projects: read,
  queriedAt,
  query,
  readings,
  scrubStrips,
  thumbnails,
}: ProjectListPaneProps) {
  const projects = read ?? NONE;
  // Read when the source is opened, which remounts the pane, rather than on
  // every render; a window left open past midnight regroups on the next.
  const [today] = useState(() => new Date());
  const rootRef = useRef<HTMLDivElement>(null);
  const isDeleted = "onRestore" in actions;
  const kindsPresent = new Set(
    projects.flatMap(({ kind: present }) => present ?? []),
  );
  // A kind this source has none of shows everything rather than nothing,
  // and the choice comes back in a source that has some.
  const shownKind: KindFilter =
    kind !== "all" && kindsPresent.has(kind) ? kind : "all";
  const ofKind = filterByKind(projects, shownKind);
  const terms =
    query.trim() === ""
      ? null
      : {
          date: parseDateQuery(query, queriedAt, dateVocabulary()),
          text: query,
        };
  const search = terms ? searchByQuery(ofKind, readings, terms) : null;
  const shown = search ? search.projects : ofKind;
  const choosable = shown.filter(({ available }) => available);
  const selection = useProjectSelection(choosable.map(({ file }) => file));
  const selected = shown.filter(({ file }) => selection.selected.has(file));
  const selectedFiles = selected.map(({ file }) => file);
  const acting = clearingChoice(actions, selection.clear);
  const menu = useProjectMenu(locations, acting);
  const open = isDeleted ? undefined : onOpen;
  useCardKeys({ onOpen: open, rootRef });
  const goneTo = useGoToCard({
    kind: shownKind,
    onClearSearch,
    onKindChange,
    projects,
    request: goTo,
    rootRef,
    search:
      terms && ((within) => searchByQuery(within, readings, terms).projects),
    shown,
  });

  useSelectionKeys({
    actions: acting,
    onClear: selection.clear,
    onSelectAll: selection.selectAll,
    selectedFiles,
  });

  const card = (
    project: ProjectSummary,
    shownAt: { hits?: number[]; previewAt?: number | null } = {},
  ) => (
    <ProjectCard
      footnote={
        project.expiresMs === null
          ? undefined
          : timeLeft(project.expiresMs, today.getTime())
      }
      found={search?.words || undefined}
      hits={shownAt.hits}
      isLinked={linked === project.file || goneTo.marked === project.file}
      isSelected={selection.selected.has(project.file)}
      isSelectionMode={selected.length > 0}
      onExtendSelection={() => {
        selection.extendTo(project.file);
      }}
      onHoverChange={(isHovered) => {
        onPoint(isHovered ? project.file : null);
      }}
      onMenu={(anchor) => {
        void menu.openProjectMenu(project.file, anchor, project.available);
      }}
      onOpen={
        open
          ? () => {
              open(project.file);
            }
          : undefined
      }
      onRename={
        isDeleted ? undefined : (title) => onRename(project.file, title)
      }
      onScrubRequest={() => {
        onRequestScrubStrip?.(project.file);
      }}
      onToggleSelection={() => {
        selection.toggle(project.file);
      }}
      previewAt={shownAt.previewAt}
      project={project}
      reading={readings[project.file]}
      scrubStrip={scrubStrips?.[project.file]}
      thumbnail={thumbnails[project.file] ?? null}
    />
  );

  return (
    // Clipped, not hidden: the island's drawing runs far below it, and a
    // hidden overflow can still be scrolled to by focus. Its start inset is
    // room for a card's selection outline and focus ring, which reach 4px
    // outside the picture; the scroll area reaches back into it (below) so
    // its viewport does not cut them off on the first column.
    <div
      className="relative flex min-h-0 min-w-0 grow flex-col gap-separate overflow-clip ps-1"
      ref={rootRef}
    >
      {read === null ? (
        <LoadingGrid />
      ) : projects.length === 0 ? (
        empty
      ) : (
        <>
          <ProjectListToolbar
            isEmpty={shown.length === 0}
            kind={shownKind}
            kindsPresent={kindsPresent}
            onEmpty={"onEmpty" in actions ? actions.onEmpty : undefined}
            onKindChange={onKindChange}
          />
          {shown.length === 0 ? (
            <ListMessage
              glyph={<SearchX />}
              title={t("project-browser-no-results-title")}
            />
          ) : (
            <ScrollArea
              className="ps-1"
              inWindowCorner
              onInitialized={goneTo.onViewport}
              rootClassName="-ms-1 min-h-0 min-w-0 grow"
            >
              <ProjectListResults
                card={card}
                isDated={isDated}
                onOpen={open}
                projects={ofKind}
                readings={readings}
                search={search}
                today={today}
              />
            </ScrollArea>
          )}
        </>
      )}
      <ProjectsIsland
        copying={copying}
        deleted={deleted}
        error={error}
        onUndoDelete={onUndoDelete}
        selection={
          selected.length > 0
            ? {
                actions: (
                  <SelectionActions
                    actions={acting}
                    files={selectedFiles}
                    onMoveMenu={(anchor) =>
                      void menu.openMoveMenu(selectedFiles, anchor)
                    }
                  />
                ),
                bytes: selected.reduce(
                  (total, { sizeBytes }) => total + (sizeBytes ?? 0),
                  0,
                ),
                count: selected.length,
                isAll: selected.length === choosable.length,
                onClear: selection.clear,
                onSelectAll: selection.selectAll,
              }
            : null
        }
      />
    </div>
  );
}
