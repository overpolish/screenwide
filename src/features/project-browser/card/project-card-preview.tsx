// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AudioLines,
  Camera,
  FileQuestion,
  ImageIcon,
  Mic,
  Monitor,
  Video,
  Volume2,
} from "lucide-react";
import { useState } from "react";
import { useListFormatter, useLocale } from "react-aria";
import { Button as AriaButton, type PressEvent } from "react-aria-components";

import { t } from "../../../i18n/i18n";
import { motionDurationCss } from "../../../lib/motion";
import {
  pictureTones,
  type PictureCorner,
  type PictureTones,
} from "../../../lib/picture-tone";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Checkbox } from "../../../primitives/checkbox/checkbox";

import {
  type CapturedTrack,
  LengthBadges,
  ScrubPlayhead,
  TrackBadge,
} from "./project-card-overlays";

import type { ProjectKind, ProjectSummary, ScrubStrip } from "../types";

const kindIcons: Record<ProjectKind, typeof Monitor> = {
  audio: AudioLines,
  camera: Video,
  screen: Monitor,
  screenshot: ImageIcon,
};

/** In capture order, with the icons the editor's timeline and the recording
 * bar give the same sources. */
const trackIcons: CapturedTrack[] = [
  { icon: Monitor, track: "screen" },
  { icon: Camera, track: "camera" },
  { icon: Volume2, track: "systemAudio" },
  { icon: Mic, track: "microphone" },
];

const trackLabels = {
  camera: () => t("project-browser-track-camera"),
  microphone: () => t("project-browser-track-microphone"),
  screen: () => t("project-browser-track-screen"),
  systemAudio: () => t("project-browser-track-system-audio"),
};

export type ProjectCardPreviewProps = {
  isSelected: boolean;
  /** Shows the selection box without a hover, once anything is chosen. */
  isSelectionMode: boolean;
  onPress: (event: PressEvent) => void;
  /** Asks for `scrubStrip`, the first time the pointer comes over a
   * recording that has not asked yet. */
  onScrubRequest: () => void;
  onSelectedChange: (selected: boolean) => void;
  project: ProjectSummary;
  /** Frames along the edit for the pointer to move through: undefined until
   * asked for, null for a recording the editor has not composed one for. */
  scrubStrip: ScrubStrip | null | undefined;
  /** A still from the project, or null while there is none to show. */
  thumbnail: string | null;
};

/**
 * A card's picture, edge to edge in rounded corners: pressed to open the
 * project, or with Shift, Command or Control to choose it. It lifts under the
 * pointer, and moving across a recording whose editor has composed a scrub
 * strip shows the edit at that point, with a playhead where the pointer is,
 * so a take can be checked without opening it.
 */
