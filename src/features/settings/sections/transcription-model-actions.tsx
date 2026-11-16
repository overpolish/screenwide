// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCw, Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { ProgressBar } from "../../../primitives/progress-bar/progress-bar";

import type { TranscriptionControls } from "./use-transcription";
import type { TranscriptionModel } from "../../transcription/types";

/**
 * What can be done with a model where it is shown: download it, follow and
 * cancel its download, or, where models are managed, download it again or
 * remove it.
 */
export function TranscriptionModelActions({
  controls,
  isRemovable = false,
  model,
}: {
  controls: TranscriptionControls;
  model: TranscriptionModel;
  /** Offers downloading again and removal once downloaded; only Settings ›
   * Transcription does. */
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
      <div className="flex items-center gap-inline">
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
    <div className="flex items-center gap-separate">
      {/* Fetches a fresh copy beside the one in use, for a model that
          misbehaves; the copy replaces it only once it arrives whole. */}
      <IconButton
        aria-label={t("settings-transcription-download-again", {
          model: model.name,
        })}
        onPress={() => {
          controls.downloadAgain(model.id);
        }}
        variant="ghost"
      >
        <RotateCw aria-hidden className="size-icon" />
      </IconButton>
      <ConfirmActionButton
        confirmLabel={t("settings-transcription-confirm-remove", {
          model: model.name,
        })}
        icon={<Trash2 />}
        label={t("settings-transcription-remove", { model: model.name })}
        onConfirm={() => {
          controls.remove(model.id);
        }}
        variant="ghost"
      />
    </div>
  );
}
