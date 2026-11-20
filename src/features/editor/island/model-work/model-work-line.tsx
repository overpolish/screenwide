// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { X } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { ProgressBar } from "../../../../primitives/progress-bar/progress-bar";
import { Text } from "../../../../primitives/text/text";
import { formatEta } from "../../duration";

import type { ModelJob } from "../../../../bindings/ModelJob";
import type { ModelWork } from "../../../../bindings/ModelWork";

const WORKING: Record<ModelWork, () => string> = {
  autoVolume: () => t("editor-island-auto-volume-measuring"),
  cameraObjects: () => t("editor-island-camera-objects-working"),
  enhance: () => t("editor-island-enhance-working"),
  followFace: () => t("editor-island-finding-face"),
  reduceNoise: () => t("editor-island-noise-cleaning"),
  removeBackground: () => t("editor-island-remove-background-working"),
  restoreVoice: () => t("editor-island-restore-voice-working"),
  smartDucking: () => t("editor-island-smart-ducking-working"),
  transcript: () => t("editor-island-transcribing"),
  vocalCleanup: () => t("editor-island-voice-cleaning"),
};

/** One tool at work: what it is doing and how long it has left on the text
 * column, its bar, and Cancel at the line's end. */
export function ModelWorkLine({
  job,
  onCancel,
}: {
  job: ModelJob;
  onCancel: () => void;
}) {
  const label = WORKING[job.work]();
  return (
    <div
      aria-label={label}
      className="flex items-center gap-separate"
      role="group"
    >
      <div className="flex min-w-0 flex-1 flex-col ps-inset-sm">
        <Text as="p" className="text-fg-secondary" variant="caption">
          {label}
        </Text>
        <Text as="p" className="text-fg-tertiary" variant="caption">
          {job.secondsLeft === null
            ? t("editor-island-model-work-estimating")
            : formatEta(job.secondsLeft)}
        </Text>
      </div>
      <ProgressBar
        aria-label={label}
        className="w-24"
        value={job.fraction * 100}
      />
      <IconButton
        aria-label={t("editor-island-model-work-cancel")}
        className="text-fg-secondary enabled:hover:text-fg"
        onPress={onCancel}
      >
        <X aria-hidden className="size-icon" />
      </IconButton>
    </div>
  );
}
