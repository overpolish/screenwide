// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pause, Play } from "lucide-react";
import { useId, useMemo, useRef } from "react";

import { IconButton } from "../../../../components/base/button/icon-button";
import { Text } from "../../../../components/base/text/text";
import { useFittedPanel } from "../../../popup-panel/use-fitted-panel";
import { formatDuration } from "../../duration";
import { timelineWaveformPath } from "../audio/timeline-waveform-path";

import { useNotePlayback } from "./use-note-playback";

import type { PopupPanelNoteContent } from "../../../popup-panel/store";

/**
 * A moment's voice note in the panel window: play and pause, and the note's
 * waveform filling with the kind's colour as it plays. The waveform is drawn
 * as the timeline's audio lanes draw theirs.
 */
export function MomentNotePanel({
  content,
}: {
  content: PopupPanelNoteContent;
}) {
  const contentRef = useRef<HTMLDivElement>(null);
  const clipId = useId();
  useFittedPanel(
    contentRef,
    `${content.artifactId.toString()}:${content.moment.toString()}`,
  );
  const { playing, progress, toggle } = useNotePlayback(
    content.artifactId,
    content.moment,
  );
  const path = useMemo(
    () => timelineWaveformPath(content.waveform, 0),
    [content.waveform],
  );

  return (
    <div
      className="flex w-full items-center gap-control p-control-inset"
      ref={contentRef}
    >
      <IconButton
        aria-label={playing ? "Pause voice note" : "Play voice note"}
        onPress={() => {
          void toggle();
        }}
      >
        {playing ? <Pause /> : <Play />}
      </IconButton>
      <svg
        aria-label={`${content.name} voice note`}
        className="h-control-height min-w-0 grow text-content-fg-tertiary"
        preserveAspectRatio="none"
        role="img"
        viewBox="0 0 1000 40"
      >
        <clipPath id={clipId}>
          <rect height="40" width={progress * 1000} x="0" y="0" />
        </clipPath>
        <path
          className="stroke-current"
          d={path}
          fill="none"
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />
        <path
          clipPath={`url(#${clipId})`}
          d={path}
          fill="none"
          stroke={content.color}
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />
      </svg>
      <Text className="shrink-0 tabular-nums" variant="footnote">
        {formatDuration(content.durationMs)}
      </Text>
    </div>
  );
}
