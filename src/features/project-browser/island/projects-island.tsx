// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlertCircle, Trash2, X } from "lucide-react";
import { type ReactNode } from "react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { CircularProgress } from "../../../primitives/circular-progress/circular-progress";
import { Island } from "../../../primitives/island/island";
import { IslandSwap } from "../../../primitives/island/island-swap";
import { Text } from "../../../primitives/text/text";
import { formatBytes } from "../../editor/duration";
import { IslandRowPresence } from "../../editor/island/island-row-presence";
import { UNDO_WINDOW_MS } from "../use-undo-delete";

import { DrainRing, IslandStatus, RollingText } from "./island-parts";

import type { CopyingProjects, DeletedProjects } from "../types";

// The browser's island: a black shape at the foot of the window, in the
// recording island's material, holding whatever is passing. Choosing
// projects brings it out with what can be done to them; a copy under way, a
// delete that can still be undone and an action that failed each take it
// over in turn, and with a choice made they hang beneath it as a second
// line. With nothing to say it shrinks away to nothing. It stays out while
// what it holds changes, as the editor's island does: the row and the line
// clear into their new ones (`IslandSwap`) while the black reshapes round
// them; what comes first is revealed as the island comes out, as a row's
// halves are.

type IslandSelection = {
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

/** The row's own box, laid out as an `IslandRow` without a notch, but
 * spanning the island (`rowFillsWidth`) with its parts at the two ends, as
 * the editor island's row does: as the black glides to a new width they
 * keep to its edges rather than sliding about its middle, and a part that
 * comes or goes (`IslandRowPresence`) is the one that gives way. Laid out
 * here rather than with an `IslandRow`, whose halves are revealed as they
 * mount, so a row is revealed once as a whole as it takes the island over
 * (`IslandSwap`). Its width is read without the span, as its content's. */
const ROW =
  "flex w-max max-w-full min-w-full items-center justify-between gap-separate px-1";

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
    <div className={cn(ROW, "min-h-10")}>
      {children}
      {end}
    </div>
  );
}

function SelectionRow({ selection }: { selection: IslandSelection }) {
  return (
    <div className={cn(ROW, "h-10")}>
      <span
        className="flex min-w-0 items-baseline gap-inline ps-3 *:shrink-0"
        role="status"
      >
        <Text as="span" className="tabular-nums" variant="label">
          <RollingText value={selection.count}>
            {t("project-browser-selected-count", {
              count: selection.count,
            })}
          </RollingText>
        </Text>
        <IslandRowPresence isShown={selection.bytes > 0}>
          <Text
            as="span"
            className="whitespace-nowrap text-fg-tertiary tabular-nums"
            variant="caption"
          >
            {formatBytes(selection.bytes)}
          </Text>
        </IslandRowPresence>
      </span>
      <span className="flex min-w-0 items-center gap-1 *:shrink-0">
        <IslandRowPresence isShown={!selection.isAll}>
          <Button onPress={selection.onSelectAll}>
            {t("project-browser-select-all")}
          </Button>
        </IslandRowPresence>
        {selection.actions}
        <IconButton
          aria-label={t("project-browser-deselect-all")}
          className="text-fg-secondary enabled:hover:text-fg"
          onPress={selection.onClear}
        >
          <X aria-hidden className="size-icon" />
        </IconButton>
      </span>
    </div>
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
  // a failure needs reading, and an undo is only a shortcut. Its key names
  // what it says, so a new failure or a second delete clears into the old
  // one's place rather than being written over it.
  let notice: { key: string; node: ReactNode; action?: ReactNode } | null =
    null;
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
      key: `error-${error}`,
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
    const run = `${deleted.title}-${String(deleted.count)}`;
    notice = {
      action: onUndoDelete ? (
        <Button color="primary" onPress={onUndoDelete}>
          {t("project-browser-undo")}
        </Button>
      ) : undefined,
      key: `deleted-${run}`,
      node: (
        <IslandStatus
          glyph={
            <DrainRing durationMs={UNDO_WINDOW_MS} runKey={run}>
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

  // The row's swap spans the row's box, so the row in it can span the
  // island.
  const row = selection ? (
    <IslandSwap className="min-w-0 grow" swapKey="selection">
      <SelectionRow selection={selection} />
    </IslandSwap>
  ) : notice ? (
    <IslandSwap className="min-w-0 grow" swapKey={notice.key}>
      <StatusRow end={notice.action}>{notice.node}</StatusRow>
    </IslandSwap>
  ) : null;
  // With a choice made, what is passing hangs beneath it rather than
  // taking its place, so the choice can still be acted on.
  const below =
    selection && notice ? (
      <IslandSwap swapKey={notice.key}>
        <div className="flex items-center gap-separate px-1 pb-1">
          {notice.node}
          {notice.action}
        </div>
      </IslandSwap>
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
          rowFillsWidth
          // It stands over the projects, where a speck left a moment would
          // read as a pill left behind, as on the editor's picture.
          shrinksToNothing
        />
      </div>
    </div>
  );
}
