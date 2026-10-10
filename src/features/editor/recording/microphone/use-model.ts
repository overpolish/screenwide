// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import {
  downloadTranscriptionModel,
  getTranscriptionState,
  listenToTranscription,
} from "../../../transcription/api";
import {
  TranscriptionModel,
  TranscriptionPurpose,
} from "../../../transcription/types";

/**
 * The model a switch works with, chosen by what it is for, as
 * Settings and every other window see it, kept current as it downloads;
 * `null` until the app has said. Downloading it is offered from the editor
 * as well as Settings.
 */
export function useModel(purpose: TranscriptionPurpose) {
  const [model, setModel] = useState<TranscriptionModel | null>(null);

  useEffect(() => {
    let current = true;
    let stop: (() => void) | undefined;
    const take = (models: TranscriptionModel[]) => {
      if (!current) return;
      setModel(models.find((each) => each.purpose === purpose) ?? null);
    };
    getTranscriptionState()
      .then((state) => {
        take(state.models);
      })
      .catch((cause: unknown) => {
        console.error(`Could not read the ${purpose} model`, cause);
      });
    listenToTranscription((state) => {
      take(state.models);
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error(`Could not follow the ${purpose} model`, cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [purpose]);

  return {
    download: () => {
      if (!model) return;
      downloadTranscriptionModel(model.id).catch((cause: unknown) => {
        console.error(`Could not download the ${purpose} model`, cause);
      });
    },
    /** The model as the panel shows it. */
    shown: model && {
      progress: model.progress ?? 0,
      sizeBytes: model.sizeBytes,
      status: model.status,
    },
  };
}
