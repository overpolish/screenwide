// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  type CaptureParts,
  captureParts,
  editorModeGroups,
  type EditorKind,
  NO_CAPTURE_PARTS,
  offeredModes,
  type RecordedParts,
  recordedTrackCount,
} from "./editor-modes";

import type { RecordingVideoTrackId } from "../types";

const EVERYTHING: CaptureParts = {
  camera: true,
  cursor: true,
  keyboard: true,
  microphone: true,
  picture: "screen",
  transcript: "heard",
};

/** The modes offered, each group a string of ids. */
const offered = (
  parts: Partial<CaptureParts>,
  kind: EditorKind = "recording",
) =>
  editorModeGroups({ kind, parts: { ...EVERYTHING, ...parts } }).map(
    ({ id, modes }) => `${id}: ${modes.map((mode) => mode.id).join(" ")}`,
  );

describe("editorModeGroups", () => {
  it("offers a screenshot its Frame and Screenshot, always", () => {
    expect(offered(NO_CAPTURE_PARTS, "screenshot")).toEqual([
      "picture: frame screen",
    ]);
  });

  it("offers every mode for a recording that holds everything", () => {
    expect(offered({})).toEqual([
      "picture: frame screen camera scene",
      "done: cursor keyboard",
      "heard: captions transcript sound",
    ]);
  });

  it("takes the camera with it when it is left out or never recorded", () => {
    expect(offered({ camera: false })[0]).toBe("picture: frame screen scene");
  });

  it("takes Screen, Scene, Cursor and Keyboard with the screen", () => {
    expect(offered({ picture: null })).toEqual([
      "picture: frame camera",
      "heard: captions transcript sound",
    ]);
  });

  it("offers Camera, no Screen, and no Cursor or Keyboard over a camera-only recording's picture", () => {
    expect(offered({ camera: false, picture: "camera" })).toEqual([
      "picture: frame camera scene",
      "heard: captions transcript sound",
    ]);
  });

  it("offers Cursor and Keyboard only where each was recorded", () => {
    expect(offered({ cursor: false })[1]).toBe("done: keyboard");
    expect(offered({ keyboard: false })[1]).toBe("done: cursor");
    expect(offered({ cursor: false, keyboard: false })).toEqual([
      "picture: frame screen camera scene",
      "heard: captions transcript sound",
    ]);
  });

  it("offers Captions and Transcript while the transcript is still to make", () => {
    expect(offered({ transcript: "pending" })[2]).toBe(
      "heard: captions transcript sound",
    );
  });

  it("takes Captions and Transcript away once the transcript heard nothing", () => {
    expect(offered({ transcript: "silent" })[2]).toBe("heard: sound");
  });

  it("takes Captions and Transcript away without the microphone", () => {
    expect(offered({ microphone: false })[2]).toBe("heard: sound");
  });

  it("offers no picture's modes, nor Captions, without a picture", () => {
    const audio = { camera: false, picture: null };
    expect(offered(audio, "audio")).toEqual(["heard: transcript sound"]);
    expect(offered(audio)).toEqual(["heard: transcript sound"]);
  });

  it("lists every mode offered for the keys", () => {
    const modes = offeredModes(
      editorModeGroups({
        kind: "recording",
        parts: { ...EVERYTHING, picture: null },
      }),
    );
    expect(modes.has("camera")).toBe(true);
    expect(modes.has("cursor")).toBe(false);
  });
});

const RECORDED: RecordedParts = {
  audio: [
    { kind: "microphone", streamIndex: 1 },
    { kind: "system-audio", streamIndex: 2 },
  ],
  camera: true,
  cursor: true,
  keyboard: true,
  picture: "screen",
};

const inExport = (
  streams: number[],
  video: RecordingVideoTrackId[],
  recorded = RECORDED,
) =>
  captureParts({
    enabledStreams: new Set(streams),
    enabledVideoTracks: new Set(video),
    recorded,
    transcript: "heard",
  });

describe("captureParts", () => {
  it("holds what was recorded while every track is in", () => {
    expect(inExport([1, 2], ["primary", "camera"])).toEqual(EVERYTHING);
  });

  it("leaves out the tracks switched off", () => {
    expect(inExport([2], ["camera"])).toMatchObject({
      camera: true,
      microphone: false,
      picture: null,
    });
    expect(inExport([], ["primary"])).toMatchObject({
      camera: false,
      microphone: false,
      picture: "screen",
    });
  });

  it("holds no camera beside the picture where none was recorded", () => {
    expect(
      inExport([1], ["primary", "camera"], { ...RECORDED, camera: false })
        .camera,
    ).toBe(false);
  });
});

describe("recordedTrackCount", () => {
  it("counts the picture, the camera and each audio stream", () => {
    expect(recordedTrackCount(RECORDED)).toBe(4);
    expect(
      recordedTrackCount({
        ...RECORDED,
        audio: [],
        camera: false,
        picture: "camera",
      }),
    ).toBe(1);
  });
});
