// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, FileVideoCamera, Monitor } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { RECORDING_VIDEO_TRACK_ORDER } from "../../screenshot/screenshot-output";
import { RecordingVideoTrackId } from "../../types";
import { TimelineViewportState } from "../timeline-viewport";

import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineVideoClip } from "./timeline-video-clip";

type VideoTrackRowsProps = Pick<
  RecordingTrackLanesProps,
  | "blade"
  | "enabledTracks"
  | "enabledVideoTracks"
  | "isCameraSeparate"
  | "layout"
  | "onEnabledVideoTracksChange"
  | "onSelectedTrackChange"
  | "selectedTrack"
  | "thumbnails"
> & { viewport: TimelineViewportState };

export function RecordingVideoTrackRows({
  blade,
  enabledTracks,
  enabledVideoTracks,
  isCameraSeparate,
  layout,
  onEnabledVideoTracksChange,
  onSelectedTrackChange,
  selectedTrack,
  thumbnails,
  viewport,
}: VideoTrackRowsProps) {
  // The screen and camera panes as rows, front layer first.
  const videoRows = layout.panes
    .map((pane, index) => ({
      pane,
      trackId: index === 0 ? ("primary" as const) : ("camera" as const),
    }))
    .sort(
      (left, right) =>
        RECORDING_VIDEO_TRACK_ORDER.indexOf(left.trackId) -
        RECORDING_VIDEO_TRACK_ORDER.indexOf(right.trackId),
    );
  return (
    <>
      {videoRows.map(({ pane, trackId }) => {
        const Icon = pane.kind === "camera" ? Camera : Monitor;
        const label =
          pane.kind === "camera"
            ? t("editor-panels-camera")
            : t("editor-panels-screen");
        const enabled = enabledVideoTracks.has(trackId);
        const mustRemainEnabled =
          enabled && enabledVideoTracks.size === 1 && enabledTracks.size === 0;
        return (
          <div className="flex items-center gap-section" key={trackId}>
            <TimelineTrackHeader
              icon={<Icon />}
              inclusion={{
                isIncluded: enabled,
                isRequired: mustRemainEnabled,
                onChange: () => {
                  const next = new Set<RecordingVideoTrackId>(
                    enabledVideoTracks,
                  );
                  if (next.has(trackId)) {
                    if (mustRemainEnabled) return;
                    next.delete(trackId);
                  } else next.add(trackId);
                  onEnabledVideoTracksChange(next);
                },
              }}
              isSelected={selectedTrack === trackId}
              label={label}
              note={
                trackId === "camera" && isCameraSeparate
                  ? {
                      icon: <FileVideoCamera />,
                      label: t("editor-timeline-camera-separate"),
                    }
                  : undefined
              }
              onSelect={() => {
                onSelectedTrackChange(trackId);
              }}
            />
            <TimelineVideoClip
              blade={blade}
              enabled={enabled}
              onSelect={onSelectedTrackChange}
              selected={selectedTrack === trackId}
              thumbnails={thumbnails[trackId]}
              trackId={trackId}
              viewport={viewport}
            />
          </div>
        );
      })}
    </>
  );
}
