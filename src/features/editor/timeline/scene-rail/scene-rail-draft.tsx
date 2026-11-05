// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { at, placed, type Span, type TimelineScale } from "../timeline-scale";

// The scene a drag along the empty rail makes: while drawn, the scene it
// will be; before, a faint one waiting at the pointer where there is room.

/** The least room, in px, the waiting scene is shown in: its glyph's. */
const GHOST_MIN_PX = 20;

export function SceneRailDraft({
  drawn,
  hover,
  scale,
}: {
  /** The scene being drawn, in recorded seconds. */
  drawn: Span | null;
  /** Where the pointer is over the empty rail, in recorded seconds, and the
   * room there, in px. */
  hover: { at: number; room: number } | null;
  scale: TimelineScale;
}) {
  if (drawn)
    return (
      <span
        aria-hidden
        className="pointer-events-none absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-part bg-fill-strong px-1 text-caption whitespace-nowrap text-fg"
        style={placed(scale, drawn.start, drawn.end)}
      >
        <Plus aria-hidden className="size-icon-sm shrink-0" />
        {t("editor-timeline-scene-new")}
      </span>
    );
  if (!hover || hover.room < GHOST_MIN_PX) return null;
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-part border border-dashed border-fg-tertiary px-1 text-caption whitespace-nowrap text-fg-tertiary pointer-away:hidden"
      style={{ ...at(scale, hover.at), maxWidth: hover.room }}
    >
      <Plus aria-hidden className="size-icon-sm shrink-0" />
      {t("editor-timeline-scene-new")}
    </span>
  );
}
