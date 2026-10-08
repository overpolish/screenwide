// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { platformArg, t } from "../../i18n/i18n";
import { PopupPanelItem } from "../popup-panel/store";
import { PopupMenuAnchor, usePopupMenu } from "../popup-panel/use-popup-menu";

import { locationGlyph, type ProjectLocation } from "./types";

/** The projects a menu acts on travel in its id as JSON, so a pick names
 * them even after the choice in the window has moved on. */
const MENU_PREFIX = "project:";
const MENU_WIDTH = 220;
const MOVE_PREFIX = "move:";

/** What can be done to projects where the browser is listing them. In
 * Recently Deleted that is putting them back or sending them on to the
 * Trash; anywhere else, the rest. */
export type ProjectActions =
  | {
      onDelete: (files: string[]) => void;
      onReveal: (file: string) => void;
      /** Absent while projects cannot be duplicated, and then not offered. */
      onDuplicate?: (files: string[]) => void;
      /** Takes a missing project off Recent; offered only there. */
      onForget?: (file: string) => void;
      /** Absent while projects cannot be moved, and then not offered. */
      onMove?: (files: string[], location: string) => void;
    }
  | {
      onEmpty: () => void;
      onRestore: (files: string[]) => void;
      onTrash: (files: string[]) => void;
    };

/** Whether the project's folder sits in this location, on either platform's
 * separator. */
const isIn = (file: string, location: string) =>
  file.startsWith(location) && /[\\/]/u.test(file.charAt(location.length));

/**
 * The actions on a project, opened from its card's More button or a right
 * click, and the move offered for a choice of several from the selection's
 * Move To button. A card's menu acts on that card alone, chosen or not:
 * actions on the choice are the toolbar's, which shows what they will touch.
 * A missing project offers only what can still be done to it: taking it off
 * Recent.
 */
export function useProjectMenu(
  locations: readonly ProjectLocation[],
  actions: ProjectActions,
) {
  const openMenu = usePopupMenu({
    idPrefix: MENU_PREFIX,
    label: t("project-browser-project-actions"),
    mode: "menu",
    onSelect: (itemId, context) => {
      const files = JSON.parse(context) as string[];
      if ("onRestore" in actions) {
        if (itemId === "restore") actions.onRestore(files);
        else if (itemId === "trash") actions.onTrash(files);
        return;
      }
      if (itemId.startsWith(MOVE_PREFIX))
        actions.onMove?.(files, itemId.slice(MOVE_PREFIX.length));
      else if (itemId === "reveal") actions.onReveal(files[0]);
      else if (itemId === "duplicate") actions.onDuplicate?.(files);
      else if (itemId === "delete") actions.onDelete(files);
      else if (itemId === "forget") actions.onForget?.(files[0]);
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });

  // A location every one of them is already in is no move at all.
  const moveItems = (files: string[]): PopupPanelItem[] =>
    "onMove" in actions && actions.onMove
      ? locations
          .filter(
            ({ available, path }) =>
              available && !files.every((file) => isIn(file, path)),
          )
          .map((location) => ({
            icon: locationGlyph(location),
            id: `${MOVE_PREFIX}${location.path}`,
            label: location.name,
            section: t("project-browser-move-to"),
          }))
      : [];

  const itemsFor = (file: string, isAvailable: boolean): PopupPanelItem[] => {
    if ("onRestore" in actions)
      return [
        { icon: "restore", id: "restore", label: t("project-browser-restore") },
        {
          icon: "trash",
          id: "trash",
          label: t("project-browser-move-to-trash", {
            platform: platformArg(),
          }),
        },
      ];
    if (!isAvailable)
      return actions.onForget
        ? [
            {
              icon: "remove",
              id: "forget",
              label: t("project-browser-remove-from-recent"),
            },
          ]
        : [];
    return [
      {
        icon: "folder-search",
        id: "reveal",
        label: t("project-browser-show-in-file-manager", {
          platform: platformArg(),
        }),
      },
      ...(actions.onDuplicate
        ? [
            {
              icon: "copy" as const,
              id: "duplicate",
              label: t("project-browser-duplicate"),
            },
          ]
        : []),
      ...moveItems([file]),
      { icon: "trash", id: "delete", label: t("project-browser-delete") },
    ];
  };

  return {
    openMoveMenu: (files: string[], anchor: PopupMenuAnchor) =>
      openMenu({
        anchor,
        context: JSON.stringify(files),
        items: moveItems(files),
      }),
    openProjectMenu: (
      file: string,
      anchor: PopupMenuAnchor,
      isAvailable: boolean,
    ) =>
      openMenu({
        anchor,
        context: JSON.stringify([file]),
        items: itemsFor(file, isAvailable),
      }),
  };
}
