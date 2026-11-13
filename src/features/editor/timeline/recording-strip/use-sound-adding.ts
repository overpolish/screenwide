// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { editorContent } from "../../island/host/island-window-place";
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
import { type ToolPanelAddingSounds } from "../../tools/tool-panel-strip";
import { type Playhead } from "../scrub-playhead";
import { type StripSelection } from "../strip/strip-selection";

import { type AudioClipWriter, type LatestEdit } from "./audio-clip-writer";
import { soundDropArea, useSoundDrag } from "./use-sound-drag";

import type { SoundAddition } from "../../island/sound/sound-library-list";

// Songs and sound effects arriving on the recording's timeline: dragged in
// from the system onto the timeline (`use-sound-drag.ts`), chosen in the
// Sound island's library, or added there from a file. They wait while the
// island window asks whether they are songs or sound effects
// (`sound-kind-chooser.tsx`), then land by the placing rules
// (`sound-track/audio-clips.ts`): a sound effect at the playhead, songs one
// after another after the last song, or at the playhead where there is
// none. The first is chosen. A drag chooses by where it is let go; the
// others by the island pressed.

/** Sounds waiting to be told what they are: files not yet in the library,
 * or sounds already there. */
type Waiting = { isCarried: boolean } & (
  { paths: string[] } | { sounds: Sound[] }
);

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
  const [waiting, setWaiting] = useState<Waiting | null>(null);
  // Read by the drag's events, which can come twice before the page draws
  // again: the sounds are taken once, by the first.
  const waitingRef = useRef<Waiting | null>(null);
  const keepWaiting = (next: Waiting | null) => {
    waitingRef.current = next;
    setWaiting(next);
  };
  const [over, setOver] = useState<AudioClipKind | null>(null);
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

  const chooseSoundKind = (kind: AudioClipKind | null) => {
    const was = waitingRef.current;
    keepWaiting(null);
    setOver(null);
    const clips = latestRef.current.edit.audioClips ?? [];
    if (kind === "music" && songsEnd(clips) >= timelineMs) return;
    if (!was || kind === null) return;
    if ("sounds" in was) {
      place(was.sounds, kind);
      return;
    }
    importSounds(was.paths)
      .then((added) => {
        place(added, kind);
      })
      .catch((cause: unknown) => {
        console.error("Could not add the sounds", cause);
      });
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

  const addSound = (addition: SoundAddition) => {
    if ("name" in addition) {
      const sound = library.find(({ name }) => name === addition.name);
      if (sound) keepWaiting({ isCarried: false, sounds: [sound] });
      return;
    }
    browseSounds()
      .then((paths) => {
        if (paths.length > 0) keepWaiting({ isCarried: false, paths });
      })
      .catch((cause: unknown) => {
        console.error("Could not add the sounds", cause);
      });
  };

  useSoundDrag({
    onCarry: (paths, at) => {
      keepWaiting({ isCarried: true, paths });
      setOver(at);
    },
    onDrop: (kind) => {
      if (waitingRef.current?.isCarried) chooseSoundKind(kind);
    },
    onOver: setOver,
  });

  const clips = latestRef.current.edit.audioClips ?? [];
  const area = soundDropArea()?.getBoundingClientRect();
  const content = editorContent()?.getBoundingClientRect();
  const adding: ToolPanelAddingSounds | null = waiting && {
    area:
      area && content
        ? {
            height: area.height,
            left: area.left - content.left,
            top: area.top - content.top,
            width: area.width,
          }
        : null,
    count: "paths" in waiting ? waiting.paths.length : waiting.sounds.length,
    hasSongRoom: songsEnd(clips) < timelineMs,
    isCarried: waiting.isCarried,
    over: waiting.isCarried ? over : null,
  };
  return { addSound, adding, chooseSoundKind, replaceSound };
}
