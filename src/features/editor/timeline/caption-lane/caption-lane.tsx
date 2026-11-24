// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { use } from "react";

import { CAPTIONS_ID } from "../../recording/formats/format-scope";
import { isNear, StripNearContext } from "../strip/strip-near";
import { type TimelineScale } from "../timeline-scale";

import { CaptionLine, type TimelineCaption } from "./caption-line";

// Where a voice speaks: a thin lane under the beats, its lines the
// captions. Pressing a line opens the captions on it; a right press offers
// what brings the formats in step for the caption track as a whole.

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
  const near = use(StripNearContext);
  return (
    <div className="relative h-4" data-menu-target={CAPTIONS_ID} dir="ltr">
      {lines
        .filter((line) => isNear(near, props.scale, line))
        .map((line) => (
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
