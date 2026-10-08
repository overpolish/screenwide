// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AudioLines,
  Camera,
  ImageIcon,
  Mic,
  Monitor,
  Video,
  Volume2,
} from "lucide-react";
import { useState } from "react";
import { Button as AriaButton, type PressEvent } from "react-aria-components";

import { Badge } from "../../components/base/badge/badge";
import { Checkbox } from "../../components/base/checkbox/checkbox";
import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { formatBytes, formatDuration } from "../editor/duration";

import {
  pictureTones,
  type PictureCorner,
  type PictureTones,
} from "./picture-tone";

import type {
  ProjectKind,
  ProjectSummary,
  ProjectTracks,
  ScrubStrip,
} from "./types";

const kindIcons: Record<ProjectKind, typeof Monitor> = {
  audio: AudioLines,
  camera: Video,
  screen: Monitor,
  screenshot: ImageIcon,
};

/** In capture order, with the icons the editor's timeline and the recording
 * bar give the same sources. */
const trackIcons: {
  icon: typeof Monitor;
  label: string;
  track: keyof ProjectTracks;
}[] = [
  { icon: Monitor, label: "screen", track: "screen" },
  { icon: Camera, label: "camera", track: "camera" },
  { icon: Volume2, label: "system audio", track: "systemAudio" },
  { icon: Mic, label: "microphone", track: "microphone" },
];

/** What kind of capture it is, where that is not an ordinary recording, then
 * how long it is and how much disk it takes, as far as each is known. The
 * length stays while scrubbing, so the whole take's is always in view. */
function badgesFor(project: ProjectSummary) {
  return [
    project.kind === "screenshot" ? "Screenshot" : null,
    project.replay ? "Replay" : null,
    project.durationMs === null ? null : formatDuration(project.durationMs),
    project.sizeBytes === null ? null : formatBytes(project.sizeBytes),
  ].filter((badge) => badge !== null);
}

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
 * A card's picture: pressed to open the project, or with Shift, Command or
 * Control to choose it. Moving the pointer across a recording whose editor
 * has composed a scrub strip shows the edit at that point, so a take can be
 * checked without opening it.
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
  const KindIcon = kindIcons[project.kind ?? "screen"];
  const isPicture = thumbnail !== null && project.kind !== "audio";
  // How light the picture is under each corner something is drawn over,
  // once it has loaded; until then, or with no picture, they take the
  // window's own appearance.
  const [tones, setTones] = useState<PictureTones | null>(null);
  // Where the pointer is across the picture, from 0 to 1, while it scrubs.
  const [scrub, setScrub] = useState<number | null>(null);
  const canScrub =
    project.available &&
    isPicture &&
    project.durationMs !== null &&
    project.kind !== "screenshot";
  const strip = canScrub ? scrubStrip : null;
  const frameIndex =
    scrub === null || !strip
      ? null
      : Math.min(strip.frames - 1, Math.floor(scrub * strip.frames));
  const badges = project.available ? badgesFor(project) : [];
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
      className="group/preview relative aspect-video"
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
    >
      <AriaButton
        aria-label={[
          `Open ${project.title}`,
          ...captured.map(({ label }) => label),
        ].join(", ")}
        className={cn(
          "rounded-control text-content-fg-tertiary absolute inset-0 flex cursor-default items-center justify-center",
          // A still is drawn as it is, square corners and all: rounding it
          // would show something the recording does not hold. Without one,
          // the fill takes the corners itself rather than through a clip,
          // which WebKit can fringe once the blurred badges are composited.
          !isPicture && "bg-fill",
          focusStyles,
          elementFocusVisible,
        )}
        isDisabled={!project.available}
        onPress={onPress}
      >
        {thumbnail && project.kind === "audio" ? (
          // The ribbon is drawn white; the mask takes its shape and the
          // label colour paints it, so it follows the appearance.
          <span
            className="text-content-fg-secondary size-full rounded-[inherit] bg-current mask-contain mask-center mask-no-repeat"
            style={{ maskImage: `url("${thumbnail}")` }}
          />
        ) : isPicture ? (
          // Contained by the browser rather than sized from the loaded
          // still, so a card drawn before its still loads is already the
          // right shape: no stretched frame snapping to size.
          <img
            alt=""
            className={cn(
              "absolute inset-0 size-full object-contain",
              frameIndex !== null && "invisible",
            )}
            // Lets the corner's pixels be read; the asset protocol allows it
            // for the window's own origin.
            crossOrigin="anonymous"
            draggable={false}
            onLoad={({ currentTarget }) => {
              setTones(pictureTones(currentTarget));
            }}
            src={thumbnail}
          />
        ) : (
          <KindIcon className="size-icon-large" />
        )}
        {/* One picture holds every frame, so moving between them shows one
            already loaded. The frame is fitted to the card as a still is,
            and the strip slid along behind it. */}
        {strip && frameIndex !== null ? (
          <span className="absolute inset-0 flex items-center justify-center">
            <span
              className={cn(
                "relative overflow-hidden",
                strip.frameWidth / strip.frameHeight >= 16 / 9
                  ? "w-full"
                  : "h-full",
              )}
              style={{
                aspectRatio: `${String(strip.frameWidth)} / ${String(strip.frameHeight)}`,
              }}
            >
              <img
                alt=""
                className="absolute top-0 left-0 h-full max-w-none"
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
        {/* Top-right, what the recording holds, so a take with no
            microphone or a missing camera shows before it is opened. */}
        {captured.length > 0 ? (
          <Badge
            className={cn(
              "right-control top-control absolute backdrop-blur-sm",
              appearanceOver("topRight"),
            )}
          >
            {/* A line tall, so the badge stands as high as the text ones
                below it. */}
            <span className="gap-control flex h-[1lh] items-center">
              {captured.map(({ icon: Icon, track }) => (
                <Icon aria-hidden key={track} />
              ))}
            </span>
          </Badge>
        ) : null}
        {/* Bottom-right, where file browsers and video players put a
            clip's length, in the appearance of the picture beneath them so
            they read on it. */}
        {badges.length > 0 ? (
          <span
            className={cn(
              "gap-control right-control bottom-control absolute flex",
              appearanceOver("bottomRight"),
            )}
          >
            {badges.map((badge) => (
              // A slight blur keeps fine detail, such as text in a
              // recording, from showing through the faint fill.
              <Badge className="backdrop-blur-sm" key={badge}>
                {badge}
              </Badge>
            ))}
          </span>
        ) : null}
      </AriaButton>
      {project.available ? (
        // Hidden until the card is pointed at or focused, as Photos does,
        // unless something is already chosen: then every card shows one.
        <div
          className={cn(
            "top-control left-control absolute flex",
            !isSelected &&
              !isSelectionMode &&
              "opacity-0 group-hover/card:opacity-100 has-[[data-focus-visible]]:opacity-100",
            appearanceOver("topLeft"),
          )}
        >
          <Checkbox
            aria-label={`Select ${project.title}`}
            isSelected={isSelected}
            onChange={onSelectedChange}
          />
        </div>
      ) : null}
    </div>
  );
}
