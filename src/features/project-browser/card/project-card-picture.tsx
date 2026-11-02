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
import { type ReactNode, useState } from "react";
import { useListFormatter, useLocale } from "react-aria";
import { Button as AriaButton, type PressEvent } from "react-aria-components";

import { t } from "../../../i18n/i18n";
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
  MomentCaption,
  TrackBadge,
} from "./project-card-overlays";
import { StripFrame } from "./strip-frame";

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

export type ProjectCardPictureProps = {
  /** Where the card shows, as a share of how long the recording plays;
   * null at rest, where the still shows. */
  at: number | null;
  /** What happens where it shows, while pointed along, with what a search
   * found in it marked. */
  caption: ReactNode;
  isLinked: boolean;
  isSelected: boolean;
  /** Shows the selection box without a hover, once anything is chosen. */
  isSelectionMode: boolean;
  /** Told where the pointer is along the picture, from 0 to 1, or null as
   * it leaves. */
  onPointAt: (at: number | null) => void;
  onPress: (event: PressEvent) => void;
  /** Asks for `scrubStrip`, the first time the pointer comes over a
   * recording that has not asked yet. */
  onScrubRequest: () => void;
  onSelectedChange: (selected: boolean) => void;
  /** A recording's beats along the picture's foot, the editor's overview
   * drawn as a picture; null for none, as for a screenshot. */
  overview: ReactNode;
  /** How long the recording plays now, in ms, which the length badge
   * shows and the pointer's place is a share of. */
  playsMs: number | null;
  project: ProjectSummary;
  /** Frames along the edit: undefined until asked for, null for a
   * recording the editor has not composed one for. */
  scrubStrip: ScrubStrip | null | undefined;
  thumbnail: string | null;
};

/**
 * A card's picture, edge to edge in rounded corners. Moving across it moves
 * through the recording: the frame the editor composed there, with what
 * happens there and the time at its end over its foot, and a recording's
 * beats along the very bottom. Pressed, it opens
 * the project; with Shift, Command or Control it is chosen.
 */
export function ProjectCardPicture({
  at,
  caption,
  isLinked,
  isSelected,
  isSelectionMode,
  onPointAt,
  onPress,
  onScrubRequest,
  onSelectedChange,
  overview,
  playsMs,
  project,
  scrubStrip,
  thumbnail,
}: ProjectCardPictureProps) {
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
  const canScrub =
    project.available &&
    project.durationMs !== null &&
    project.kind !== "screenshot";
  const strip = canScrub && isPicture ? scrubStrip : null;
  const { tracks } = project;
  const captured =
    project.available && tracks
      ? trackIcons.filter(({ track }) => tracks[track])
      : [];
  const appearanceOver = (corner: PictureCorner) => {
    const tone = isPicture ? tones?.[corner] : null;
    return tone ? `appearance-${tone}` : undefined;
  };
  const frame =
    at === null || !strip
      ? null
      : Math.min(strip.frames - 1, Math.floor(at * strip.frames));

  return (
    // The card stays put under the pointer, with no lift: the scrub is its
    // answer to pointing. One pointed at in the storage strip is outlined.
    <div
      className={cn(
        "relative aspect-video rounded-panel outline-2 outline-offset-2",
        isSelected
          ? "outline-primary"
          : isLinked
            ? "outline-fg-tertiary"
            : "outline-transparent",
      )}
      onPointerEnter={() => {
        if (canScrub && scrubStrip === undefined) onScrubRequest();
      }}
      onPointerLeave={() => {
        onPointAt(null);
      }}
      onPointerMove={(event) => {
        if (!canScrub || event.pointerType !== "mouse") return;
        const bounds = event.currentTarget.getBoundingClientRect();
        // The recording runs left to right in any language.
        onPointAt(
          Math.min(
            0.999,
            Math.max(0, (event.clientX - bounds.left) / bounds.width),
          ),
        );
      }}
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
        data-project-picture=""
        isDisabled={!project.available}
        onPress={onPress}
      >
        {/* Clipped on its own, apart from the badges over it: WebKit
                can fringe a rounded clip once blurred badges are
                composited inside it. */}
        <span className="absolute inset-0 overflow-hidden rounded-panel bg-recess">
          {thumbnail && project.kind === "audio" ? (
            // The ribbon is drawn white; the mask takes its shape and
            // the label colour paints it, so it follows the appearance.
            <span
              className="absolute inset-x-inset inset-y-group bg-current text-fg-secondary mask-contain mask-center mask-no-repeat"
              style={{ maskImage: `url("${thumbnail}")` }}
            />
          ) : isPicture ? (
            <img
              alt=""
              className={cn(
                "absolute inset-0 size-full object-cover",
                frame !== null && "invisible",
              )}
              // Lets the corner's pixels be read; the asset protocol
              // allows it for the window's own origin.
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
          {strip && frame !== null ? (
            <StripFrame frame={frame} strip={strip} />
          ) : null}
          {/* A hairline inside the edge, so a light still keeps its
                  shape against a light window. */}
          <span className="absolute inset-0 rounded-panel inset-ring inset-ring-divider" />
        </span>
        <TrackBadge appearance={appearanceOver("topEnd")} tracks={captured} />
        {/* Along the foot, the caption giving way to the length, standing
            on the overview as they would on the picture's edge. */}
        <span className="absolute inset-x-0 bottom-0 flex flex-col">
          <span className="flex items-end gap-inline px-inset-sm pb-inset-sm">
            {caption && at !== null ? (
              <MomentCaption appearance={appearanceOver("bottomStart")}>
                {caption}
              </MomentCaption>
            ) : null}
            {project.available ? (
              <LengthBadges
                appearance={appearanceOver("bottomEnd")}
                kind={project.kind}
                lengthMs={
                  playsMs !== null && at !== null ? at * playsMs : playsMs
                }
                replay={project.replay}
              />
            ) : null}
          </span>
          {overview ? (
            // On a band of the badges' backing, in the appearance the
            // picture's foot calls for, so the line reads on any picture;
            // rounded on its own rather than inside the picture's clip,
            // where WebKit can fringe a blur.
            <span
              className={cn(
                "rounded-b-panel bg-fill-strong px-inset-sm py-1 backdrop-blur-md",
                appearanceOver("bottomStart"),
              )}
            >
              {overview}
            </span>
          ) : null}
        </span>
      </AriaButton>
      {project.available ? (
        // Hidden until the card is pointed at or focused, as Photos does,
        // unless something is already chosen: then every card shows one.
        // On a blurred disc of the strong fill in the appearance the
        // picture's corner calls for, as the badges are, so the box keeps
        // its edge over a busy picture; it stands where it would without.
        <div
          className={cn(
            "absolute start-0.5 top-0.5 flex rounded-full bg-fill-strong p-1 backdrop-blur-md transition-opacity",
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
