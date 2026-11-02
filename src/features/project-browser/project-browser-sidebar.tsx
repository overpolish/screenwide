// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clock, Folder, FolderRoot, HardDrive, Trash2 } from "lucide-react";

import { SidebarNav } from "../../composites/sidebar-nav/sidebar-nav";
import { t } from "../../i18n/i18n";
import { Button } from "../../primitives/button/button";

import { type PointedCard, usePointedCard } from "./storage/pointed-card";
import { StorageStrip } from "./storage/storage-strip";
import {
  DELETED,
  locationGlyph,
  RECENT,
  type ProjectLocation,
  type ProjectSummary,
} from "./types";
import { useLocationMenu } from "./use-location-menu";

const locationGlyphs = {
  folder: Folder,
  "folder-root": FolderRoot,
  "hard-drive": HardDrive,
};

/** The browser's sources down the side: Recent, the locations, Recently
 * Deleted; under them Add Location, and at the foot the disk the source's
 * projects take. A location is opened or taken off from its right-click
 * menu. */
export function ProjectBrowserSidebar({
  linked,
  locations,
  onAddLocation,
  onGoTo,
  onLink,
  onOpenLocation,
  onRemoveLocation,
  onSelect,
  pointedCard,
  projects,
  selected,
}: {
  /** The segment pointed at, whose card lights. */
  linked: string | null;
  locations: ProjectLocation[];
  onAddLocation: () => void;
  /** Goes to a project's card from its segment. */
  onGoTo: (file: string) => void;
  onLink: (file: string | null) => void;
  onOpenLocation: (path: string) => void;
  onRemoveLocation: (path: string) => void;
  onSelect: (selected: string) => void;
  /** The card the pointer is over, whose segment lights while no segment
   * is pointed at. */
  pointedCard: PointedCard;
  projects: readonly ProjectSummary[];
  selected: string;
}) {
  const pointed = usePointedCard(pointedCard);
  const showLocationMenu = useLocationMenu({
    onOpen: onOpenLocation,
    onRemove: onRemoveLocation,
  });
  return (
    <div className="flex w-48 shrink-0 flex-col gap-separate pb-inset">
      <SidebarNav
        aria-label={t("project-browser-locations")}
        className="grow"
        isExpandable={false}
        isExpanded
        items={[
          {
            icon: <Clock />,
            id: RECENT,
            label: t("project-browser-recent"),
          },
          // A folder that is not there, such as one on a drive that is not
          // connected, is shown but cannot be chosen; its menu can still
          // take it off the sidebar.
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
            label: t("project-browser-recently-deleted"),
          },
        ]}
        onItemContextMenu={(id, point) => {
          const clicked = locations.find(({ path }) => path === id);
          if (clicked) void showLocationMenu(clicked, point);
        }}
        onSelectionChange={onSelect}
        selected={selected}
      />
      <Button className="self-start" onPress={onAddLocation} variant="ghost">
        {t("project-browser-add-location")}
      </Button>
      <StorageStrip
        linked={linked ?? pointed}
        onGoTo={onGoTo}
        onLink={onLink}
        projects={projects}
      />
    </div>
  );
}
