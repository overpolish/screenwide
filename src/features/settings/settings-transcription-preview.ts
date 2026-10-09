// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type {
  TranscriptionModel,
  TranscriptionState,
} from "../transcription/types";

/** The Moments model, at its real download size. The recordings model joins
 * it once something transcribes recordings. */
const MOMENTS_MODEL: TranscriptionModel = {
  description: "Turns your moments' voice notes into text.",
  id: "base",
  name: "Moments",
  progress: null,
  purpose: "moments",
  sizeBytes: 147_951_465,
  status: "available",
};

/** The Studio sound model, at its real download size, both files together. */
const STUDIO_SOUND_MODEL: TranscriptionModel = {
  description: "Makes your microphone sound like a studio recording.",
  id: "sidon-v0.1",
  name: "Studio sound",
  progress: null,
  purpose: "studioSound",
  sizeBytes: 1_005_394_589,
  status: "available",
};

export type TranscriptionPreviewSeed = Pick<
  TranscriptionModel,
  "progress" | "status"
>;

/** How long a story's pretend download takes, in milliseconds. */
const DOWNLOAD_MS = 4_000;
const TICK_MS = 100;

/**
 * The transcription side of the Settings stories' API: the models kept in
 * memory, Moments as `seed` says and Studio sound not yet downloaded, with
 * downloads that fill over a few seconds. Nothing reaches the app or the
 * network.
 */
export function transcriptionPreviewApi(
  seed: TranscriptionPreviewSeed = { progress: null, status: "downloaded" },
) {
  let state: TranscriptionState = {
    language: "system",
    models: [{ ...MOMENTS_MODEL, ...seed }, STUDIO_SOUND_MODEL],
    systemLanguage: "en",
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
      publish({ ...state, language });
      return Promise.resolve(null);
    },
  };
}
