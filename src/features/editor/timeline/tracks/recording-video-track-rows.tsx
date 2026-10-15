// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, FileVideoCamera, Monitor } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { RECORDING_VIDEO_TRACK_ORDER } from "../../screenshot/screenshot-output";
import { RecordingVideoTrackId } from "../../types";
import { TimelineViewportState } from "../timeline-viewport";

import { CameraBackgroundStrip } from "./camera-background-strip";
import { RecordingTrackLanesProps } from "./recording-track-lanes-contract";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineVideoClip } from "./timeline-video-clip";

type VideoTrackRowsProps = Pick<
  RecordingTrackLanesProps,
  | "blade"
  | "cameraBackground"
  | "enabledTracks"
  | "enabledVideoTracks"
  | "isCameraSeparate"
  | "layout"
  | "onEnabledVideoTracksChange"
  | "onSeek"
  | "onSelectedTrackChange"
  | "selectedTrack"
  | "sourceDurationMs"
  | "thumbnails"
> & { viewport: TimelineViewportState };

export function RecordingVideoTrackRows({
  blade,
  cameraBackground,
  enabledTracks,
  enabledVideoTracks,
  isCameraSeparate,
  layout,
  onEnabledVideoTracksChange,
  onSeek,
  onSelectedTrackChange,
  selectedTrack,
  sourceDurationMs,
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
        const strip =
          pane.kind === "camera" && cameraBackground ? (
            <CameraBackgroundStrip
              edit={blade.edit}
              isEnabled={enabled}
              lane={cameraBackground}
              onSeek={onSeek}
              onSelectTrack={() => {
                onSelectedTrackChange(trackId);
              }}
              sourceDurationMs={sourceDurationMs}
              viewport={viewport}
            />
          ) : null;
        const hasRanges =
          strip !== null && (cameraBackground?.ranges.length ?? 0) > 0;
        const clip = (
          <TimelineVideoClip
            blade={blade}
            enabled={enabled}
            onSelect={onSelectedTrackChange}
            selected={selectedTrack === trackId}
            thumbnails={thumbnails[trackId]}
            trackId={trackId}
            viewport={viewport}
          />
        );
        return (
          <div
            className={cn(
              "flex gap-section",
              strip ? "items-stretch" : "items-center",
            )}
            key={trackId}
          >
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
              spansRow={Boolean(strip)}
            />
            {strip ? (
              // The ranges and the clip they act on share one fill, so the
              // strip reads as part of the camera.
              <div
                className={cn(
                  "flex min-w-0 grow flex-col rounded-control transition-colors duration-200",
                  hasRanges && "bg-fill-tertiary",
                )}
              >
                {strip}
                {clip}
              </div>
            ) : (
              clip
            )}
          </div>
        );
      })}
    </>
  );
}
