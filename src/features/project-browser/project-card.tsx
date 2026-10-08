// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis } from "lucide-react";
import { useState } from "react";

import { IconButton } from "../../components/base/button/icon-button";
import { Text } from "../../components/base/text/text";
import { EditableTitle } from "../../components/shared/editable-title/editable-title";
import { cn } from "../../lib/styling";
import {
  boundsAnchor,
  PopupMenuAnchor,
  pointerAnchor,
} from "../popup-panel/use-popup-menu";

import { ProjectCardPreview } from "./project-card-preview";

import type { ProjectSummary, ScrubStrip } from "./types";

const editedAt = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "short",
});

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

/**
 * One project: its picture, its name, and when it was last edited. Its
 * actions are in a menu, opened from the More button or a right click, the
 * way Finder offers them.
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
  return (
    <article
      aria-label={project.title}
      className={cn(
        "group/card gap-control rounded-panel bg-layer p-control flex min-w-0 flex-col inset-ring",
        isSelected
          ? "inset-ring-2 inset-ring-primary"
          : "inset-ring-layer-stroke",
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
      <div className="gap-control flex items-center">
        <div className="px-control flex min-w-0 grow flex-col">
          <Text as="h3" title={project.title}>
            {project.available && onRename ? (
              <EditableTitle
                className="truncate focus:text-clip"
                key={refusals}
                label="Project name"
                onChange={(title) => {
                  onRename(title).catch(() => {
                    setRefusals((count) => count + 1);
                  });
                }}
                title={project.title}
              />
            ) : (
              <span className="block truncate">{project.title}</span>
            )}
          </Text>
          <Text className="truncate" variant="footnote">
            {footnote ??
              (!project.available
                ? "Not available"
                : project.modifiedMs === null
                  ? null
                  : editedAt.format(project.modifiedMs))}
          </Text>
        </div>
        <IconButton
          aria-label={`More actions for ${project.title}`}
          className="shrink-0"
          onPress={(event) => {
            onMenu(boundsAnchor(event.target.getBoundingClientRect()));
          }}
        >
          <Ellipsis />
        </IconButton>
      </div>
    </article>
  );
}
