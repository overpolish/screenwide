// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ProgressBar } from "../../../primitives/progress-bar/progress-bar";
import { Text } from "../../../primitives/text/text";

// A tool at work, which can take a while the first time, on the island's
// text column, as a model's download is (`IslandModelLine`), with the bar
// at the line's end: the camera's effects and the microphone's tools alike.

/** A tool at work: what it is doing, and how far it has got. */
export function IslandWorkLine({
  label,
  value,
}: {
  label: string;
  /** How far the work has got, 0 to 1. */
  value: number;
}) {
  return (
    <div className="flex items-center justify-between gap-separate px-inset-sm">
      <Text as="p" className="text-fg-secondary" variant="caption">
        {label}
      </Text>
      <ProgressBar aria-label={label} className="w-24" value={value * 100} />
    </div>
  );
}
