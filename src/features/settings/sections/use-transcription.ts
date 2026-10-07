// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { useSettingsApi } from "../settings-api-context";

import type {
  TranscriptionModel,
  TranscriptionPurpose,
  TranscriptionState,
} from "../../transcription/types";

/**
 * The transcription models and language as Rust has them, kept current by
 * its change event (download progress arrives that way too), and the actions
 * on them. An action's result comes back through the same event rather than
 * its reply, so every window shows one state.
 */
export function useTranscription(onError: (error: string | null) => void) {
  const api = useSettingsApi();
  const { getTranscriptionState, listenToTranscription } = api;
  const [state, setState] = useState<TranscriptionState | null>(null);

  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    getTranscriptionState()
      .then(setState)
      .catch((reason: unknown) => {
        onError(String(reason));
      });
    void listenToTranscription(setState).then((unlisten) => {
      if (stopped) unlisten();
      else stop = unlisten;
    });
    return () => {
      stopped = true;
      stop?.();
    };
  }, [getTranscriptionState, listenToTranscription, onError]);

  const run = useCallback(
    (action: Promise<unknown>) => {
      onError(null);
      action.catch((reason: unknown) => {
        onError(String(reason));
      });
    },
    [onError],
  );

  return {
    cancel: (model: string) => {
      run(api.cancelTranscriptionDownload(model));
    },
    download: (model: string) => {
      run(api.downloadTranscriptionModel(model));
    },
    remove: (model: string) => {
      run(api.removeTranscriptionModel(model));
    },
    setLanguage: (language: string) => {
      run(api.setTranscriptionLanguage(language));
    },
    state,
  };
}

export type TranscriptionControls = ReturnType<typeof useTranscription>;

/** The model `purpose` uses. */
export const modelFor = (
  state: TranscriptionState | null,
  purpose: TranscriptionPurpose,
): TranscriptionModel | undefined =>
  state?.models.find((model) => model.purpose === purpose);

/** Whether `purpose` can be transcribed now: with its own model, or with any
 * other that is downloaded, more slowly perhaps but never not at all. */
export const canTranscribe = (
  state: TranscriptionState | null,
  purpose: TranscriptionPurpose,
) =>
  modelFor(state, purpose)?.status === "downloaded" ||
  (state?.models.some((model) => model.status === "downloaded") ?? false);
