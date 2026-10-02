// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { composedVideoTracks } from "../../export/camera-output";

import type { ResolvedScrubPreviewProps } from "../../recording/preview/recording-preview-props";

/** The track sets and volumes the preview derives from its props.
 * `selectedVideoTracks` are those kept in the export, which the timeline's
 * switches show; `composedVideoTracks` are those the workspace draws and
 * edits, which leave out a camera saved as a separate file.
 *
 * Everything derived here feeds memoized children. A canvas-resize gesture
 * re-renders the preview at pointer rate, so a derived array or Set rebuilt
 * per render would defeat the memo of every subtree it reaches. */
export function useRecordingPreviewTracks(props: ResolvedScrubPreviewProps) {
  const {
    audioTrackVolumes,
    audioTracks,
    bakeCamera,
    enabledStreamIndices,
    enabledVideoTracks,
  } = props;
  const selectedStreamIndices = useMemo(
    () => enabledStreamIndices ?? audioTracks.map((track) => track.streamIndex),
    [audioTracks, enabledStreamIndices],
  );
  const enabledTracks = useMemo(
    () => new Set(selectedStreamIndices),
    [selectedStreamIndices],
  );
  const selectedVideoTracks = useMemo(
    () => new Set(enabledVideoTracks),
    [enabledVideoTracks],
  );
  const composedTracks = useMemo(
    () => new Set(composedVideoTracks(enabledVideoTracks, bakeCamera)),
    [bakeCamera, enabledVideoTracks],
  );
  const audioVolumeByStream = useMemo(
    () =>
      new Map(
        audioTrackVolumes.map(({ decibels, streamIndex }) => [
          streamIndex,
          decibels,
        ]),
      ),
    [audioTrackVolumes],
  );
  return {
    audioVolumeByStream,
    composedVideoTracks: composedTracks,
    enabledTracks,
    isCameraSeparate:
      selectedVideoTracks.has("camera") && !composedTracks.has("camera"),
    selectedStreamIndices,
    selectedVideoTracks,
  };
}
