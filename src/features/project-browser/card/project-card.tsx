// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis } from "lucide-react";
import { useState } from "react";
import { useDateFormatter } from "react-aria";

import { EditableTitle } from "../../../composites/editable-title/editable-title";
import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../../primitives/button/icon-button";
import { Text } from "../../../primitives/text/text";
import { formatBytes } from "../../editor/duration";
import {
  boundsAnchor,
  PopupMenuAnchor,
  pointerAnchor,
} from "../../popup-panel/use-popup-menu";

import { ProjectCardPreview } from "./project-card-preview";

import type { ProjectSummary, ScrubStrip } from "../types";

export type ProjectCardProps = {
  isSelected: boolean;
  /** Something is chosen, so every card shows its selection box. */
  isSelectionMode: boolean;
  /** Adds the projects between the last one chosen and this one. */
  onExtendSelection: () => void;
  /** Opens the project's actions, hung off what asked for them. */
  onMenu: (anchor: PopupMenuAnchor) => void;
  /** Asks for the strip to scrub through, which arrives as `scrubStrip`. */
  onScrubRequest: () => void;
  onToggleSelection: () => void;
  project: ProjectSummary;
  thumbnail: string | null;
  /** In place of when it was last edited, such as how long a deleted
   * project has before it goes to the Trash. */
  footnote?: string;
  /** Absent where a project cannot be opened; a plain press then chooses
   * it. */
  onOpen?: () => void;
  /** Resolves once the project is renamed, and rejects when it is not.
   * Absent where a project cannot be renamed. */
  onRename?: (title: string) => Promise<void>;
  /** Undefined until asked for; null for a recording without one. */
  scrubStrip?: ScrubStrip | null;
};

/** When it was last edited, as short as still tells it apart: the time for
 * today's, the day for this year's, the full date before that. The date
 * groups above the cards already say roughly when. */
function useEditedAt() {
  const time = useDateFormatter({ timeStyle: "short" });
  const day = useDateFormatter({ day: "numeric", month: "short" });
  const date = useDateFormatter({ dateStyle: "medium" });
  return (modifiedMs: number) => {
    const edited = new Date(modifiedMs);
    const now = new Date();
    if (edited.toDateString() === now.toDateString())
      return time.format(edited);
    return edited.getFullYear() === now.getFullYear()
      ? day.format(edited)
      : date.format(edited);
  };
}

/**
 * One project: its picture, its name, and when it was last edited with the
 * disk it takes. Its actions are in a menu, opened from the More button that
 * comes up under the pointer or a right click, the way Finder offers them.
 */
export function ProjectCard({
  footnote,
  isSelected,
  isSelectionMode,
  onExtendSelection,
  onMenu,
  onOpen,
  onRename,
  onScrubRequest,
  onToggleSelection,
  project,
  scrubStrip,
  thumbnail,
}: ProjectCardProps) {
  // A refused rename leaves the field showing what was typed; remounting it
  // puts the project's own name back.
  const [refusals, setRefusals] = useState(0);
  const editedAt = useEditedAt();
  const details =
    footnote ??
    (!project.available
      ? t("project-browser-unavailable")
      : project.modifiedMs === null
        ? null
        : project.sizeBytes === null
          ? editedAt(project.modifiedMs)
          : t("project-browser-card-details", {
              date: editedAt(project.modifiedMs),
              size: formatBytes(project.sizeBytes),
            }));
  return (
    <article
      aria-label={project.title}
      className={cn(
        "group/card flex min-w-0 flex-col gap-inline",
        !project.available && "opacity-60",
      )}
      onContextMenu={(event) => {
        event.preventDefault();
        onMenu(pointerAnchor(event.clientX, event.clientY));
      }}
    >
      <ProjectCardPreview
        isSelected={isSelected}
        isSelectionMode={isSelectionMode}
        onPress={(event) => {
          // Finder's modifiers: Shift takes a range, Command (Control on
          // Windows) one more. A plain press opens and leaves the choice as
          // it was, or chooses where there is nothing to open.
          if (event.shiftKey) onExtendSelection();
          else if (event.metaKey || event.ctrlKey || !onOpen)
            onToggleSelection();
          else onOpen();
        }}
        onScrubRequest={onScrubRequest}
        onSelectedChange={onToggleSelection}
        project={project}
        scrubStrip={scrubStrip}
        thumbnail={thumbnail}
      />
      <div className="flex min-w-0 items-start gap-inline ps-0.5">
        <div className="flex min-w-0 grow flex-col">
          <Text as="h3" title={project.title} variant="body">
            {project.available && onRename ? (
              <EditableTitle
                className="truncate focus:text-clip"
                key={refusals}
                label={t("project-browser-project-name")}
                onChange={(title) => {
                  onRename(title).catch(() => {
                    setRefusals((count) => count + 1);
                  });
                }}
                title={project.title}
              />
            ) : (
              <span className="block truncate" dir="auto">
                {project.title}
              </span>
            )}
          </Text>
          <Text className="truncate tabular-nums" variant="caption">
            {details}
          </Text>
        </div>
        {/* Out of the way until the card is pointed at or the button is
            reached from the keyboard; a right click opens the same menu. */}
        <IconButton
          aria-label={t("project-browser-more-actions", {
            title: project.title,
          })}
          className="shrink-0 opacity-0 transition-opacity group-hover/card:opacity-100 data-[focus-visible]:opacity-100"
          onPress={(event) => {
            onMenu(boundsAnchor(event.target.getBoundingClientRect()));
          }}
          variant="ghost"
        >
          <Ellipsis />
        </IconButton>
      </div>
    </article>
  );
}
