// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Mic, Volume2 } from "lucide-react";

import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { PreparedAudioTrack } from "../types";

import { AudioTrackVolumes } from "./audio-level";
import { Waveform } from "./scrub-timeline";
import { TimelineBladeController } from "./timeline-blade";
import { TimelineViewportState } from "./timeline-viewport";

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
    <div className="flex flex-col gap-0.5">
      {audioTracks.map((track) => {
        const enabled = enabledTracks.has(track.streamIndex);
        const mustRemainEnabled =
          enabled && enabledTracks.size === 1 && !hasEnabledVideo;
        const Icon = track.kind === "microphone" ? Mic : Volume2;
        return (
          <div className="flex items-center gap-2" key={track.streamIndex}>
            <div
              className={`flex h-8 w-[calc(var(--recording-inspector-width,clamp(270px,23vw,300px))-1.25rem)] shrink-0 items-center gap-2 rounded px-2 text-xs font-medium text-content-fg transition-colors ${selectedTrack === track.streamIndex ? "bg-info/15" : ""}`}
              onClick={() => {
                onSelectTrack(track.streamIndex);
              }}
            >
              <Checkbox
                aria-label={
                  mustRemainEnabled
                    ? `${track.label} must remain included`
                    : `${enabled ? "Exclude" : "Include"} ${track.label}`
                }
                isDisabled={mustRemainEnabled}
                isSelected={enabled}
                onChange={() => {
                  const next = new Set(enabledTracks);
                  if (next.has(track.streamIndex)) {
                    if (mustRemainEnabled) return;
                    next.delete(track.streamIndex);
                  } else next.add(track.streamIndex);
                  onEnabledTracksChange(next);
                }}
              />
              <Icon className="shrink-0 text-muted" size={14} />
              <span className="min-w-0 grow truncate">{track.label}</span>
            </div>
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