export function ProjectCardPreview({
  isSelected,
  isSelectionMode,
  onPress,
  onScrubRequest,
  onSelectedChange,
  project,
  scrubStrip,
  thumbnail,
}: ProjectCardPreviewProps) {
  const KindIcon = project.available
    ? kindIcons[project.kind ?? "screen"]
    : FileQuestion;
  const isPicture = thumbnail !== null && project.kind !== "audio";
  // How light the picture is under each corner something is drawn over,
  // once it has loaded; until then, or with no picture, they take the
  // window's own appearance.
  const [tones, setTones] = useState<PictureTones | null>(null);
  const list = useListFormatter({ style: "short", type: "unit" });
  const { direction } = useLocale();
  // Where the pointer is across the picture, from 0 to 1, while it scrubs.
  const [scrub, setScrub] = useState<number | null>(null);
  const canScrub =
    project.available &&
    isPicture &&
    project.durationMs !== null &&
    project.kind !== "screenshot";
  const strip = canScrub ? scrubStrip : null;
  const isScrubbing = Boolean(strip) && scrub !== null;
  const frameIndex =
    scrub === null || !strip
      ? null
      : Math.min(strip.frames - 1, Math.floor(scrub * strip.frames));
  const { tracks } = project;
  const captured =
    project.available && tracks
      ? trackIcons.filter(({ track }) => tracks[track])
      : [];
  const appearanceOver = (corner: PictureCorner) => {
    const tone = isPicture ? tones?.[corner] : null;
    return tone ? `appearance-${tone}` : undefined;
  };
  return (
    <div
      className={cn(
        "relative aspect-video rounded-panel outline-2 outline-offset-2 transition-[translate,box-shadow,outline-color] ease-out",
        isSelected ? "outline-primary" : "outline-transparent",
        project.available &&
          "group-hover/card:-translate-y-0.5 group-hover/card:shadow-lg",
      )}
      onPointerEnter={() => {
        if (canScrub && scrubStrip === undefined) onScrubRequest();
      }}
      onPointerLeave={() => {
        setScrub(null);
      }}
      onPointerMove={(event) => {
        if (!strip || event.pointerType !== "mouse") return;
        const bounds = event.currentTarget.getBoundingClientRect();
        setScrub(
          Math.min(
            1,
            Math.max(0, (event.clientX - bounds.left) / bounds.width),
          ),
        );
      }}
      style={{ transitionDuration: motionDurationCss("travel") }}
    >
      <AriaButton
        aria-label={list.format([
          t("project-browser-open-project", { title: project.title }),
          ...captured.map(({ track }) => trackLabels[track]()),
        ])}
        className={cn(
          "absolute inset-0 flex cursor-default items-center justify-center rounded-panel text-fg-tertiary",
          focusStyles,
          elementFocusVisible,
        )}
        isDisabled={!project.available}
        onPress={onPress}
      >
        {/* The picture is clipped on its own, apart from the badges over
            it: WebKit can fringe a rounded clip once blurred badges are
            composited inside it. */}
        <span className="absolute inset-0 overflow-hidden rounded-panel bg-recess">
          {thumbnail && project.kind === "audio" ? (
            // The ribbon is drawn white; the mask takes its shape and the
            // label colour paints it, so it follows the appearance.
            <span
              className="absolute inset-x-inset inset-y-group bg-current text-fg-secondary mask-contain mask-center mask-no-repeat"
              style={{ maskImage: `url("${thumbnail}")` }}
            />
          ) : isPicture ? (
            <img
              alt=""
              className={cn(
                "absolute inset-0 size-full object-cover",
                frameIndex !== null && "invisible",
              )}
              // Lets the corner's pixels be read; the asset protocol allows
              // it for the window's own origin.
              crossOrigin="anonymous"
              draggable={false}
              onLoad={({ currentTarget }) => {
                setTones(pictureTones(currentTarget, direction));
              }}
              src={thumbnail}
            />
          ) : (
            <KindIcon className="absolute inset-0 m-auto size-icon-lg" />
          )}
          {/* One picture holds every frame, so moving between them shows
              one already loaded. The frame covers the card as the still
              does, and the strip slides along behind it. */}
          {strip && frameIndex !== null ? (
            <span className="absolute inset-0 flex items-center justify-center">
              <span
                className={cn(
                  "relative shrink-0 overflow-hidden",
                  strip.frameWidth / strip.frameHeight >= 16 / 9
                    ? "h-full"
                    : "w-full",
                )}
                // The strip is one picture with its frames left to right.
                dir="ltr"
                style={{
                  aspectRatio: `${String(strip.frameWidth)} / ${String(strip.frameHeight)}`,
                }}
              >
                <img
                  alt=""
                  className="absolute start-0 top-0 h-full max-w-none"
                  draggable={false}
                  src={strip.src}
                  style={{
                    transform: `translateX(-${String((frameIndex * 100) / strip.frames)}%)`,
                    width: `${String(strip.frames * 100)}%`,
                  }}
                />
              </span>
            </span>
          ) : null}
          {/* A hairline inside the edge, so a light still keeps its shape
              against a light window. */}
          <span className="absolute inset-0 rounded-panel inset-ring inset-ring-divider" />
        </span>
        {isScrubbing ? <ScrubPlayhead at={scrub} /> : null}
        <TrackBadge appearance={appearanceOver("topEnd")} tracks={captured} />
        {project.available ? (
          <LengthBadges
            appearance={appearanceOver("bottomEnd")}
            project={project}
            scrubMs={
              isScrubbing && project.durationMs !== null
                ? scrub * project.durationMs
                : null
            }
          />
        ) : null}
      </AriaButton>
      {project.available ? (
        // Hidden until the card is pointed at or focused, as Photos does,
        // unless something is already chosen: then every card shows one.
        <div
          className={cn(
            "absolute start-inset-sm top-inset-sm flex transition-opacity",
            !isSelected &&
              !isSelectionMode &&
              "opacity-0 group-hover/card:opacity-100 has-[[data-focus-visible]]:opacity-100",
            appearanceOver("topStart"),
          )}
        >
          <Checkbox
            aria-label={t("project-browser-select-project", {
              title: project.title,
            })}
            isSelected={isSelected}
            onChange={onSelectedChange}
          />
        </div>
      ) : null}
    </div>
  );
}
