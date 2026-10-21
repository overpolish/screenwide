// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ConfirmActionButton } from "../../composites/confirm-action-button/confirm-action-button";
import { platformArg, t } from "../../i18n/i18n";
import { Button } from "../../primitives/button/button";
import { boundsAnchor, PopupMenuAnchor } from "../popup-panel/use-popup-menu";

import type { ProjectActions } from "./use-project-menu";

/** What can be done to the chosen projects: in Recently Deleted, put them
 * back or send them on to the Trash; anywhere else, move or delete them. */
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
  if ("onRestore" in actions)
    return (
      <>
        <Button
          onPress={() => {
            actions.onRestore(files);
          }}
        >
          {t("project-browser-restore")}
        </Button>
        <Button
          onPress={() => {
            actions.onTrash(files);
          }}
        >
          {t("project-browser-move-to-trash", { platform: platformArg() })}
        </Button>
      </>
    );
  return (
    <>
      {actions.onMove ? (
        <Button
          onPress={(event) => {
            onMoveMenu(boundsAnchor(event.target.getBoundingClientRect()));
          }}
        >
          {t("project-browser-move-to")}
        </Button>
      ) : null}
      <Button
        onPress={() => {
          actions.onDelete(files);
        }}
      >
        {t("project-browser-delete")}
      </Button>
    </>
  );
}

/** Emptying Recently Deleted, in two presses: everything in it goes on to
 * the Trash at once. Nothing elsewhere. */
export function ListActions({
  actions,
  isEmpty,
}: {
  actions: ProjectActions;
  isEmpty: boolean;
}) {
  if (!("onRestore" in actions)) return null;
  return (
    <ConfirmActionButton
      confirmLabel={t("project-browser-empty-recently-deleted-confirm")}
      isDisabled={isEmpty}
      label={t("project-browser-empty-recently-deleted")}
      onConfirm={actions.onEmpty}
    />
  );
}
