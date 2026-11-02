// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type LucideIcon } from "lucide-react";
import { type ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { Badge } from "../../../primitives/badge/badge";
import { formatDuration } from "../../editor/duration";

import type { ProjectKind, ProjectTracks } from "../types";

// What is drawn over a card's picture. Each part takes the appearance of the
// picture under its corner (`appearance`), so it reads on a light still as
// on a dark one. Each is a `Badge`, a slight blur under it keeping fine
// detail, such as text in a recording, from showing through its fill.

/** One source a recording captured, with its icon. */
export type CapturedTrack = { icon: LucideIcon; track: keyof ProjectTracks };

/** What the recording holds, so a take with no microphone or a missing
 * camera shows before it is opened. Out of the way until the card is
 * pointed at, like the selection box opposite it. */
export function TrackBadge({
  appearance,
  tracks,
}: {
  appearance: string | undefined;
  tracks: CapturedTrack[];
}) {
  if (tracks.length === 0) return null;
  return (
    // The fade is on a wrapper: the badge sets its own opacity as it pops.
    <span
      className={cn(
        "absolute end-inset-sm top-inset-sm flex opacity-0 transition-opacity group-hover/card:opacity-100",
        appearance,
      )}
    >
      <Badge className="backdrop-blur-md">
        {/* A line tall, so the badge stands as high as the text ones. */}
        <span className="flex h-[1lh] items-center gap-inline">
          {tracks.map(({ icon: Icon, track }) => (
            <Icon aria-hidden key={track} />
          ))}
        </span>
      </Badge>
    </span>
  );
}

/** Bottom end, where file browsers and video players put a clip's length:
 * a replay says it is one, then the length, or, while the card is pointed
 * along, where in the take the frame shown is. */
export function LengthBadges({
  appearance,
  kind: projectKind,
  lengthMs,
  replay,
}: {
  appearance: string | undefined;
  kind: ProjectKind | null;
  /** How long it plays, or where in it the frame shown is, in ms. */
  lengthMs: number | null;
  replay: boolean;
}) {
  const length = lengthMs === null ? null : formatDuration(lengthMs);
  const kind =
    projectKind === "screenshot"
      ? t("project-browser-badge-screenshot")
      : replay
        ? t("project-browser-badge-replay")
        : null;
  if (!length && !kind) return null;
  return (
    <span className={cn("ms-auto flex shrink-0 gap-inline", appearance)}>
      {kind ? <Badge className="backdrop-blur-md">{kind}</Badge> : null}
      {length ? <Badge className="backdrop-blur-md">{length}</Badge> : null}
    </span>
  );
}

/** Bottom start, while the card is pointed along: what happens there, the
 * line being said or the beat's name, as the strip's island says it. It
 * gives way to the length beside it, cut short at its own end. */
export function MomentCaption({
  appearance,
  children,
}: {
  appearance: string | undefined;
  children: ReactNode;
}) {
  return (
    <span className={cn("flex min-w-0", appearance)}>
      {/* The badge keeps its size by default; here it is the part that
          gives way, so it may shrink and its text is cut short. */}
      <Badge className="min-w-0 shrink backdrop-blur-md">
        <span className="min-w-0 truncate" dir="auto">
          {children}
        </span>
      </Badge>
    </span>
  );
}
