// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { dictationFor } from "./settings-dictation-fixtures";

import type { PermissionSnapshot } from "../permissions/types";
import type {
  TranscriptionModel,
  TranscriptionState,
} from "../transcription/types";

/** Every permission granted, or none yet asked for, as dictation's row in
 * Models reads them. */
export const accessSnapshot = (granted: boolean): PermissionSnapshot => {
  const status = { canRequest: true, granted };
  return {
    accessibility: status,
    camera: status,
    microphone: status,
    screenRecording: status,
  };
};

/** The Transcripts model, at its real download size, which also transcribes
 * voice notes. */
const TRANSCRIPTS_MODEL: TranscriptionModel = {
  description: "Turns your recordings and voice notes into text.",
  id: "large-v3-turbo-q8_0",
  name: "Transcripts",
  progress: null,
  purpose: "transcripts",
  sizeBytes: 874_188_075,
  status: "available",
  writes: null,
};

/** The Restore voice model, at its real download size, every file together. */
const RESTORE_VOICE_MODEL: TranscriptionModel = {
  description: "Makes your voice sound like a studio recording.",
  id: "unipase",
  name: "Restore voice",
  progress: null,
  purpose: "restoreVoice",
  sizeBytes: 1_123_349_858,
  status: "available",
  writes: null,
};

/** The Smart ducking model, at its real download size. */
const SMART_DUCKING_MODEL: TranscriptionModel = {
  description: "Keeps your voice clear over other audio.",
  id: "mel-roformer-vocals",
  name: "Smart ducking",
  progress: null,
  purpose: "smartDucking",
  sizeBytes: 165_612_636,
  status: "available",
  writes: null,
};

/** The background removal model, at its real download size. */
const CAMERA_BACKGROUND_MODEL: TranscriptionModel = {
  description: "Removes the background behind you on camera.",
  id: "rvm-mobilenetv3",
  name: "Background removal",
  progress: null,
  purpose: "cameraBackground",
  sizeBytes: 14_975_696,
  status: "available",
  writes: null,
};

/** The object tracking model, at its real download size. */
const CAMERA_OBJECTS_MODEL: TranscriptionModel = {
  description: "Keeps or removes objects on camera.",
  id: "sam2.1-hiera-tiny",
  name: "Camera objects",
  progress: null,
  purpose: "cameraObjects",
  sizeBytes: 190_082_794,
  status: "available",
  writes: null,
};

/** The meeting notes models, Parakeet Ultra and the speaker model, at their
 * real download size together. */
const NOTES_MODEL: TranscriptionModel = {
  description: "Writes notes of your calls and tells the speakers apart.",
  id: "notes-ultra-nemotron-3-diarization-q8_0",
  name: "Meeting notes",
  progress: null,
  purpose: "notes",
  sizeBytes: 876_972_274,
  status: "available",
  writes: null,
};

/** How the Transcripts model stands, and the language transcription and
 * dictation are set to, `system` (English) unless given. */
export type TranscriptionPreviewSeed = Pick<
  TranscriptionModel,
  "progress" | "status"
> & { language?: string };

/** How long a story's pretend download takes, in milliseconds. */
const DOWNLOAD_MS = 4_000;
const TICK_MS = 100;

/**
 * The transcription side of the Settings stories' API: the models kept in
 * memory, Transcripts as `seed` says and the others not yet downloaded, with
 * downloads that fill over a few seconds; dictation's model and commands
 * those of the seed's language. Nothing reaches the app or the network.
 */
export function transcriptionPreviewApi(
  seed: TranscriptionPreviewSeed = { progress: null, status: "downloaded" },
) {
  const { language = "system", ...transcriptsSeed } = seed;
  const transcripts = { ...TRANSCRIPTS_MODEL, ...transcriptsSeed };
  const { dictationCommands, models: dictation } = dictationFor(
    language,
    transcripts,
  );
  let state: TranscriptionState = {
    dictationCommands,
    language,
    models: [
      transcripts,
      ...dictation,
      NOTES_MODEL,
      RESTORE_VOICE_MODEL,
      SMART_DUCKING_MODEL,
      CAMERA_BACKGROUND_MODEL,
      CAMERA_OBJECTS_MODEL,
    ],
    systemLanguage: "en",
    words: ["Screenwide", "Overpolish"],
  };
  const listeners = new Set<(next: TranscriptionState) => void>();
  const downloads = new Map<string, number>();
  const publish = (next: TranscriptionState) => {
    state = next;
    for (const listener of listeners) listener(state);
  };
  const withModel = (id: string, change: Partial<TranscriptionModel>) => ({
    ...state,
    models: state.models.map((model) =>
      model.id === id ? { ...model, ...change } : model,
    ),
  });
  const stopDownload = (id: string) => {
    window.clearInterval(downloads.get(id));
    downloads.delete(id);
  };
  return {
    cancelTranscriptionDownload: (id: string) => {
      stopDownload(id);
      publish(withModel(id, { progress: null, status: "available" }));
      return Promise.resolve(null);
    },
    downloadTranscriptionModel: (id: string) => {
      publish(withModel(id, { progress: 0, status: "downloading" }));
      downloads.set(
        id,
        window.setInterval(() => {
          const model = state.models.find((candidate) => candidate.id === id);
          const progress = Math.min(
            1,
            (model?.progress ?? 0) + TICK_MS / DOWNLOAD_MS,
          );
          if (progress < 1) {
            publish(withModel(id, { progress }));
            return;
          }
          stopDownload(id);
          publish(withModel(id, { progress: null, status: "downloaded" }));
        }, TICK_MS),
      );
      return Promise.resolve(null);
    },
    getTranscriptionState: () => Promise.resolve(state),
    listenToTranscription: (onChange: (next: TranscriptionState) => void) => {
      listeners.add(onChange);
      return Promise.resolve(() => {
        listeners.delete(onChange);
      });
    },
    removeTranscriptionModel: (id: string) => {
      publish(withModel(id, { status: "available" }));
      return Promise.resolve(null);
    },
    setTranscriptionLanguage: (language: string) => {
      const transcripts = state.models.find(
        (model) => model.purpose === "transcripts",
      );
      const { dictationCommands, models: dictation } = dictationFor(
        language,
        transcripts ?? TRANSCRIPTS_MODEL,
      );
      const others = state.models.filter(
        (model) =>
          model.purpose !== "dictation" && model.purpose !== "dictationCleanup",
      );
      publish({
        ...state,
        dictationCommands,
        language,
        // Dictation's models after the Transcripts model, as the app lists
        // them.
        models: [...others.slice(0, 1), ...dictation, ...others.slice(1)],
      });
      return Promise.resolve(null);
    },
    setTranscriptionWords: (words: string[]) => {
      publish({ ...state, words });
      return Promise.resolve(null);
    },
  };
}
