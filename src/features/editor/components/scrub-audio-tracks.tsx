// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, Volume2 } from "lucide-react";

import { PreparedAudioTrack } from "../types";

import { AudioTrackVolumes } from "./audio-level";
import { TimelineBladeController } from "./timeline-blade";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineViewportState } from "./timeline-viewport";
import { Waveform } from "./timeline-waveform";

export function ScrubAudioTracks({
  audioTracks,
  blade,
  enabledTracks,
  hasEnabledVideo,
  onEnabledTracksChange,
  onSelectTrack,
  selectedTrack,
  viewport,
  volumes,
}: {
  audioTracks: PreparedAudioTrack[];
  blade: TimelineBladeController;
  enabledTracks: Set<number>;
  hasEnabledVideo: boolean;
  onEnabledTracksChange: (tracks: Set<number>) => void;
  onSelectTrack: (streamIndex: number) => void;
  selectedTrack: number | null;
  viewport: TimelineViewportState;
  volumes: AudioTrackVolumes;
}) {
  return (
    <div className="flex flex-col gap-control">
      {audioTracks.map((track) => {
        const enabled = enabledTracks.has(track.streamIndex);
        const mustRemainEnabled =
          enabled && enabledTracks.size === 1 && !hasEnabledVideo;
        const Icon = track.kind === "microphone" ? Mic : Volume2;
        return (
          <div
            className="flex items-center gap-section"
            key={track.streamIndex}
          >
            <TimelineTrackHeader
              icon={<Icon />}
              inclusion={{
                isIncluded: enabled,
                isRequired: mustRemainEnabled,
                onChange: () => {
                  const next = new Set(enabledTracks);
                  if (next.has(track.streamIndex)) {
                    if (mustRemainEnabled) return;
                    next.delete(track.streamIndex);
                  } else next.add(track.streamIndex);
                  onEnabledTracksChange(next);
                },
              }}
              isSelected={selectedTrack === track.streamIndex}
              label={track.label}
              onSelect={() => {
                onSelectTrack(track.streamIndex);
              }}
            />
            <Waveform
              blade={blade}
              enabled={enabled}
              onSelect={() => {
                onSelectTrack(track.streamIndex);
              }}
              track={track}
              viewport={viewport}
              volumeDecibels={volumes.get(track.streamIndex) ?? 0}
            />
          </div>
        );
      })}
    </div>
  );
}
