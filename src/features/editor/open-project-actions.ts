// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { deleteOpenProject, renameOpenProject } from "./editor-actions";

/** What the title bar does to the project itself: rename it, or delete it
 * to Recently Deleted, which closes the editor. The editor has no place to
 * show a failure outside the Export island, which speaks only of exports, so
 * one is logged. */
export function openProjectActions() {
  const fail = (action: string) => (cause: unknown) => {
    console.error(`Could not ${action} the project`, cause);
  };
  return {
    onDeleteProject: () => {
      deleteOpenProject().catch(fail("delete"));
    },
    onRenameProject: (title: string) => {
      renameOpenProject(title).catch(fail("rename"));
    },
  };
}
