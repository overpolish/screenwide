// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Clock,
  Folder,
  FolderOpen,
  FolderRoot,
  HardDrive,
  Trash2,
} from "lucide-react";
import { useState } from "react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Button } from "../../components/base/button/button";
import { SearchField } from "../../components/base/input-fields/search-field";
import { SidebarNav } from "../../components/base/sidebar-nav/sidebar-nav";
import { Text } from "../../components/base/text/text";
import { WindowHeader } from "../../components/shared/window-header/window-header";
import { WindowShell } from "../../components/shared/window-shell/window-shell";

import {
  ProjectBrowserNotices,
  type CopyingProjects,
  type DeletedProjects,
} from "./project-browser-notices";
import { ProjectListPane } from "./project-list-pane";
import {
  DELETED,
  locationGlyph,
  RECENT,
  type ProjectLocation,
  type ProjectSummary,
  type ScrubStrip,
} from "./types";
import { useLocationMenu } from "./use-location-menu";

import type { KindFilter } from "./project-list";

const locationGlyphs = {
  folder: Folder,
  "folder-root": FolderRoot,
  "hard-drive": HardDrive,
};

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
  /** The sidebar's selection: `RECENT`, `DELETED`, or a location's path. */
  selected: string;
  /** Each project's still by its manifest, once there is one. */
  thumbnails: Record<string, string | null>;
  /** A move or duplicate under way, while it copies. */
  copying?: CopyingProjects | null;
  /** Shown while the last delete can still be undone from its notice. */
  deleted?: DeletedProjects | null;
  /** Absent while projects cannot be duplicated, and then not offered. */
  onDuplicate?: (files: string[]) => void;
  /** Absent while projects cannot be moved, and then not offered. */
  onMove?: (files: string[], location: string) => void;
  /** Asks for a recording's scrub strip, the first time its card is
   * pointed at. */
  onRequestScrubStrip?: (file: string) => void;
  /** Puts the last projects deleted back. */
  onUndoDelete?: () => void;
  /** Each recording's scrub strip, by manifest: null for one without. */
  scrubStrips?: Record<string, ScrubStrip | null>;
};

function EmptyState({
  location,
  selected,
}: {
  location: ProjectLocation | undefined;
  selected: string;
}) {
  return (
    <div className="gap-control pr-window-inset pb-window-inset flex grow flex-col items-center justify-center text-center">
      <Text variant="headline">
        {selected === DELETED ? "No deleted projects" : "No projects"}
      </Text>
      <Text variant="subheadline">
        {selected === DELETED
          ? "Projects you delete wait here for 30 days."
          : location
            ? "Projects saved in this folder appear here."
            : "Recordings and screenshots you make or open appear here."}
      </Text>
    </div>
  );
}

/**
 * The projects the app knows: the recent ones, wherever they are, the ones
 * in the projects folder and any folder added to the sidebar, and the ones
 * deleted in the last 30 days. A folder in the sidebar is opened or taken
 * off it from its right-click menu. The search and the kind filter hold
 * across folders; the choice of projects does not.
 */
export function ProjectBrowser({
  copying,
  deleted,
  error,
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
  scrubStrips,
  selected,
  thumbnails,
}: ProjectBrowserProps) {
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<KindFilter>("all");
  const location = locations.find(({ path }) => path === selected);
  const showLocationMenu = useLocationMenu({
    onOpen: onOpenLocation,
    onRemove: onRemoveLocation,
  });

  return (
    <WindowShell
      header={
        <WindowHeader
          actions={
            <div className="gap-control flex items-center">
              <SearchField
                aria-label="Search projects"
                className="w-48"
                onChange={setQuery}
                placeholder="Search"
                value={query}
              />
              <Button onPress={onOpenFile} variant="ghost">
                <FolderOpen />
                Open File
              </Button>
            </div>
          }
          leadingSection={
            <img
              alt="Screenwide"
              className="brightness-0 dark:invert"
              draggable={false}
              src={logoUrl}
            />
          }
          onClose={onClose}
          onMinimize={onMinimize}
          title="Projects"
        />
      }
    >
      <div className="gap-layout pl-window-inset flex min-h-0 grow">
        <div className="gap-section pb-window-inset flex shrink-0 flex-col">
          <SidebarNav
            aria-label="Project locations"
            className="grow"
            isExpandable={false}
            isExpanded
            items={[
              { icon: <Clock />, id: RECENT, label: "Recent" },
              // A folder that is not there, such as one on a drive that is
              // not connected, is shown but cannot be chosen; its menu can
              // still take it off the sidebar.
              ...locations.map((item) => {
                const Glyph = locationGlyphs[locationGlyph(item)];
                return {
                  icon: <Glyph />,
                  id: item.path,
                  isDisabled: !item.available,
                  label: item.name,
                };
              }),
              {
                icon: <Trash2 />,
                id: DELETED,
                isPinned: true,
                label: "Recently Deleted",
              },
            ]}
            onItemContextMenu={(id, point) => {
              const clicked = locations.find(({ path }) => path === id);
              if (clicked) void showLocationMenu(clicked, point);
            }}
            onSelectionChange={onSelect}
            selected={selected}
          />
          <Button
            className="self-start"
            onPress={onAddLocation}
            variant="ghost"
          >
            Add Location
          </Button>
        </div>
        <div className="gap-section flex min-h-0 min-w-0 grow flex-col">
          {projects?.length === 0 ? (
            <EmptyState location={location} selected={selected} />
          ) : projects ? (
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
              key={selected}
              kind={kind}
              locations={locations}
              onKindChange={setKind}
              onOpen={onOpen}
              onRename={onRename}
              onRequestScrubStrip={onRequestScrubStrip}
              projects={projects}
              query={query}
              scrubStrips={scrubStrips}
              thumbnails={thumbnails}
            />
          ) : (
            <div className="grow" />
          )}
          <ProjectBrowserNotices
            copying={copying ?? null}
            deleted={deleted ?? null}
            error={error}
            onUndoDelete={onUndoDelete}
          />
        </div>
      </div>
    </WindowShell>
  );
}
