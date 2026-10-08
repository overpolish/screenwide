// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { deleteOpenProject, renameOpenProject } from "./editor-actions";

/** What the title bar does to the project itself: rename it, or delete it
 * to Recently Deleted, which closes the editor. A failure is shown in the
 * window. */
export function openProjectActions(setError: (message: string) => void) {
  const fail = (cause: unknown) => {
    setError(cause instanceof Error ? cause.message : String(cause));
  };
  return {
    onDeleteProject: () => {
      deleteOpenProject().catch(fail);
    },
    onRenameProject: (title: string) => {
      renameOpenProject(title).catch(fail);
    },
  };
}
