// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { kindOf, placedClip, trimBounds } from "../../sound-track/audio-clips";
import {
  browseSounds,
  importSounds,
  type Sound,
} from "../../sound-track/sound-library";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";

import { type AudioClipWriter, type LatestEdit } from "./audio-clip-writer";

import type { SoundAddition } from "../../island/sound/sound-library-list";

// Songs and sound effects arriving on the recording's timeline: dropped on
// the strip or on the picture, chosen in the Sound island's library or
// added there from a file, or taking a clip's place. Each lands by the
// placing rules (`sound-track/audio-clips.ts`) and the first is chosen.

/** The event the editor's picture tells of sounds dropped on it
 * (`editor/images/dropped.rs`). */
const SOUND_DROP_EVENT = "editor://sound-drop";

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
  /** Lays `added` on the timeline: from `atMs` where they were dropped on
   * the strip, songs one after another; elsewhere a song after the last
   * song, or at the playhead where there is none, and a sound effect at
   * the playhead. The first is chosen. */
  const place = (added: Sound[], atMs: number | null) => {
    let next = latestRef.current.edit.audioClips ?? [];
    const playheadMs = playhead.ratio() * timelineMs;
    const songsEnd = () => {
      const ends = next
        .filter((clip) => clip.kind === "music")
        .map((clip) => clip.endMs);
      return ends.length > 0 ? Math.max(...ends) : playheadMs;
    };
    let songAt = atMs;
    let first: string | null = null;
    for (const sound of added) {
      const isSong = kindOf(sound) === "music";
      const at = isSong ? (songAt ?? songsEnd()) : (atMs ?? playheadMs);
      const clip = placedClip(next, sound, { atMs: at, timelineMs });
      if (!clip) continue;
      next = [...next, clip];
      first ??= clip.id;
      if (isSong) songAt = clip.endMs;
    }
    if (first === null) return;
    writer.write(next);
    select({ id: first, kind: "audioClip" });
  };

  const imported = (paths: string[], atMs: number | null) => {
    importSounds(paths)
      .then((added) => {
        place(added, atMs);
      })
      .catch((cause: unknown) => {
        console.error("Could not add the sounds", cause);
      });
  };

  const fromDisk = async () => {
    const paths = await browseSounds();
    return paths.length > 0 ? importSounds(paths) : [];
  };

  /** Gives the clip `id` another sound, chosen from a file. */
  const replaceSound = async (id: string) => {
    const added = await fromDisk();
    if (added.length === 0) return;
    const sound = added[0];
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

  const addSound = (addition: SoundAddition) => {
    if ("name" in addition) {
      const sound = library.find(({ name }) => name === addition.name);
      if (sound) place([sound], null);
      return;
    }
    fromDisk()
      .then((added) => {
        place(added, null);
      })
      .catch((cause: unknown) => {
        console.error("Could not add the sounds", cause);
      });
  };

  // Dropped on the picture, sounds land by the rules for a drop off the
  // strip.
  const droppedRef = useRef(imported);
  droppedRef.current = imported;
  useEffect(() => {
    let isDisposed = false;
    let unlisten: (() => void) | undefined;
    void listen<{ paths: string[] }>(SOUND_DROP_EVENT, ({ payload }) => {
      droppedRef.current(payload.paths, null);
    }).then((stop) => {
      if (isDisposed) stop();
      else unlisten = stop;
    });
    return () => {
      isDisposed = true;
      unlisten?.();
    };
  }, []);

  return {
    addSound,
    /** Told sounds dropped on the strip, at a moment in seconds, or
     * elsewhere in the window. */
    onAudioDrop: (paths: string[], at: number | null) => {
      imported(paths, at === null ? null : Math.round(at * 1000));
    },
    replaceSound,
  };
}
