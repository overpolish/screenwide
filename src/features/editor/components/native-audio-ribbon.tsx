// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef } from "react";

import { setRecordingAudioVisualizer } from "../recording-audio-visualizer-api";
import { PreparedAudioTrack } from "../types";

import { AudioTrackVolumes } from "./audio-level";
import { NativeRecordingWorkspaceViewport } from "./native-recording-workspace-viewport";

/**
 * The audio-only preview.
 *
 * Uploads the enabled tracks' envelopes and gains. The native playback
 * worker owns the position and follows the audio clock used by video.
 */
export function NativeAudioRibbon({
  artifactId,
  audioTracks,
  enabledTracks,
  volumes,
}: {
  artifactId: number;
  audioTracks: PreparedAudioTrack[];
  enabledTracks: Set<number>;
  volumes: AudioTrackVolumes;
}) {
  // One invoke at a time, in the order they were made: a clear and the set
  // that replaces it must never land the wrong way round.
  const queueRef = useRef(Promise.resolve());
  const send = useCallback((work: () => Promise<unknown>) => {
    queueRef.current = queueRef.current.then(() =>
      work().then(
        () => undefined,
        () => undefined,
      ),
    );
  }, []);

  const tracks = useMemo(
    () =>
      audioTracks
        .filter((track) => enabledTracks.has(track.streamIndex))
        .map((track) => ({
          gainDecibels: volumes.get(track.streamIndex) ?? 0,
          streamIndex: track.streamIndex,
          waveform: track.waveform,
        })),
    [audioTracks, enabledTracks, volumes],
  );
  // The envelopes themselves never change; only which tracks are enabled and
  // how loud they are. Uploading on that alone keeps a re-render free.
  const tracksKey = tracks
    .map(
      ({ gainDecibels, streamIndex, waveform }) =>
        `${streamIndex.toString()}:${gainDecibels.toString()}:${waveform.length.toString()}`,
    )
    .join("|");
  const tracksRef = useRef(tracks);
  tracksRef.current = tracks;
  useEffect(() => {
    send(() =>
      setRecordingAudioVisualizer({ artifactId, tracks: tracksRef.current }),
    );
    return () => {
      send(() => setRecordingAudioVisualizer({ artifactId, tracks: [] }));
    };
  }, [artifactId, send, tracksKey]);

  return (
    <NativeRecordingWorkspaceViewport
      ariaLabel="Audio visualizer"
      audioOnly
      isBusy={false}
      panes={[]}
      workspaceHeight={0}
      workspaceWidth={0}
    />
  );
}
