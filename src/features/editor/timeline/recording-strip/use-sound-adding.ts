// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type AudioClipKind,
  placedClip,
  type RecordingAudioClip,
  trimBounds,
} from "../../sound-track/audio-clips";
import {
  browseSounds,
  importSounds,
  type Sound,
} from "../../sound-track/sound-library";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";

import { type AudioClipWriter, type LatestEdit } from "./audio-clip-writer";
import { useSoundCarry } from "./use-sound-carry";

import type { SoundAddition } from "../../island/library/library-panel";

// Songs and sound effects arriving on the recording's timeline. Dragged in
// from the system or out of the library, they land on the lane they are let
// go on (`use-sound-carry.ts`). Pressed in the library, they come as the
// kind the library shows, by the placing rules (`sound-track/audio-clips.ts`):
// a sound effect at the playhead, songs one after another after the last
// song, or at the playhead where there is none. The first is chosen.

export function useSoundAdding({
  latestRef,
  library,
  playhead,
  select,
  timelineMs,
  writer,
}: {
  latestRef: LatestEdit;
  library: Sound[];
  playhead: Playhead;
  select: (next: StripSelection | null) => void;
  /** How long the edited video plays, in ms. */
  timelineMs: number;
  writer: AudioClipWriter;
}) {
  const carried = useSoundCarry({
    latestRef,
    playhead,
    select,
    timelineMs,
    writer,
  });
  /** Where the next song goes: after the last song, or at the playhead
   * where there is none. */
  const songsEnd = (clips: RecordingAudioClip[]) => {
    const ends = clips
      .filter((clip) => clip.kind === "music")
      .map((clip) => clip.endMs);
    return ends.length > 0 ? Math.max(...ends) : playhead.ratio() * timelineMs;
  };

  const place = (added: Sound[], kind: AudioClipKind) => {
    let next = latestRef.current.edit.audioClips ?? [];
    let songAt = songsEnd(next);
    const playheadMs = playhead.ratio() * timelineMs;
    let first: string | null = null;
    for (const sound of added) {
      const atMs = kind === "music" ? songAt : playheadMs;
      const clip = placedClip(next, sound, { atMs, kind, timelineMs });
      if (!clip) continue;
      next = [...next, clip];
      first ??= clip.id;
      if (kind === "music") songAt = clip.endMs;
    }
    if (first === null) return;
    writer.write(next);
    select({ id: first, kind: "audioClip" });
  };

  /** Gives the clip `id` another sound, chosen from a file. */
  const replaceSound = async (id: string) => {
    const paths = await browseSounds();
    const added = paths.length > 0 ? await importSounds(paths) : [];
    const sound = added[0] as Sound | undefined;
    if (!sound) return;
    writer.update(id, (clip, others) => {
      const replaced = { ...clip, offsetMs: 0, sound: sound.name };
      const { max } = trimBounds(others, replaced, {
        durationMs: sound.durationMs,
        edge: "end",
        timelineMs: Math.max(timelineMs, clip.endMs),
      });
      return {
        ...replaced,
        endMs: clip.isLooped ? Math.min(clip.endMs, max) : max,
      };
    });
  };

  const addSound = ({ kind, name }: SoundAddition) => {
    const sound = library.find((shelved) => shelved.name === name);
    if (sound) place([sound], kind);
  };

  const clips = latestRef.current.edit.audioClips ?? [];
  return {
    addSound,
    carried,
    hasSongRoom: songsEnd(clips) < timelineMs,
    replaceSound,
  };
}
