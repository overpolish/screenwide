// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis } from "lucide-react";
import { useState } from "react";
import { useDateFormatter } from "react-aria";

import { EditableTitle } from "../../../composites/editable-title/editable-title";
import { t } from "../../../i18n/i18n";
import { findIn } from "../../../lib/found-text";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../../primitives/button/icon-button";
import { Text } from "../../../primitives/text/text";
import { formatBytes } from "../../editor/duration";
import { beatLabel } from "../../editor/timeline/beat-clip/beat-label";
import { Minimap } from "../../editor/timeline/minimap/minimap";
import { foundText } from "../../editor/tools/transcript/transcript-found-text";
import {
  boundsAnchor,
  PopupMenuAnchor,
  pointerAnchor,
} from "../../popup-panel/use-popup-menu";
import {
  beatAt,
  lineAt,
  type ProjectReading,
} from "../activity/project-reading";
import { recordedStyle } from "../list/project-list";

import { ProjectCardPicture } from "./project-card-picture";

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
  /** Replaces the date, such as the days left before the Trash. */
  footnote?: string;
  /** Search text, marked in the name and caption. */
  found?: string;
  /** Where a search found something, in recorded seconds. */
  hits?: readonly number[];
  /** Pointed at elsewhere, such as in the storage strip. */
  isLinked?: boolean;
  onHoverChange?: (isHovered: boolean) => void;
  /** Opens it in the editor; absent, a press chooses it. */
  onOpen?: () => void;
  /** Rejects when the rename fails; absent where it cannot be renamed. */
  onRename?: (title: string) => Promise<void>;
  /** A moment to show, such as a search find, in recorded seconds. */
  previewAt?: number | null;
  /** What happened in it: undefined while it is read, null for none. */
  reading?: ProjectReading | null;
  /** Undefined until asked for; null for a recording without one. */
  scrubStrip?: ScrubStrip | null;
};

/** When it was recorded, in the style `recordedStyle` picks for it. */
function useRecordedAt() {
  const weekday = useDateFormatter({
    hour: "numeric",
    minute: "2-digit",
    weekday: "long",
  });
  const day = useDateFormatter({ day: "numeric", month: "short" });
  const date = useDateFormatter({ dateStyle: "medium" });
  return (createdMs: number) => {
    const formatter = { date, day, weekday }[
      recordedStyle(createdMs, new Date())
    ];
    return formatter.format(new Date(createdMs));
  };
}

// Actions open from More or a right click, as Finder offers them.
/** A project's picture, name, date and size on disk. */
export function ProjectCard({
  footnote,
  found,
  hits = [],
  isLinked = false,
  isSelected,
  isSelectionMode,
  onExtendSelection,
  onHoverChange,
  onMenu,
  onOpen,
  onRename,
  onScrubRequest,
  onToggleSelection,
  previewAt = null,
  project,
  reading,
  scrubStrip,
  thumbnail,
}: ProjectCardProps) {
  // A refused rename leaves the field showing what was typed; remounting it
  // puts the project's own name back.
  const [refusals, setRefusals] = useState(0);
  const [pointerAt, setPointerAt] = useState<number | null>(null);
  const whenRecorded = useRecordedAt();
  const map = reading?.map;
  const playsMs = map ? map.duration * 1000 : project.durationMs;
  const toShare = (recorded: number) =>
    map ? map.toPlayed(recorded) / Math.max(map.duration, 1e-6) : null;
  // The pointer first, then a find pointed at elsewhere, then a search's
  // first find; at rest, the still.
  const restAt =
    previewAt !== null
      ? toShare(previewAt)
      : hits.length > 0
        ? toShare(hits[0])
        : null;
  const at = pointerAt ?? restAt;
  const recordedAt =
    at !== null && map ? map.toRecorded(at * map.duration) : null;
  let caption: string | null = null;
  if (reading && recordedAt !== null) {
    const beat = beatAt(reading, recordedAt);
    caption =
      lineAt(reading, recordedAt)?.text ?? (beat ? beatLabel(beat.beat) : null);
  }
  const marked = (text: string) => {
    const match = found ? findIn(text, found) : null;
    return foundText(text, match ? [match] : undefined);
  };
  const details = !project.available
    ? t("project-browser-unavailable")
    : (footnote ??
      (project.createdMs === null ? null : whenRecorded(project.createdMs)));

  return (
    <article
      aria-label={project.title}
      className={cn(
        "group/card flex min-w-0 flex-col gap-inline",
        !project.available && "opacity-60",
      )}
      data-project-file={project.file}
      onContextMenu={(event) => {
        event.preventDefault();
        onMenu(pointerAnchor(event.clientX, event.clientY));
      }}
      onPointerEnter={() => onHoverChange?.(true)}
      onPointerLeave={() => onHoverChange?.(false)}
    >
      <ProjectCardPicture
        at={at}
        caption={caption === null ? null : marked(caption)}
        isLinked={isLinked}
        isSelected={isSelected}
        isSelectionMode={isSelectionMode}
        onPointAt={setPointerAt}
        onPress={(event) => {
          // Finder's modifiers: Shift takes a range, Command (Control on
          // Windows) one more. A plain press opens it, or chooses it where
          // it cannot be opened.
          if (event.shiftKey) onExtendSelection();
          else if (event.metaKey || event.ctrlKey || !onOpen)
            onToggleSelection();
          else onOpen();
        }}
        onScrubRequest={onScrubRequest}
        onSelectedChange={onToggleSelection}
        overview={
          reading ? (
            <Minimap
              at={at}
              beats={reading.beats.map(({ beat, span }) => ({
                ...span,
                color: beat.kind === "moment" ? beat.color : undefined,
              }))}
              finds={hits}
              isPicture
              map={reading.map}
            />
          ) : null
        }
        playsMs={playsMs}
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
                found={found}
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
                {marked(project.title)}
              </span>
            )}
          </Text>
          {/* When and how much, apart by layout rather than a dot. */}
          <div className="flex min-w-0 items-baseline justify-between gap-separate">
            <Text className="truncate tabular-nums" variant="caption">
              {details}
            </Text>
            {project.available && project.sizeBytes !== null ? (
              <Text
                className="shrink-0 text-fg-tertiary tabular-nums"
                variant="caption"
              >
                {formatBytes(project.sizeBytes)}
              </Text>
            ) : null}
          </div>
        </div>
        {/* Always there, so the actions are found without pointing at
            the card first; a right click opens the same menu. */}
        <IconButton
          aria-label={t("project-browser-more-actions", {
            title: project.title,
          })}
          className="shrink-0"
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
