// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fileManagerName } from "../../lib/platform";
import { PopupPanelItem } from "../popup-panel/store";
import { pointerAnchor, usePopupMenu } from "../popup-panel/use-popup-menu";

import type { ProjectLocation } from "./types";

/** One menu per location, so a selection names the folder it came from. */
const MENU_PREFIX = "project-location:";
const MENU_WIDTH = 200;

/**
 * A right click on a sidebar folder, answered with the app's own menu: open
 * it in Finder or Explorer, or, for a folder the user added, take it off the
 * sidebar. The projects folder Settings names cannot be removed here.
 */
export function useLocationMenu({
  onOpen,
  onRemove,
}: {
  onOpen: (path: string) => void;
  onRemove: (path: string) => void;
}) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: "Location actions",
    mode: "menu",
    onSelect: (itemId, path) => {
      if (itemId === "open") onOpen(path);
      else if (itemId === "remove") onRemove(path);
    },
    width: MENU_WIDTH,
  });

  return (location: ProjectLocation, point: { x: number; y: number }) => {
    const items: PopupPanelItem[] = [
      {
        icon: "folder-open",
        id: "open",
        label: `Open in ${fileManagerName()}`,
      },
    ];
    if (!location.isDefault) {
      items.push({ icon: "remove", id: "remove", label: "Remove Location" });
    }
    return openMenu({
      anchor: pointerAnchor(point.x, point.y),
      context: location.path,
      items,
    });
  };
}
