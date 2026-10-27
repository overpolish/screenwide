// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { type SpokenLine } from "../recording";

// A recording with no picture: its words are the work, so they take the
// picture's place, the line at the playhead in full ink.

export function SoundView({
  playhead,
  speech,
}: {
  playhead: number;
  speech: SpokenLine[];
}) {
  return (
    <div className="flex max-w-[640px] flex-col gap-separate text-title font-normal">
      {speech.map((line) => (
        <p
          className={cn(
            playhead >= line.start && playhead < line.end
              ? "text-fg"
              : "text-fg-tertiary",
          )}
          key={line.start}
        >
          {line.text}
        </p>
      ))}
    </div>
  );
}
