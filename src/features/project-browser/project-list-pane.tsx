// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";
import { isWindows } from "../../lib/platform";
import { ownsTextEditingKeys } from "../editor/shortcuts/keyboard-target";

import { ProjectCard } from "./project-card";
import {
  filterProjects,
  groupByDate,
  timeLeft,
  type KindFilter,
} from "./project-list";
import { ListActions, SelectionActions } from "./project-list-actions";
import { ProjectListToolbar } from "./project-list-toolbar";
import { useProjectMenu, type ProjectActions } from "./use-project-menu";
import { useProjectSelection } from "./use-project-selection";

import type { ProjectLocation, ProjectSummary, ScrubStrip } from "./types";

export type ProjectListPaneProps = {
  actions: ProjectActions;
  kind: KindFilter;
  locations: readonly ProjectLocation[];
  onKindChange: (kind: KindFilter) => void;
  onOpen: (file: string) => void;
  onRename: (file: string, title: string) => Promise<void>;
  projects: readonly ProjectSummary[];
  query: string;
  thumbnails: Record<string, string | null>;
  onRequestScrubStrip?: (file: string) => void;
  scrubStrips?: Record<string, ScrubStrip | null>;
};

/**
 * One source's projects: filtered by kind and the search, grouped by when
 * they were last edited unless a search is under way, and chosen and acted
 * on one at a time or together. Mounted per source, so a choice made in one
 * folder never carries into another. Recently Deleted lists them newest
 * first, ungrouped, and they cannot be opened or renamed there: only
 * restored or sent on.
 */
export function ProjectListPane({
  actions,
  kind,
  locations,
  onKindChange,
  onOpen,
  onRename,
  onRequestScrubStrip,
  projects,
  query,
  scrubStrips,
  thumbnails,
}: ProjectListPaneProps) {
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
  const selection = useProjectSelection(
    shown.filter(({ available }) => available).map(({ file }) => file),
  );
  const selected = shown.filter(({ file }) => selection.selected.has(file));
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
        actions.onDelete(selected.map(({ file }) => file));
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

  const selectedFiles = selected.map(({ file }) => file);
  return (
    <div className="gap-section flex min-h-0 min-w-0 grow flex-col">
      <ProjectListToolbar
        kind={shownKind}
        kindsPresent={kindsPresent}
        listActions={
          <ListActions actions={actions} isEmpty={projects.length === 0} />
        }
        listed={listed}
        onClearSelection={selection.clear}
        onKindChange={onKindChange}
        onSelectAll={selection.selectAll}
        selectableCount={shown.filter(({ available }) => available).length}
        selected={selected}
        selectionActions={
          <SelectionActions
            actions={actions}
            files={selectedFiles}
            onMoveMenu={(anchor) =>
              void menu.openMoveMenu(selectedFiles, anchor)
            }
          />
        }
      />
      {listed.length === 0 ? (
        <div className="gap-control pr-window-inset pb-window-inset flex grow flex-col items-center justify-center text-center">
          <Text variant="headline">No results</Text>
          <Text variant="subheadline">
            No project names contain “{query.trim()}”.
          </Text>
        </div>
      ) : (
        <ScrollArea edgeEffect="shadow" rootClassName="min-h-0 min-w-0 grow">
          <div className="gap-layout pr-window-inset pb-window-inset flex flex-col">
            {groups.map((group) => (
              <section
                aria-label={group.label ?? "Search results"}
                className="gap-control flex flex-col"
                key={group.label ?? "results"}
              >
                {group.label ? (
                  <Text as="h2" className="px-control" variant="section">
                    {group.label}
                  </Text>
                ) : null}
                <div className="gap-section grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))]">
                  {group.projects.map((project) => (
                    <ProjectCard
                      footnote={
                        project.expiresMs === null
                          ? undefined
                          : timeLeft(project.expiresMs, today.getTime())
                      }
                      isSelected={selection.selected.has(project.file)}
                      isSelectionMode={selected.length > 0}
                      key={project.file}
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
                  ))}
                </div>
              </section>
            ))}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}
