// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clock, Folder, FolderOpen, HardDrive } from "lucide-react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { SidebarNav } from "../../components/base/sidebar-nav/sidebar-nav";
import { Text } from "../../components/base/text/text";
import { WindowHeader } from "../../components/shared/window-header/window-header";
import { WindowShell } from "../../components/shared/window-shell/window-shell";

import { ProjectCard } from "./project-card";
import { RECENT, type ProjectLocation, type ProjectSummary } from "./types";
import { useLocationMenu } from "./use-location-menu";

export type ProjectBrowserProps = {
  error: string | null;
  locations: ProjectLocation[];
  onAddLocation: () => void;
  onClose: () => void;
  onForget: (file: string) => void;
  onMinimize: () => void;
  onOpen: (file: string) => void;
  onOpenFile: () => void;
  onOpenLocation: (path: string) => void;
  onRemoveLocation: (path: string) => void;
  /** Resolves once the project is renamed, and rejects when it is not. */
  onRename: (file: string, title: string) => Promise<void>;
  onReveal: (file: string) => void;
  onSelect: (selected: string) => void;
  onTrash: (file: string) => void;
  /** Null while the selected source is being read. */
  projects: ProjectSummary[] | null;
  /** The sidebar's selection: `RECENT`, or a location's path. */
  selected: string;
  /** Each project's still by its manifest, once there is one. */
  thumbnails: Record<string, string | null>;
};

function EmptyState({ location }: { location: ProjectLocation | undefined }) {
  const [title, detail] = !location
    ? [
        "No projects",
        "Recordings and screenshots you make or open appear here.",
      ]
    : location.available
      ? ["No projects", "Projects saved in this folder appear here."]
      : [
          "Not available",
          "Connect the drive this folder is on to see its projects.",
        ];
  return (
    <div className="gap-control pr-window-inset pb-window-inset flex grow flex-col items-center justify-center text-center">
      <Text variant="headline">{title}</Text>
      <Text variant="subheadline">{detail}</Text>
    </div>
  );
}

/**
 * The projects the app knows: the recent ones, wherever they are, and the
 * ones in the projects folder and any folder added to the sidebar. A folder
 * in the sidebar is opened or taken off it from its right-click menu.
 */
export function ProjectBrowser({
  error,
  locations,
  onAddLocation,
  onClose,
  onForget,
  onMinimize,
  onOpen,
  onOpenFile,
  onOpenLocation,
  onRemoveLocation,
  onRename,
  onReveal,
  onSelect,
  onTrash,
  projects,
  selected,
  thumbnails,
}: ProjectBrowserProps) {
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
            <Button onPress={onOpenFile} variant="ghost">
              <FolderOpen />
              Open File
            </Button>
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
              // A folder that is not there stays selectable, so its menu can
              // still take it off the sidebar.
              ...locations.map((item) => ({
                icon: item.isDefault ? <Folder /> : <HardDrive />,
                id: item.path,
                label: item.name,
              })),
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
          {projects?.length === 0 || location?.available === false ? (
            <EmptyState location={location} />
          ) : (
            <ScrollArea
              edgeEffect="shadow"
              key={selected}
              rootClassName="min-h-0 min-w-0 grow"
            >
              <div className="gap-section pr-window-inset pb-window-inset grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))]">
                {projects?.map((project) => (
                  <ProjectCard
                    key={project.file}
                    onForget={
                      selected === RECENT
                        ? () => {
                            onForget(project.file);
                          }
                        : undefined
                    }
                    onOpen={() => {
                      onOpen(project.file);
                    }}
                    onRename={(title) => onRename(project.file, title)}
                    onReveal={() => {
                      onReveal(project.file);
                    }}
                    onTrash={() => {
                      onTrash(project.file);
                    }}
                    project={project}
                    thumbnail={thumbnails[project.file] ?? null}
                  />
                ))}
              </div>
            </ScrollArea>
          )}
          {error ? (
            <Alert
              className="mr-window-inset mb-window-inset"
              color="error"
              role="alert"
            >
              {error}
            </Alert>
          ) : null}
        </div>
      </div>
    </WindowShell>
  );
}
