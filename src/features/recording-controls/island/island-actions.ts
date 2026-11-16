// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { startNotes } from "../../notes/api";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { useRecordingSourceStore } from "../../recording-sources/store";
import { startRecording } from "../api";
import { notesOptions, startRecordingOptions } from "../recording-request";

import { changeRecordingMode } from "./use-recording-ui-session";

import type { IslandMode } from "./island-types";

/** Chooses `mode` on the island: No screen stands the recorder in its
 * camera mode while the camera is on and its audio mode when not, and Notes
 * in its audio mode, so nothing shows on the screen. */
export const chooseIslandMode = (mode: IslandMode) => {
  const { setNotes } = useRecordingSourceStore.getState();
  const isCameraOn = useRecordingInputStore.getState().inputs.camera;
  setNotes(mode === "notes");
  changeRecordingMode(
    mode === "none"
      ? isCameraOn
        ? "camera"
        : "audio"
      : mode === "notes"
        ? "audio"
        : mode,
  );
};

/** What Record does: takes notes while Notes is chosen, and otherwise starts
 * a recording. */
export const recordOrTakeNotes = () => {
  if (useRecordingSourceStore.getState().isNotes) {
    startNotes(notesOptions()).catch((error: unknown) => {
      console.error("Could not start taking notes", error);
    });
    return;
  }
  startRecordingOptions()
    .then(startRecording)
    .catch((error: unknown) => {
      console.error("Could not start the recording", error);
    });
};
