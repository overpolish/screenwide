// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Trash2 } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { ProgressBar } from "../../../components/base/progress-bar/progress-bar";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";

import type { TranscriptionControls } from "./use-transcription";
import type { TranscriptionModel } from "../../transcription/types";

/**
 * What can be done with a model where it is shown: download it, follow and
 * cancel its download, or, where models are managed, remove it.
 */
export function TranscriptionModelActions({
  controls,
  isRemovable = false,
  model,
}: {
  controls: TranscriptionControls;
  model: TranscriptionModel;
  /** Offers removal once downloaded; only Settings › Transcription does. */
  isRemovable?: boolean;
}) {
  if (model.status === "available")
    return (
      <Button
        onPress={() => {
          controls.download(model.id);
        }}
      >
        Download
      </Button>
    );
  if (model.status === "downloading")
    return (
      <div className="gap-control flex items-center">
        <ProgressBar
          aria-label={`Downloading the ${model.name} model`}
          className="w-24"
          value={(model.progress ?? 0) * 100}
        />
        <Button
          onPress={() => {
            controls.cancel(model.id);
          }}
        >
          Cancel
        </Button>
      </div>
    );
  if (!isRemovable) return null;
  return (
    <ConfirmActionButton
      armedIcon={<Check />}
      armedLabel={`Confirm removing the ${model.name} model`}
      idleIcon={<Trash2 />}
      idleLabel={`Remove the ${model.name} model`}
      onConfirm={() => {
        controls.remove(model.id);
      }}
      variant="icon"
    />
  );
}
