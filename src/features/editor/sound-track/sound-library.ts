// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useSyncExternalStore } from "react";

import type { AudioClipKind } from "../../../bindings/AudioClipKind";
import type { Sound } from "../../../bindings/Sound";

// The sound library as this window knows it: every song and sound effect
// added, read once and again whenever it changes, so the timeline's lanes
// draw each clip's waveform and the islands name it. The library itself
// lives in the app's data folder (`screenwide_sound_track::store`); a sound
// taken out of it is still found in the projects that carry a copy.

export type { Sound };

/** The kinds of file the library takes, as `store::EXTENSIONS` lists them. */
const EXTENSIONS: readonly string[] = [
  "aac",
  "aif",
  "aiff",
  "caf",
  "flac",
  "m4a",
  "mp3",
  "mp4",
  "oga",
  "ogg",
  "opus",
  "wav",
];

let library: Sound[] = [];
let isRead = false;
const listeners = new Set<() => void>();

const notify = () => {
  for (const listener of listeners) listener();
};

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

const keep = (sounds: Sound[]) => {
  const added = new Set(sounds.map(({ name }) => name));
  library = [...sounds, ...library.filter(({ name }) => !added.has(name))];
  notify();
};

/** Whether `path` names a kind of file the library takes. */
export const isSoundPath = (path: string) =>
  EXTENSIONS.includes(path.split(".").pop()?.toLowerCase() ?? "");

/** Adds the sounds at `paths` to the library, listed as `kind` where they
 * were added under one, and says what each is. */
export async function importSounds(paths: string[], kind?: AudioClipKind) {
  const sounds = await invoke<Sound[]>("import_sounds", {
    kind: kind ?? null,
    paths,
  });
  keep(sounds);
  return sounds;
}

/** Asks for sounds to add; none where nothing was chosen. */
export const browseSounds = () => invoke<string[]>("browse_sounds");

/** Takes the sound `name` out of the library; projects that play it keep
 * their own copy. */
export const removeSound = (name: string) =>
  invoke<null>("remove_sound", { name });

/** Takes the sound `name`, which the open recording carries but the
 * library lacks, into the library as `kind`, and says what it is there. */
export async function keepProjectSound(name: string, kind: AudioClipKind) {
  const sound = await invoke<Sound>("keep_project_sound", { kind, name });
  keep([sound]);
  return sound;
}

/** Starts dragging the sound `name`'s file out of this window, from the
 * press under way, onto the lanes or into another app, drawn as `look`. */
export const dragLibrarySound = (
  name: string,
  look: { height: number; png: number[]; width: number },
) => invoke<null>("drag_library_sound", { name, ...look });

/** Gives the sound `name` the title `title` in the library. */
export const renameSound = (name: string, title: string) =>
  invoke<null>("rename_sound", { name, title });

/** Says whether sounds let go where the pointer is now would be refused,
 * which the drag shows (`sound_library::drop_refusal`). */
export const setSoundDropRefused = (refused: boolean) =>
  invoke<null>("set_sound_drop_refused", { refused });

/** The longest stretch a preview plays, in ms, as the editor's
 * `sound_library::preview` cuts it. */
export const MOST_PREVIEW_MS = 30_000;

/** `lengthMs` of the sound `name` from `fromMs` in, at most the longest a
 * preview plays, as a WAV for the island to play. */
export const previewSound = (name: string, fromMs: number, lengthMs: number) =>
  invoke<ArrayBuffer>("get_sound_preview", { fromMs, lengthMs, name });

/** Reads the library again, keeps it for this window, and says what it
 * holds. */
export const readSoundLibrary = () =>
  invoke<Sound[]>("get_sound_library").then((sounds) => {
    library = sounds;
    notify();
    return sounds;
  });

const read = () => {
  readSoundLibrary().catch((cause: unknown) => {
    console.error("Could not read the sound library", cause);
  });
};

/** Every sound in the library, read once per window, and again whenever any
 * window adds or removes one (`editor://sound-library`). */
export function useSoundLibrary() {
  useEffect(() => {
    if (isRead) return;
    isRead = true;
    read();
    void listen("editor://sound-library", read);
  }, []);
  return useSyncExternalStore(subscribe, () => library);
}

/** Sounds this window has found in the open recording's own copies. */
const carried = new Map<string, Sound>();

/** Each of `names` as the library or, for one it no longer holds, the open
 * recording's own copy has it, by name. */
export function useClipSounds(names: string[]) {
  const shelved = useSoundLibrary();
  useSyncExternalStore(subscribe, () => carried.size);
  const missing = [...new Set(names)]
    .filter(
      (name) =>
        !carried.has(name) && !shelved.some((sound) => sound.name === name),
    )
    .sort()
    .join("\n");
  useEffect(() => {
    if (missing === "") return;
    invoke<Sound[]>("get_project_sounds", { names: missing.split("\n") })
      .then((sounds) => {
        for (const sound of sounds) carried.set(sound.name, sound);
        notify();
      })
      .catch((cause: unknown) => {
        console.error("Could not read the project's sounds", cause);
      });
  }, [missing]);
  return new Map([
    ...carried,
    ...shelved.map((sound) => [sound.name, sound] as const),
  ]);
}
