// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Copy, FolderInput, Trash2 } from "lucide-react";

import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { ProgressBar } from "../../components/base/progress-bar/progress-bar";
import { t } from "../../i18n/i18n";

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

const copyLabel = ({ count, destination, kind, title }: CopyingProjects) => {
  if (kind === "duplicate") {
    return t("project-browser-duplicating", { count, title });
  }
  return destination === null
    ? t("project-browser-moving-elsewhere", { count, title })
    : t("project-browser-moving", { count, destination, title });
};

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
  const copyText = copying ? copyLabel(copying) : "";
  return (
    <div className="gap-control me-window-inset mb-window-inset flex flex-col">
      {copying ? (
        <Alert
          icon={copying.kind === "move" ? <FolderInput /> : <Copy />}
          role="status"
        >
          <div className="gap-control flex flex-col">
            {copyText}
            <ProgressBar aria-label={copyText} value={copying.fraction * 100} />
          </div>
        </Alert>
      ) : null}
      {deleted ? (
        <Alert
          action={
            onUndoDelete ? (
              <Button onPress={onUndoDelete} variant="ghost">
                {t("project-browser-undo")}
              </Button>
            ) : null
          }
          icon={<Trash2 />}
          role="status"
        >
          {t("project-browser-deleted", {
            count: deleted.count,
            title: deleted.title,
          })}
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
