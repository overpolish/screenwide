// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type LucideIcon } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { Badge } from "../../../primitives/badge/badge";
import { formatDuration } from "../../editor/duration";

import type { ProjectSummary, ProjectTracks } from "../types";

// What is drawn over a card's picture. Each part takes the appearance of the
// picture under its corner (`appearance`), so it reads on a light still as
// on a dark one.

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
 * a replay says it is one, then the length, or, while the pointer scrubs,
 * where in the take the frame shown is. */
export function LengthBadges({
  appearance,
  project,
  scrubMs,
}: {
  appearance: string | undefined;
  project: ProjectSummary;
  scrubMs: number | null;
}) {
  const length =
    scrubMs !== null
      ? formatDuration(scrubMs)
      : project.durationMs === null
        ? null
        : formatDuration(project.durationMs);
  const kind =
    project.kind === "screenshot"
      ? t("project-browser-badge-screenshot")
      : project.replay
        ? t("project-browser-badge-replay")
        : null;
  if (!length && !kind) return null;
  return (
    <span
      className={cn(
        "absolute end-inset-sm bottom-inset-sm flex gap-inline",
        appearance,
      )}
    >
      {/* A slight blur keeps fine detail, such as text in a recording,
          from showing through the faint fill. */}
      {kind ? <Badge className="backdrop-blur-md">{kind}</Badge> : null}
      {length ? <Badge className="backdrop-blur-md">{length}</Badge> : null}
    </span>
  );
}

/** Where the pointer is in the take while it scrubs: the playhead's orange,
 * a line through the picture as the editor's timeline draws it. */
export function ScrubPlayhead({ at }: { at: number }) {
  return (
    // The picture runs left to right in any language, as the take does.
    <span
      aria-hidden
      className="pointer-events-none absolute inset-0"
      dir="ltr"
    >
      <span
        className="absolute inset-y-0 w-0.5 -translate-x-1/2 rounded-full bg-emphasis shadow-[0_0_6px_var(--color-emphasis)]"
        style={{ insetInlineStart: `${String(at * 100)}%` }}
      />
    </span>
  );
}
