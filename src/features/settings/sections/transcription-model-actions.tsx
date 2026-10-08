// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, Trash2 } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { ProgressBar } from "../../../components/base/progress-bar/progress-bar";
import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";

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
        {t("settings-transcription-download")}
      </Button>
    );
  if (model.status === "downloading")
    return (
      <div className="gap-control flex items-center">
        <ProgressBar
          aria-label={t("settings-transcription-downloading", {
            model: model.name,
          })}
          className="w-24"
          value={(model.progress ?? 0) * 100}
        />
        <Button
          onPress={() => {
            controls.cancel(model.id);
          }}
        >
          {t("settings-transcription-cancel")}
        </Button>
      </div>
    );
  if (!isRemovable) return null;
  return (
    <ConfirmActionButton
      armedIcon={<Check />}
      armedLabel={t("settings-transcription-confirm-remove", {
        model: model.name,
      })}
      idleIcon={<Trash2 />}
      idleLabel={t("settings-transcription-remove", { model: model.name })}
      onConfirm={() => {
        controls.remove(model.id);
      }}
      variant="icon"
    />
  );
}
