// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play, Scissors } from "lucide-react";

import { cn } from "../../../lib/styling";
import {
  IconButton,
  IconToggleButton,
} from "../../../primitives/button/icon-button";
import { TooltipTrigger } from "../../../primitives/tooltip/tooltip-trigger";

import { LEFT_COLUMN } from "./left-column";
import { type Editor } from "./model/use-editor";

// The timeline's own controls, centred in the left column under the tools:
// play at the top, where you are in the video over how long it is, and at
// the foot the scissors, beside the strip they cut.

/** Seconds as a timecode: hours, minutes and seconds, two digits each. */
const timecode = (seconds: number) => {
  const whole = Math.max(0, Math.round(seconds));
  return [Math.floor(whole / 3600), Math.floor(whole / 60) % 60, whole % 60]
    .map((part) => String(part).padStart(2, "0"))
    .join(":");
};

export function Transport({ editor }: { editor: Editor }) {
  return (
    <div
      className={cn(LEFT_COLUMN, "flex shrink-0 flex-col items-center gap-1")}
    >
      <IconButton
        aria-label={editor.isPlaying ? "Pause" : "Play"}
        color="primary"
        onPress={editor.togglePlay}
      >
        {editor.isPlaying ? <Pause /> : <Play />}
      </IconButton>
      {/* Where the playhead is, and how long the video is: both in the
          edited time, so the length shortens as the pauses are squeezed. */}
      <span className="flex flex-col text-caption whitespace-nowrap tabular-nums">
        <span className="text-fg">
          {timecode(editor.map.toTime(editor.playhead))}
        </span>
        <span className="text-fg-tertiary">
          {timecode(editor.map.duration)}
        </span>
      </span>
      {/* On, a drag across the strip draws a stretch to cut. ⌥ held does
          the same for one drag. */}
      <div className="mt-auto">
        <TooltipTrigger tooltip={{ label: "Cut", shortcut: "⌥" }}>
          <IconToggleButton
            aria-label="Cut"
            isSelected={editor.isCutting}
            onChange={editor.setCutting}
          >
            <Scissors />
          </IconToggleButton>
        </TooltipTrigger>
      </div>
    </div>
  );
}
