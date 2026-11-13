// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import {
  type AudioClipKind,
  placedClip,
  type RecordingAudioClip,
} from "../../sound-track/audio-clips";
import { droppedClips, guessedKind } from "../../sound-track/dropped-clips";
import {
  importSounds,
  setSoundDropRefused,
  type Sound,
} from "../../sound-track/sound-library";
import {
  type SoundDropSpot,
  soundDropSpot,
} from "../audio-lane/sound-drop-target";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";

import { type AudioClipWriter, type LatestEdit } from "./audio-clip-writer";
import { useSoundDrag } from "./use-sound-drag";

// Sound files dragged in from the system, or out of the library, and let
// go on the timeline. They are read into the library as they arrive, so
// where they would land can be drawn at their length while they are
// carried. Let go on a lane, they become what that lane holds, from the
// moment under the pointer, one after another (`dropped-clips.ts`). Off the
// lanes, or on the song rail with no room for them, the drag shows the drop
// refused and nothing is added. Let go on the picture, which knows no
// lanes, each goes by its length to the playhead, as a sound added from the
// library does. The first added is chosen.

/** Sounds on their way in: their files, the sounds read from them once they
 * are, and the spot on the lanes under the pointer. */
type Carry = {
  paths: string[];
  reading: Promise<Sound[]>;
  sounds: Sound[] | null;
  spot: SoundDropSpot | null;
};

/** A clip a carried sound would make, and its title, which the library
 * the strip reads may not hold yet. */
export type CarriedClip = RecordingAudioClip & { title: string };

/** The file's name without its folder or extension, a carried sound's
 * title until it is read. */
const fileTitle = (path: string) =>
  (path.split(/[\\/]/).pop() ?? path).replace(/\.[^.]+$/, "");

const isSameSpot = (one: SoundDropSpot | null, other: SoundDropSpot | null) =>
  one?.kind === other?.kind &&
  Math.round(one?.atMs ?? 0) === Math.round(other?.atMs ?? 0);

export function useSoundCarry({
  latestRef,
  playhead,
  select,
  timelineMs,
  writer,
}: {
  latestRef: LatestEdit;
  playhead: Playhead;
  select: (next: StripSelection | null) => void;
  /** How long the edited video plays, in ms. */
  timelineMs: number;
  writer: AudioClipWriter;
}) {
  const [carry, setCarry] = useState<Carry | null>(null);
  // Read by the drag's events, which can come twice before the page draws
  // again.
  const carryRef = useRef<Carry | null>(null);
  const clipsNow = () => latestRef.current.edit.audioClips ?? [];
  // Whether the drag now shows a drop refused: off the lanes, or on the song
  // rail with no room for what is carried, nothing would be added.
  const refusedRef = useRef(false);
  const showRefused = (next: Carry | null) => {
    const refused =
      next !== null &&
      (next.spot === null ||
        (next.sounds !== null &&
          droppedClips(clipsNow(), next.sounds, { ...next.spot, timelineMs })
            .added.length === 0));
    if (refused === refusedRef.current) return;
    refusedRef.current = refused;
    setSoundDropRefused(refused).catch(failed);
  };
  const keep = (next: Carry | null) => {
    carryRef.current = next;
    setCarry(next);
    showRefused(next);
  };
  const write = (clips: RecordingAudioClip[], first: string | undefined) => {
    if (first === undefined) return;
    writer.write(clips);
    select({ id: first, kind: "audioClip" });
  };
  function failed(cause: unknown) {
    console.error("Could not add the sounds", cause);
  }
  /** Reads `paths` into the library, keeping what they are on the carry
   * that asked for them, if it is still carried. */
  const read = (paths: string[]) => {
    const reading = importSounds(paths);
    reading
      .then((sounds) => {
        const now = carryRef.current;
        if (now?.reading === reading) keep({ ...now, sounds });
      })
      .catch(failed);
    return reading;
  };
  const readOf = (paths: string[]) => {
    const now = carryRef.current;
    return now && now.paths.join() === paths.join() ? now.reading : null;
  };

  useSoundDrag({
    onCarry: (paths) => {
      if (readOf(paths)) return;
      keep({ paths, reading: read(paths), sounds: null, spot: null });
    },
    onDrop: ({ x, y }) => {
      const was = carryRef.current;
      keep(null);
      const spot = soundDropSpot(x, y);
      if (!was || !spot) return;
      was.reading
        .then((sounds) => {
          const { added, clips } = droppedClips(clipsNow(), sounds, {
            ...spot,
            timelineMs,
          });
          write(clips, added[0]?.id);
        })
        .catch(failed);
    },
    onLeave: () => {
      keep(null);
    },
    onMove: (point) => {
      const was = carryRef.current;
      const spot = point && soundDropSpot(point.x, point.y);
      if (was && !isSameSpot(was.spot, spot)) keep({ ...was, spot });
    },
    onPictureDrop: (paths) => {
      const reading = readOf(paths) ?? importSounds(paths);
      keep(null);
      reading
        .then((sounds) => {
          let clips = clipsNow();
          const atMs = playhead.ratio() * timelineMs;
          let first: string | undefined;
          for (const sound of sounds) {
            const kind = guessedKind(sound);
            const clip = placedClip(clips, sound, { atMs, kind, timelineMs });
            if (!clip) continue;
            clips = [...clips, clip];
            first ??= clip.id;
          }
          write(clips, first);
        })
        .catch(failed);
    },
  });

  // Where the carried sounds would land if let go now, drawn on the lane
  // under the pointer: at their lengths once read, as stubs before.
  const spot = carry?.spot ?? null;
  const ghosts = ((): CarriedClip[] => {
    if (!carry || !spot) return [];
    if (!carry.sounds)
      return carry.paths.map((path, index) => ({
        ...placedStub(spot, index),
        title: fileTitle(path),
      }));
    const titles = new Map(
      carry.sounds.map(({ name, title }) => [name, title]),
    );
    return droppedClips(clipsNow(), carry.sounds, {
      ...spot,
      timelineMs,
    }).added.map((clip) => ({ ...clip, title: titles.get(clip.sound) ?? "" }));
  })();
  const carried: { ghosts: CarriedClip[]; kind: AudioClipKind | null } | null =
    carry && { ghosts, kind: spot?.kind ?? null };
  return carried;
}

/** A sound not yet read, drawn as a stub where it would start. */
const placedStub = ({ atMs, kind }: SoundDropSpot, index: number) => ({
  endMs: Math.round(atMs) + 1,
  id: `carried-${String(index)}`,
  isInstrumental: false,
  isLooped: false,
  kind,
  lane: 0,
  offsetMs: 0,
  sound: "",
  startMs: Math.round(atMs),
  volumeDb: 0,
});
