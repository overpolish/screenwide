// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";

import { NOTE_TAKER, speakerInitials } from "./speaker-initials";

// A speaker as a round avatar of their initials, so each is known at a
// glance down the page: the note taker in the primary ink, as the one
// speaker who is always there, everyone else on the quieter grey fill.

export function SpeakerAvatar({
  className,
  name,
  size = "page",
}: {
  name: string;
  className?: string;
  /** `page` beside a speaker's words, `card` on a project card. */
  size?: "card" | "page";
}) {
  return (
    <span
      aria-hidden
      className={cn(
        "flex shrink-0 items-center justify-center rounded-full font-medium tabular-nums select-none",
        size === "page" ? "size-7 text-label" : "size-5 text-caption",
        name === NOTE_TAKER
          ? "bg-primary text-on-primary"
          : "bg-fill-strong text-fg-secondary",
        className,
      )}
      dir="auto"
    >
      {speakerInitials(name)}
    </span>
  );
}
