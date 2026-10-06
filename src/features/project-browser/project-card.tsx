// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AudioLines,
  Check,
  FolderSearch,
  ImageIcon,
  Monitor,
  Trash2,
  Video,
  X,
} from "lucide-react";
import { useState } from "react";
import { Button as AriaButton } from "react-aria-components";

import { Badge } from "../../components/base/badge/badge";
import { IconButton } from "../../components/base/button/icon-button";
import { Text } from "../../components/base/text/text";
import { ConfirmActionButton } from "../../components/shared/confirm-action-button/confirm-action-button";
import { EditableTitle } from "../../components/shared/editable-title/editable-title";
import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { formatBytes, formatDuration } from "../editor/duration";

import { pictureCornerTone, type PictureTone } from "./picture-tone";

import type { ProjectKind, ProjectSummary } from "./types";

const isWindows = () => document.documentElement.dataset.platform === "windows";
const binName = () => (isWindows() ? "Recycle Bin" : "Trash");

const editedAt = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "short",
});

const kindIcons: Record<ProjectKind, typeof Monitor> = {
  audio: AudioLines,
  camera: Video,
  screen: Monitor,
  screenshot: ImageIcon,
};

/** What kind of capture it is, where that is not an ordinary recording, then
 * how long it is and how much disk it takes, as far as each is known. */
function badgesFor(project: ProjectSummary) {
  return [
    project.kind === "screenshot" ? "Screenshot" : null,
    project.replay ? "Replay" : null,
    project.durationMs === null ? null : formatDuration(project.durationMs),
    project.sizeBytes === null ? null : formatBytes(project.sizeBytes),
  ].filter((badge) => badge !== null);
}

export type ProjectCardProps = {
  onOpen: () => void;
  /** Resolves once the project is renamed, and rejects when it is not. */
  onRename: (title: string) => Promise<void>;
  onReveal: () => void;
  onTrash: () => void;
  project: ProjectSummary;
  /** A still from the project, or null while there is none to show. */
  thumbnail: string | null;
  /** Takes the project off Recent; offered there for one that is missing. */
  onForget?: () => void;
};

export function ProjectCard({
  onForget,
  onOpen,
  onRename,
  onReveal,
  onTrash,
  project,
  thumbnail,
}: ProjectCardProps) {
  const KindIcon = kindIcons[project.kind ?? "screen"];
  // A refused rename leaves the field showing what was typed; remounting it
  // puts the project's own name back.
  const [refusals, setRefusals] = useState(0);
  const isPicture = thumbnail !== null && project.kind !== "audio";
  // How light the picture is under the badges, once it has loaded; until
  // then, or with no picture, they take the window's own appearance.
  const [tone, setTone] = useState<PictureTone | null>(null);
  return (
    <article
      aria-label={project.title}
      className={cn(
        "gap-control rounded-panel bg-layer p-control inset-ring-layer-stroke flex min-w-0 flex-col inset-ring",
        !project.available && "opacity-60",
      )}
    >
      <AriaButton
        aria-label={`Open ${project.title}`}
        className={cn(
          "rounded-control text-content-fg-tertiary relative flex aspect-video cursor-default items-center justify-center",
          // A still is drawn as it is, square corners and all: rounding it
          // would show something the recording does not hold.
          !isPicture && "bg-fill overflow-hidden",
          focusStyles,
          elementFocusVisible,
        )}
        isDisabled={!project.available}
        onPress={onOpen}
      >
        {thumbnail && project.kind === "audio" ? (
          // The ribbon is drawn white; the mask takes its shape and the
          // label colour paints it, so it follows the appearance.
          <span
            className="text-content-fg-secondary size-full bg-current mask-contain mask-center mask-no-repeat"
            style={{ maskImage: `url("${thumbnail}")` }}
          />
        ) : isPicture ? (
          // Contained by the browser rather than sized from the loaded
          // still, so a card drawn before its still loads is already the
          // right shape: no stretched frame snapping to size.
          <img
            alt=""
            className="absolute inset-0 size-full object-contain"
            // Lets the corner's pixels be read; the asset protocol allows it
            // for the window's own origin.
            crossOrigin="anonymous"
            draggable={false}
            onLoad={({ currentTarget }) => {
              setTone(pictureCornerTone(currentTarget));
            }}
            src={thumbnail}
          />
        ) : (
          <KindIcon className="size-icon-large" />
        )}
        {/* Bottom-right, where file browsers and video players put a
            clip's length, in the appearance of the picture beneath them so
            they read on it. */}
        {project.available && badgesFor(project).length > 0 ? (
          <span
            className={cn(
              "gap-control right-control bottom-control absolute flex",
              isPicture && tone === "light" && "appearance-light",
              isPicture && tone === "dark" && "appearance-dark",
            )}
          >
            {badgesFor(project).map((badge) => (
              // A slight blur keeps fine detail, such as text in a
              // recording, from showing through the faint fill.
              <Badge className="backdrop-blur-sm" key={badge}>
                {badge}
              </Badge>
            ))}
          </span>
        ) : null}
      </AriaButton>
      <div className="gap-control flex items-center">
        <div className="px-control flex min-w-0 grow flex-col">
          <Text as="h3" title={project.title}>
            {project.available ? (
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
            {!project.available
              ? "Not available"
              : project.modifiedMs === null
                ? null
                : editedAt.format(project.modifiedMs)}
          </Text>
        </div>
        <div className="gap-control flex shrink-0">
          <IconButton
            aria-label={isWindows() ? "Show in Explorer" : "Show in Finder"}
            isDisabled={!project.available}
            onPress={onReveal}
          >
            <FolderSearch />
          </IconButton>
          {!project.available && onForget ? (
            // Nothing is deleted, only the entry, so it needs no confirming.
            <IconButton aria-label="Remove from Recents" onPress={onForget}>
              <X />
            </IconButton>
          ) : (
            <ConfirmActionButton
              armedIcon={<Check />}
              armedLabel={`Confirm moving to ${binName()}`}
              idleIcon={<Trash2 />}
              idleLabel={`Move to ${binName()}`}
              isDisabled={!project.available}
              onConfirm={onTrash}
              variant="icon"
            />
          )}
        </div>
      </div>
    </article>
  );
}
