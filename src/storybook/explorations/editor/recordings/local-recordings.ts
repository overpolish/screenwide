// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Action } from "../../ideas/ideas-data";
import { pictureSpans, readingsOf } from "../model/picture-states";
import { recordingOf } from "../recording";

// Real recordings for the Your Recording story, from `local/`, which git
// ignores. Each is a folder holding what the capture wrote, made small
// enough for a browser:
//
// - `recording-picture.jsonl`, the picture readings the capture writes
//   beside the movie;
// - `recording.mp4`, the movie at 1280 wide, with a key frame every few
//   frames so it seeks quickly:
//   `ffmpeg -i recording.mov -vf scale=1280:-2 -c:v h264_videotoolbox -b:v 2500k -g 6 -an recording.mp4`
// - `thumbnails.jpg`, a frame a second, ten across, for the beats:
//   `ffmpeg -i recording.mov -vf "fps=1,scale=160:-2,tile=10x9" -frames:v 1 thumbnails.jpg`
// - `events.json`, if any: `[seconds, what]` pairs for what was done, until
//   the story reads the cursor and key files.

const readings = import.meta.glob<string>("./local/*/recording-picture.jsonl", {
  eager: true,
  import: "default",
  query: "?raw",
});
const videos = import.meta.glob<string>("./local/*/recording.mp4", {
  eager: true,
  import: "default",
  query: "?url",
});
const thumbnails = import.meta.glob<string>("./local/*/thumbnails.jpg", {
  eager: true,
  import: "default",
  query: "?url",
});
const events = import.meta.glob<[number, string][]>("./local/*/events.json", {
  eager: true,
  import: "default",
});

/** What a logged event was, as an action; `next` is the event after it,
 * which ends a typing run. */
function actionOf(
  [at, what]: [number, string],
  next?: [number, string],
): Action | null {
  if (what.startsWith("switch:"))
    return { at, kind: "switch", label: what.slice("switch:".length) };
  if (what === "open page") return { at, kind: "click", label: "Link" };
  if (what === "type")
    return {
      at,
      kind: "type",
      label: "Notes",
      seconds: next?.[1] === "typing ends" ? next[0] - at : undefined,
    };
  if (what === "scroll") return { at, kind: "keys", label: "↓" };
  return null;
}

export const LOCAL_RECORDINGS = Object.entries(readings).flatMap(
  ([path, text]) => {
    const folder = path.slice(0, path.lastIndexOf("/"));
    const video = videos[`${folder}/recording.mp4`] as string | undefined;
    if (!video) return [];
    const found = readingsOf(text);
    const sheet = thumbnails[`${folder}/thumbnails.jpg`] as string | undefined;
    const logged = events[`${folder}/events.json`] ?? [];
    return [
      {
        name: folder.slice(folder.lastIndexOf("/") + 1),
        recording: recordingOf({
          actions: logged.flatMap(
            (event, index) => actionOf(event, logged[index + 1]) ?? [],
          ),
          duration: found.length / 10,
          look: "screen",
          markTimes: {},
          media: {
            thumbnails: sheet
              ? { columns: 10, every: 1, url: sheet }
              : undefined,
            video,
          },
          moments: [],
          picture: pictureSpans(found),
          recordedPreset: "screen",
          rest: "still",
          scenes: [],
          sounds: [],
          speech: [],
        }),
      },
    ];
  },
);
