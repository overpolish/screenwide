// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { StripLane } from "../strip/strip-lane";

import {
  AudioStack,
  type AudioStackProps,
  type StripAudio,
} from "./audio-stack";
import { SongRail } from "./song-rail";

// The sound effects' stack and, under it, the songs' rail, each in a lane
// of its own on the strip, shown where it has clips.

type AudioLanesProps = Omit<AudioStackProps, keyof StripAudio> & {
  audio?: StripAudio;
  /** Where a song's dragged end has snapped, or null, for the guide. */
  onGuide?: (seconds: number | null) => void;
};

export function AudioLanes({ audio, onGuide, ...lane }: AudioLanesProps) {
  const effects = audio?.clips.filter((clip) => clip.kind === "effect") ?? [];
  const songs = audio?.clips.filter((clip) => clip.kind === "music") ?? [];
  return (
    <>
      <StripLane>
        {audio && effects.length > 0 ? (
          <AudioStack {...audio} {...lane} clips={effects} />
        ) : null}
      </StripLane>
      <StripLane>
        {audio && songs.length > 0 ? (
          <SongRail
            {...lane}
            onGuide={onGuide}
            onRetime={audio.onRetime}
            songs={songs}
          />
        ) : null}
      </StripLane>
    </>
  );
}
