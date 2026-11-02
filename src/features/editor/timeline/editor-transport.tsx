// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play, Scissors } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../../primitives/button/icon-button";
import { ToolbarTool } from "../../../primitives/toolbar/toolbar-tool";
import { TooltipTrigger } from "../../../primitives/tooltip/tooltip-trigger";
import { formatDuration } from "../duration";
import { EDITOR_LEFT_COLUMN } from "../editor-left-column";
import { ChromeSetBack } from "../panel/chrome-set-back";

import { Playhead } from "./scrub-playhead";
import { ElapsedTime } from "./timeline-elapsed-time";

// The strip's own controls, centred in the editor's left column under the
// tools: play at the top, where the playhead is over how long the recording
// is, and at the foot the scissors, beside the strip they cut. Tracks is in
// the title bar with the modes (`EditorModeBar`). While the Export island
// stands over the editor only play answers: the scissors are set back.

export type EditorTransportProps = {
  /** How long the recording plays once squeezed and cut, in ms. */
  durationMs: number;
  /** Whether a drag across the strip draws a stretch to cut. */
  isCutting: boolean;
  isPlaying: boolean;
  onCuttingChange: (isCutting: boolean) => void;
  onPlayingChange: (isPlaying: boolean) => void;
  // Its time is written straight to the readout, so playing never renders
  // the transport.
  /** Where the playhead is, in the edited time. */
  playhead: Playhead;
  /** Only play answers: nothing is cut. */
  isLocked?: boolean;
};

export function EditorTransport({
  durationMs,
  isCutting,
  isLocked = false,
  isPlaying,
  onCuttingChange,
  onPlayingChange,
  playhead,
}: EditorTransportProps) {
  const playLabel = isPlaying
    ? t("editor-timeline-pause-preview")
    : t("editor-timeline-play-preview");
  return (
    <div
      className={cn(
        "flex h-full shrink-0 flex-col items-center gap-inline",
        EDITOR_LEFT_COLUMN,
      )}
    >
      <TooltipTrigger tooltip={{ label: playLabel, shortcut: "Space" }}>
        <IconButton
          aria-keyshortcuts="Space"
          aria-label={playLabel}
          color="primary"
          onPress={() => {
            onPlayingChange(!isPlaying);
          }}
        >
          {isPlaying ? (
            <Pause className="fill-current" />
          ) : (
            <Play className="fill-current" />
          )}
        </IconButton>
      </TooltipTrigger>
      <span className="flex flex-col items-center text-caption whitespace-nowrap tabular-nums">
        <span className="text-fg">
          <ElapsedTime playhead={playhead} />
        </span>
        <span className="text-fg-tertiary">{formatDuration(durationMs)}</span>
      </span>
      <ChromeSetBack
        className="mt-auto flex flex-col items-center gap-inline"
        isSetBack={isLocked}
      >
        {/* Held, ⌥ draws a stretch to cut for one drag without the toggle. */}
        <ToolbarTool
          aria-label={t("editor-timeline-cut-name")}
          isSelected={isCutting && !isLocked}
          label={t("editor-timeline-cut")}
          onChange={onCuttingChange}
          shortcut="Alt"
        >
          <Scissors />
        </ToolbarTool>
      </ChromeSetBack>
    </div>
  );
}
