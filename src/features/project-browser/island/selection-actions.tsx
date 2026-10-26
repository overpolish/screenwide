// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { platformArg, t } from "../../../i18n/i18n";
import { IslandTextButton } from "../../../primitives/island/island-controls";
import {
  boundsAnchor,
  type PopupMenuAnchor,
} from "../../popup-panel/use-popup-menu";

import type { ProjectActions } from "../list/use-project-menu";

/** What can be done to the chosen projects, on the island: in Recently
 * Deleted, put them back or send them on to the Trash; anywhere else, move,
 * duplicate or delete them. */
export function SelectionActions({
  actions,
  files,
  onMoveMenu,
}: {
  actions: ProjectActions;
  files: string[];
  /** Opens the locations to move the choice to, hung off its button. */
  onMoveMenu: (anchor: PopupMenuAnchor) => void;
}) {
  const moveRef = useRef<HTMLButtonElement>(null);
  if ("onRestore" in actions)
    return (
      <>
        <IslandTextButton
          color="primary"
          onPress={() => {
            actions.onRestore(files);
          }}
        >
          {t("project-browser-restore")}
        </IslandTextButton>
        <IslandTextButton
          onPress={() => {
            actions.onTrash(files);
          }}
        >
          {t("project-browser-move-to-trash", { platform: platformArg() })}
        </IslandTextButton>
      </>
    );
  const { onDuplicate, onMove } = actions;
  return (
    <>
      {onMove ? (
        <IslandTextButton
          onPress={() => {
            const button = moveRef.current;
            if (button)
              onMoveMenu(boundsAnchor(button.getBoundingClientRect()));
          }}
          ref={moveRef}
        >
          {t("project-browser-move-to")}
        </IslandTextButton>
      ) : null}
      {onDuplicate ? (
        <IslandTextButton
          onPress={() => {
            onDuplicate(files);
          }}
        >
          {t("project-browser-duplicate")}
        </IslandTextButton>
      ) : null}
      <IslandTextButton
        onPress={() => {
          actions.onDelete(files);
        }}
      >
        {t("project-browser-delete")}
      </IslandTextButton>
    </>
  );
}
