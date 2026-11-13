// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { cn } from "../../../../lib/styling";
import { StripLane } from "../strip/strip-lane";

import {
  AudioStack,
  type AudioStackProps,
  type StripAudio,
} from "./audio-stack";
import { SongRail } from "./song-rail";
import { setSoundDropTarget } from "./sound-drop-target";
import { SoundGhosts } from "./sound-ghosts";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

// The sound effects' stack and, under it, the songs' rail, each in a lane
// of its own on the strip, shown where it has clips. While sounds dragged
// in from the system are carried over the editor both show, an empty one
// with room for a row, and the lane under the pointer says what a sound let
// go there becomes, and the time under it where it starts
// (`sound-drop-target.ts`); where they would land is drawn over it.

type AudioLanesProps = Omit<AudioStackProps, keyof StripAudio | "clips"> & {
  audio?: StripAudio;
  /** Where a song's dragged end has snapped, or null, for the guide. */
  onGuide?: (seconds: number | null) => void;
};

/** How far, in px, past a lane's top and bottom a sound let go still lands
 * on it, so the gap between lanes is not a miss. */
const LANE_REACH_PX = 4;

export function AudioLanes({ audio, onGuide, ...lane }: AudioLanesProps) {
  const effects = audio?.clips.filter((clip) => clip.kind === "effect") ?? [];
  const songs = audio?.clips.filter((clip) => clip.kind === "music") ?? [];
  const carried = audio?.carried ?? null;
  const isCarried = carried !== null;
  const effectLaneRef = useRef<HTMLDivElement>(null);
  const songLaneRef = useRef<HTMLDivElement>(null);
  const laneRefOf = (kind: AudioClipKind) =>
    kind === "effect" ? effectLaneRef : songLaneRef;
  const scaleRef = useRef(lane.scale);
  scaleRef.current = lane.scale;
  useEffect(() => {
    if (!isCarried) return;
    return setSoundDropTarget((x, y) => {
      for (const kind of ["effect", "music"] as const) {
        const box = laneRefOf(kind).current?.getBoundingClientRect();
        if (
          !box ||
          box.width === 0 ||
          x < box.left ||
          x > box.right ||
          y < box.top - LANE_REACH_PX ||
          y > box.bottom + LANE_REACH_PX
        )
          continue;
        const scale = scaleRef.current;
        const seconds = scale.toSeconds((x - box.left) / box.width);
        return { atMs: scale.map.toPlayed(seconds) * 1_000, kind };
      }
      return null;
    });
  }, [isCarried]);
  const laneOf = (kind: AudioClipKind, children: React.ReactNode) => (
    <div
      className={cn(
        "relative rounded-part transition-colors",
        carried?.kind === kind && "bg-fill",
      )}
      ref={laneRefOf(kind)}
    >
      {children}
      {carried?.kind === kind ? (
        <SoundGhosts
          ghosts={carried.ghosts}
          scale={lane.scale}
          width={lane.width}
        />
      ) : null}
    </div>
  );
  return (
    <>
      <StripLane>
        {audio && (effects.length > 0 || isCarried)
          ? laneOf(
              "effect",
              <AudioStack {...audio} {...lane} clips={effects} />,
            )
          : null}
      </StripLane>
      <StripLane>
        {audio && (songs.length > 0 || isCarried)
          ? laneOf(
              "music",
              <SongRail
                {...lane}
                onGuide={onGuide}
                onRetime={audio.onRetime}
                songs={songs}
              />,
            )
          : null}
      </StripLane>
    </>
  );
}
