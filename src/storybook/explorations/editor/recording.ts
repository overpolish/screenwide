// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe } from "../ideas/action-script-lines";
import { type Action } from "../ideas/ideas-data";

import { type SceneClip, type ScenePreset } from "./editor-data";

// The recording the editor edits, as the app would hand it over: its
// length, and what was read from it. Every recording has the same readings,
// whatever it is of; which of them say much differs:
//
// - what was done, from the cursor and key files
//   (`src-tauri/src/editor/auto_zoom/signals.rs`), on a screen recording;
// - what was said, from the transcript;
// - what the picture did, read from its frames (`screenwide-activity`): still,
//   moving, or new, against the recording's own level;
// - loud moments in its sound, and the moments flagged while recording
//   (`screenwide-moments`).
//
// Something done, said, flagged, loud or new must play as it was: a hold,
// and a beat on the strip. Between beats is give, springs: what moves gives
// a little, what is still gives most. Recordings in `recordings/`.

export type Span = { end: number; start: number };
export type SpokenLine = Span & { text: string };

/** What the picture is, which says how fast its springs can play before
 * it shows: a still screen hides any speed, play in a game blurs past twice
 * its pace, and a face looks wrong much past its own. */
export type Look = "camera" | "game" | "screen";

/** A stretch of the picture, read from its frames. Outside every stretch
 * the picture is at its `rest`. `label` says what a game shows there. */
export type PictureSpan = Span & {
  state: "moving" | "new" | "still";
  label?: string;
};

/** What a beat that is no action or line came from. */
export type Cue = "moment" | "new" | "sound";

/** A stretch of the recording to edit by: something done, said, flagged,
 * heard or newly shown. */
export type Beat =
  | { action: Action; end: number; id: string; kind: "action"; start: number }
  | {
      at: number;
      cue: Cue;
      end: number;
      id: string;
      kind: "cue";
      start: number;
    }
  | { end: number; id: string; kind: "speech"; start: number; text: string };

/** A real recording's own picture, where it has one: the movie, and a
 * sheet of frames for thumbnails, one every `every` seconds, `columns`
 * across. Without it the picture is drawn. */
export type RecordedMedia = {
  video: string;
  thumbnails?: { columns: number; every: number; url: string };
};

export type RecordingReadings = {
  actions: Action[];
  duration: number;
  look: Look;
  /** When each seeded annotation (`look-pens-model.ts`) shows, by its id. */
  markTimes: Record<number, Span>;
  /** When a moment was flagged while recording. */
  moments: number[];
  picture: PictureSpan[];
  /** The composition the recording was made with, outside every scene. */
  recordedPreset: ScenePreset;
  /** What the picture is doing outside its stretches. */
  rest: "moving" | "still";
  /** The scenes auto zoom proposes (`src-tauri/src/editor/auto_zoom`). */
  scenes: SceneClip[];
  /** Where the sound jumps above its own level: a shot, a crash, a cheer. */
  sounds: number[];
  speech: SpokenLine[];
  media?: RecordedMedia;
};

export type Recording = RecordingReadings & {
  actionBeats: Beat[];
  /** Beats from flagged moments, loud sounds and new pictures. */
  cueBeats: Beat[];
  speechBeats: Beat[];
  /** Every word with the moment it starts, spread over its line. */
  words: { at: number; word: string }[];
};

/** How long before each action the pointer sets off, and after it the work
 * settles, in seconds. */
const LEAD: Record<Action["kind"], [number, number]> = {
  click: [1.4, 0.5],
  idle: [0, 0],
  keys: [0.4, 0.5],
  switch: [0.4, 0.6],
  type: [0.6, 1.2],
};

/** How much a flagged moment and a loud one keep around them, in seconds:
 * what led up to it and its outcome. */
const AROUND: Record<Exclude<Cue, "new">, [number, number]> = {
  moment: [4, 2],
  sound: [1.5, 1.5],
};

const cueBeat = (cue: Cue, at: number, [before, after]: [number, number]) => ({
  at,
  cue,
  end: at + after,
  id: `${cue}${String(at)}`,
  kind: "cue" as const,
  start: at - before,
});

export function recordingOf(readings: RecordingReadings): Recording {
  return {
    ...readings,
    actionBeats: readings.actions
      .filter(({ kind }) => kind !== "idle")
      .sort((first, second) => first.at - second.at)
      .reduce<Beat[]>((beats, action) => {
        // Actions close together would overlap: the pointer sets off for
        // the next one only once the last has settled.
        const last = beats[beats.length - 1] as Beat | undefined;
        beats.push({
          action,
          // A typing run lasts as long as the keys say it did.
          end: action.at + Math.max(LEAD[action.kind][1], action.seconds ?? 0),
          id: `a${String(action.at)}`,
          kind: "action",
          start: Math.max(action.at - LEAD[action.kind][0], last?.end ?? 0),
        });
        return beats;
      }, []),
    cueBeats: [
      ...readings.picture
        .filter(({ state }) => state === "new")
        .map(({ end, start }) => cueBeat("new", start, [0, end - start])),
      ...readings.moments.map((at) => cueBeat("moment", at, AROUND.moment)),
      ...readings.sounds.map((at) => cueBeat("sound", at, AROUND.sound)),
    ],
    speechBeats: readings.speech.map((line) => ({
      end: line.end,
      id: `s${String(line.start)}`,
      kind: "speech",
      start: line.start,
      text: line.text,
    })),
    words: readings.speech.flatMap((line) => {
      const words = line.text.split(" ");
      const step = (line.end - line.start) / words.length;
      return words.map((word, index) => ({
        at: line.start + index * step,
        word,
      }));
    }),
  };
}

/** The spoken line over a moment, if there is one. */
export const lineAt = (recording: Recording, seconds: number) =>
  recording.speech.find(({ end, start }) => seconds >= start && seconds < end);

/** The stretch of picture over a moment, if it is not at rest. */
export const pictureAt = (recording: Recording, seconds: number) =>
  recording.picture.find(({ end, start }) => seconds >= start && seconds < end);

/** Where choosing a beat takes the playhead: to what it is about. */
export const beatAt = (beat: Beat) =>
  beat.kind === "action"
    ? beat.action.at
    : beat.kind === "cue"
      ? beat.at
      : beat.start;

const CUE_TITLES: Record<Cue, string> = {
  moment: "Flagged moment",
  new: "New screen",
  sound: "Loud moment",
};

export const beatTitle = (beat: Beat) =>
  beat.kind === "action"
    ? describe(beat.action)
    : beat.kind === "cue"
      ? CUE_TITLES[beat.cue]
      : beat.text;
