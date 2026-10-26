// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlertCircle, Trash2, X } from "lucide-react";
import { type ReactNode } from "react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../i18n/i18n";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island } from "../../../primitives/island/island";
import {
  IslandButton,
  IslandTextButton,
} from "../../../primitives/island/island-controls";
import { IslandRow } from "../../../primitives/island/island-row";
import { Text } from "../../../primitives/text/text";
import { formatBytes } from "../../editor/duration";
import { UNDO_WINDOW_MS } from "../use-undo-delete";

import { DrainRing, IslandStatus, RollingText } from "./island-parts";

import type { CopyingProjects, DeletedProjects } from "../types";

// The browser's island: a black shape at the foot of the window, in the
// recording island's material, holding whatever is passing. Choosing
// projects brings it out with what can be done to them; a copy under way, a
// delete that can still be undone and an action that failed each take it
// over in turn, and with a choice made they hang beneath it as a second
// line. With nothing to say it shrinks to a speck and goes.

export type IslandSelection = {
  /** The buttons acting on the choice (`SelectionActions`). */
  actions: ReactNode;
  /** The disk the choice takes, so a clean-up shows what it will free. */
  bytes: number;
  count: number;
  /** Every project that can be chosen is. */
  isAll: boolean;
  onClear: () => void;
  onSelectAll: () => void;
};

export type ProjectsIslandProps = {
  copying: CopyingProjects | null;
  deleted: DeletedProjects | null;
  error: string | null;
  selection: IslandSelection | null;
  onUndoDelete?: () => void;
};

const copyLabel = ({ count, destination, kind, title }: CopyingProjects) => {
  if (kind === "duplicate") {
    return t("project-browser-duplicating", { count, title });
  }
  return destination === null
    ? t("project-browser-moving-elsewhere", { count, title })
    : t("project-browser-moving", { count, destination, title });
};

/** A row that grows with what it holds, for a status that may wrap; the
 * island's own row is a control's height. */
function StatusRow({
  children,
  end,
}: {
  children: ReactNode;
  end?: ReactNode;
}) {
  return (
    <div className="flex min-h-10 items-center gap-separate px-1">
      {children}
      {end}
    </div>
  );
}

function SelectionRow({ selection }: { selection: IslandSelection }) {
  return (
    <IslandRow
      end={
        <span className="flex items-center gap-1">
          {selection.isAll ? null : (
            <IslandTextButton onPress={selection.onSelectAll}>
              {t("project-browser-select-all")}
            </IslandTextButton>
          )}
          {selection.actions}
          <IslandButton
            aria-label={t("project-browser-deselect-all")}
            className="text-fg-secondary enabled:hover:text-fg"
            onPress={selection.onClear}
          >
            <X aria-hidden className="size-icon" />
          </IslandButton>
        </span>
      }
      start={
        <span className="flex items-baseline gap-inline ps-3" role="status">
          <Text as="span" className="tabular-nums" variant="label">
            <RollingText value={selection.count}>
              {t("project-browser-selected-count", {
                count: selection.count,
              })}
            </RollingText>
          </Text>
          {selection.bytes > 0 ? (
            <Text
              as="span"
              className="whitespace-nowrap text-fg-tertiary tabular-nums"
              variant="caption"
            >
              {formatBytes(selection.bytes)}
            </Text>
          ) : null}
        </span>
      }
    />
  );
}

export function ProjectsIsland({
  copying,
  deleted,
  error,
  onUndoDelete,
  selection,
}: ProjectsIslandProps) {
  const percent = useNumberFormatter({ style: "percent" });

  // Only the most pressing passing thing is shown: a copy holds the disk,
  // a failure needs reading, and an undo is only a shortcut.
  let notice: { key: string; node: ReactNode } | null = null;
  if (copying) {
    const label = copyLabel(copying);
    notice = {
      key: "copying",
      node: (
        <IslandStatus
          glyph={
            <CircularProgress
              aria-label={label}
              size="small"
              value={copying.fraction * 100}
            />
          }
          trailing={percent.format(copying.fraction)}
        >
          {label}
        </IslandStatus>
      ),
    };
  } else if (error) {
    notice = {
      key: "error",
      node: (
        <IslandStatus
          glyph={<AlertCircle aria-hidden className="size-icon" />}
          isError
        >
          {error}
        </IslandStatus>
      ),
    };
  } else if (deleted) {
    notice = {
      key: "deleted",
      node: (
        <IslandStatus
          glyph={
            <DrainRing
              durationMs={UNDO_WINDOW_MS}
              runKey={`${deleted.title}-${String(deleted.count)}`}
            >
              <Trash2 aria-hidden />
            </DrainRing>
          }
        >
          {t("project-browser-deleted", {
            count: deleted.count,
            title: deleted.title,
          })}
        </IslandStatus>
      ),
    };
  }
  const undo =
    notice?.key === "deleted" && onUndoDelete ? (
      <IslandTextButton color="primary" onPress={onUndoDelete}>
        {t("project-browser-undo")}
      </IslandTextButton>
    ) : null;

  const row = selection ? (
    <SelectionRow key="selection" selection={selection} />
  ) : notice ? (
    <StatusRow end={undo} key={notice.key}>
      {notice.node}
    </StatusRow>
  ) : null;
  // With a choice made, what is passing hangs beneath it rather than
  // taking its place, so the choice can still be acted on.
  const below =
    selection && notice ? (
      <div className="flex items-center gap-separate px-1 pb-1">
        {notice.node}
        {undo}
      </div>
    ) : undefined;

  return (
    // Over the foot of the list, centred in it; only the island itself
    // takes the pointer.
    <div className="pointer-events-none absolute inset-x-0 bottom-inset flex justify-center">
      <div className="pointer-events-auto">
        <Island
          aria-label={t("project-browser-island")}
          below={below}
          isShown={row !== null}
          row={row}
        />
      </div>
    </div>
  );
}
