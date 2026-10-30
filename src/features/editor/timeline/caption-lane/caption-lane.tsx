// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineScale } from "../timeline-scale";

import { CaptionLine, type TimelineCaption } from "./caption-line";

// Where a voice speaks: a thin lane under the beats, its lines the
// captions. Pressing a line opens the captions on it.

export type CaptionLaneProps = {
  /** Whether the captions are hidden from the picture. */
  isHidden: boolean;
  lines: TimelineCaption[];
  onPress: (line: TimelineCaption) => void;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
};

export function CaptionLane({ lines, onPress, ...props }: CaptionLaneProps) {
  return (
    <div className="relative h-4" dir="ltr">
      {lines.map((line) => (
        <CaptionLine
          key={line.start}
          {...props}
          line={line}
          onPress={() => {
            onPress(line);
          }}
        />
      ))}
    </div>
  );
}
