// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import {
  downloadTranscriptionModel,
  getTranscriptionState,
  listenToTranscription,
} from "../../../transcription/api";
import { TranscriptionModel } from "../../../transcription/types";

/**
 * The model Studio sound rebuilds the microphone with, as Settings and every
 * other window see it, kept current as it downloads; `null` until the app
 * has said. Downloading it is offered from the editor as well as Settings.
 */
export function useStudioSoundModel() {
  const [model, setModel] = useState<TranscriptionModel | null>(null);

  useEffect(() => {
    let current = true;
    let stop: (() => void) | undefined;
    const take = (models: TranscriptionModel[]) => {
      if (!current) return;
      setModel(models.find((each) => each.purpose === "studioSound") ?? null);
    };
    getTranscriptionState()
      .then((state) => {
        take(state.models);
      })
      .catch((cause: unknown) => {
        console.error("Could not read the Studio sound model", cause);
      });
    listenToTranscription((state) => {
      take(state.models);
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the Studio sound model", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, []);

  return {
    download: () => {
      if (!model) return;
      downloadTranscriptionModel(model.id).catch((cause: unknown) => {
        console.error("Could not download the Studio sound model", cause);
      });
    },
    model,
  };
}
