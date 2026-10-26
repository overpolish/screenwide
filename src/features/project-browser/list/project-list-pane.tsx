// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchX } from "lucide-react";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { t } from "../../../i18n/i18n";
import { isWindows } from "../../../lib/platform";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { ownsTextEditingKeys } from "../../editor/shortcuts/keyboard-target";
import { ProjectCard } from "../card/project-card";
import { ProjectsIsland } from "../island/projects-island";
import { SelectionActions } from "../island/selection-actions";

import { ProjectGrid } from "./project-grid";
import {
  filterProjects,
  groupByDate,
  timeLeft,
  type KindFilter,
} from "./project-list";
import { ListMessage, LoadingGrid } from "./project-list-placeholders";
import { ProjectListToolbar } from "./project-list-toolbar";
import { useProjectMenu, type ProjectActions } from "./use-project-menu";
import { useProjectSelection } from "./use-project-selection";

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
  kind: KindFilter;
  locations: readonly ProjectLocation[];
  onKindChange: (kind: KindFilter) => void;
  onOpen: (file: string) => void;
  onRename: (file: string, title: string) => Promise<void>;
  /** Null while the source is being read. */
  projects: readonly ProjectSummary[] | null;
  query: string;
  thumbnails: Record<string, string | null>;
  onRequestScrubStrip?: (file: string) => void;
  onUndoDelete?: () => void;
  scrubStrips?: Record<string, ScrubStrip | null>;
};

const NONE: readonly ProjectSummary[] = [];

/**
 * One source's projects: filtered by kind and the search, grouped by when
 * they were last edited unless a search is under way, and chosen and acted
 * on one at a time or together from the island. Mounted per source, so a
 * choice made in one folder never carries into another. Recently Deleted
 * lists them newest first, ungrouped, and they cannot be opened or renamed
 * there: only restored or sent on.
 */
export function ProjectListPane({
  actions,
  copying,
  deleted,
  empty,
  error,
  kind,
  locations,
  onKindChange,
  onOpen,
  onRename,
  onRequestScrubStrip,
  onUndoDelete,
  projects: read,
  query,
  scrubStrips,
  thumbnails,
}: ProjectListPaneProps) {
  const projects = read ?? NONE;
  // Read when the source is opened, which remounts the pane, rather than on
  // every render; a window left open past midnight regroups on the next.
  const [today] = useState(() => new Date());
  const isDeleted = "onRestore" in actions;
  const kindsPresent = new Set(
    projects.flatMap(({ kind: present }) => present ?? []),
  );
  // A kind this source has none of shows everything rather than nothing,
  // and the choice comes back in a source that has some.
  const shownKind: KindFilter =
    kind !== "all" && kindsPresent.has(kind) ? kind : "all";
  const listed = filterProjects(projects, query, shownKind);
  // Search results are one list: the match is what matters, not the date.
  // Recently Deleted keeps the order things were deleted in.
  const groups: { label: string | null; projects: ProjectSummary[] }[] =
    query.trim() === "" && !isDeleted
      ? groupByDate(listed, today)
      : [{ label: null, projects: listed }];
  const shown = groups.flatMap((group) => group.projects);
  const choosable = shown.filter(({ available }) => available);
  const selection = useProjectSelection(choosable.map(({ file }) => file));
  const selected = shown.filter(({ file }) => selection.selected.has(file));
  const selectedFiles = selected.map(({ file }) => file);
  const menu = useProjectMenu(locations, actions);

  const keysRef = useRef<(event: KeyboardEvent) => void>(() => undefined);
  useEffect(() => {
    keysRef.current = (event) => {
      if (ownsTextEditingKeys(event.target)) return;
      const command = isWindows() ? event.ctrlKey : event.metaKey;
      if (command && event.key.toLowerCase() === "a") {
        event.preventDefault();
        selection.selectAll();
      } else if (event.key === "Escape" && selected.length > 0) {
        selection.clear();
      } else if (
        selected.length > 0 &&
        "onDelete" in actions &&
        (isWindows()
          ? event.key === "Delete"
          : event.metaKey && event.key === "Backspace")
      ) {
        event.preventDefault();
        actions.onDelete(selectedFiles);
      }
    };
  });
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      keysRef.current(event);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    // Clipped, not hidden: the island's drawing runs far below it, and a
    // hidden overflow can still be scrolled to by focus.
    <div className="relative flex min-h-0 min-w-0 grow flex-col gap-separate overflow-clip">
      {read === null ? (
        <LoadingGrid />
      ) : projects.length === 0 ? (
        empty
      ) : (
        <>
          <ProjectListToolbar
            kind={shownKind}
            kindsPresent={kindsPresent}
            listed={listed}
            onEmpty={"onEmpty" in actions ? actions.onEmpty : undefined}
            onKindChange={onKindChange}
          />
          {listed.length === 0 ? (
            <ListMessage
              glyph={<SearchX />}
              text={t("project-browser-no-results", { query: query.trim() })}
              title={t("project-browser-no-results-title")}
            />
          ) : (
            <ScrollArea inWindowCorner rootClassName="min-h-0 min-w-0 grow">
              {/* Room above the first row for a card to lift into. */}
              <div className="pt-1">
                <ProjectGrid groups={groups}>
                  {(project) => (
                    <ProjectCard
                      footnote={
                        project.expiresMs === null
                          ? undefined
                          : timeLeft(project.expiresMs, today.getTime())
                      }
                      isSelected={selection.selected.has(project.file)}
                      isSelectionMode={selected.length > 0}
                      onExtendSelection={() => {
                        selection.extendTo(project.file);
                      }}
                      onMenu={(anchor) => {
                        void menu.openProjectMenu(
                          project.file,
                          anchor,
                          project.available,
                        );
                      }}
                      onOpen={
                        isDeleted
                          ? undefined
                          : () => {
                              onOpen(project.file);
                            }
                      }
                      onRename={
                        isDeleted
                          ? undefined
                          : (title) => onRename(project.file, title)
                      }
                      onScrubRequest={() => {
                        onRequestScrubStrip?.(project.file);
                      }}
                      onToggleSelection={() => {
                        selection.toggle(project.file);
                      }}
                      project={project}
                      scrubStrip={scrubStrips?.[project.file]}
                      thumbnail={thumbnails[project.file] ?? null}
                    />
                  )}
                </ProjectGrid>
              </div>
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
                    actions={actions}
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
