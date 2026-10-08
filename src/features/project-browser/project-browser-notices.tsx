// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, FolderInput, Trash2 } from "lucide-react";

import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { ProgressBar } from "../../components/base/progress-bar/progress-bar";

/** The last delete, named for the notice that offers to undo it. */
export type DeletedProjects = { count: number; title: string };

/** Projects being moved or duplicated, and how far the copy has got. */
export type CopyingProjects = {
  count: number;
  /** The location's name, for a move. */
  destination: string | null;
  /** From 0 to 1. */
  fraction: number;
  kind: "duplicate" | "move";
  /** The project's name, for a single one. */
  title: string;
};

/** "“Product demo”" for one project, "3 projects" for several. */
const named = (count: number, title: string) =>
  count === 1 ? `“${title}”` : `${String(count)} projects`;

/**
 * Below the projects: a copy under way, a delete that can still be undone,
 * and the last action that failed, in that order. Together they take the
 * window's inset.
 */
export function ProjectBrowserNotices({
  copying,
  deleted,
  error,
  onUndoDelete,
}: {
  copying: CopyingProjects | null;
  deleted: DeletedProjects | null;
  error: string | null;
  onUndoDelete: (() => void) | undefined;
}) {
  if (!copying && !deleted && !error) return null;
  const copyLabel = copying
    ? copying.kind === "move"
      ? `Moving ${named(copying.count, copying.title)} to ${copying.destination ?? "another location"}`
      : `Duplicating ${named(copying.count, copying.title)}`
    : "";
  return (
    <div className="gap-control mr-window-inset mb-window-inset flex flex-col">
      {copying ? (
        <Alert
          icon={copying.kind === "move" ? <FolderInput /> : <Copy />}
          role="status"
        >
          <div className="gap-control flex flex-col">
            {copyLabel}
            <ProgressBar
              aria-label={copyLabel}
              value={copying.fraction * 100}
            />
          </div>
        </Alert>
      ) : null}
      {deleted ? (
        <Alert
          action={
            onUndoDelete ? (
              <Button onPress={onUndoDelete} variant="ghost">
                Undo
              </Button>
            ) : null
          }
          icon={<Trash2 />}
          role="status"
        >
          Moved {named(deleted.count, deleted.title)} to Recently Deleted.
        </Alert>
      ) : null}
      {error ? (
        // Several failed actions come one to a line.
        <Alert className="whitespace-pre-line" color="error" role="alert">
          {error}
        </Alert>
      ) : null}
    </div>
  );
}
