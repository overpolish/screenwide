// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Trash2 } from "lucide-react";

import { Button } from "../../components/base/button/button";
import { ConfirmActionButton } from "../../components/shared/confirm-action-button/confirm-action-button";
import { trashName } from "../../lib/platform";
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
          Restore
        </Button>
        <Button
          onPress={() => {
            actions.onTrash(files);
          }}
        >
          Move to {trashName()}
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
          Move To
        </Button>
      ) : null}
      <Button
        onPress={() => {
          actions.onDelete(files);
        }}
      >
        Delete
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
      armedIcon={<Check />}
      armedLabel="Confirm emptying Recently Deleted"
      idleIcon={<Trash2 />}
      idleLabel="Empty"
      isDisabled={isEmpty}
      onConfirm={actions.onEmpty}
      variant="text"
    />
  );
}
