// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { NativeRecordingWorkspaceViewport } from "../../recording/preview/native-recording-workspace-viewport";
import { PreparedAudioTrack } from "../../types";

import { AudioTrackVolumes } from "./audio-level";
import { setRecordingAudioVisualizer } from "./recording-audio-visualizer-api";

/** A number for each envelope the editor has been handed, so a new envelope
 * for the same track, as a microphone switch brings, is uploaded again. */
const envelopeIds = new WeakMap<number[], number>();
let nextEnvelopeId = 0;
const envelopeId = (waveform: number[]) => {
  let id = envelopeIds.get(waveform);
  if (id === undefined) {
    id = nextEnvelopeId++;
    envelopeIds.set(waveform, id);
  }
  return id;
};

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
  // Uploaded when which tracks are enabled, how loud they are, or their
  // envelopes change, and not on every re-render.
  const tracksKey = tracks
    .map(
      ({ gainDecibels, streamIndex, waveform }) =>
        `${streamIndex.toString()}:${gainDecibels.toString()}:${envelopeId(waveform).toString()}`,
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
      ariaLabel={t("editor-timeline-audio-visualizer")}
      audioOnly
      isBusy={false}
      panes={[]}
      workspaceHeight={0}
      workspaceWidth={0}
    />
  );
}
