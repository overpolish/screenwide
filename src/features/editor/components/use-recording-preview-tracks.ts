// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import {
  RecordingOutputSettings,
  recordingVideoTrackOrder,
} from "../screenshot-output";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

/** The track sets, volumes and layer order the preview derives from its props.
 *
 * Everything derived here feeds memoized children. A canvas-resize gesture
 * re-renders the preview at pointer rate, so a derived array or Set rebuilt
 * per render would defeat the memo of every subtree it reaches. */
export function useRecordingPreviewTracks(
  props: ResolvedScrubPreviewProps,
  effectiveRecordingOutput: RecordingOutputSettings,
) {
  const {
    audioTrackVolumes,
    audioTracks,
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
  // Resizing never changes layer order, so keep its identity across the drag.
  const videoTrackOrder = useMemo(
    () => recordingVideoTrackOrder(effectiveRecordingOutput),
    // eslint-disable-next-line @eslint-react/exhaustive-deps
    [effectiveRecordingOutput.cameraOnTop],
  );
  const videoTrackOrderList = useMemo(
    () => [...videoTrackOrder],
    [videoTrackOrder],
  );
  return {
    audioVolumeByStream,
    enabledTracks,
    selectedStreamIndices,
    selectedVideoTracks,
    videoTrackOrder,
    videoTrackOrderList,
  };
}
