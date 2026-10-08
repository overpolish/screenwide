// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, X } from "lucide-react";

import { ConfirmActionButton } from "../../../../components/shared/confirm-action-button/confirm-action-button";
import { ProgressPanel } from "../../../../components/shared/progress-panel/progress-panel";
import { t } from "../../../../i18n/i18n";
import { formatEta } from "../../duration";
import { ExportPhase } from "../use-export-progress";

export type ExportProgressProps = {
  /** Whether the backend can stop this save once it has started. */
  cancellable: boolean;
  etaSeconds: number | null;
  isAudioOnly: boolean;
  isCancelingSave: boolean;
  isRecording: boolean;
  phase: ExportPhase;
  progress: number | null;
  onCancel?: () => void;
};

/** What the save is currently writing, said in the user's terms. */
const saveLabel = ({
  isAudioOnly,
  isRecording,
  phase,
}: Pick<ExportProgressProps, "isAudioOnly" | "isRecording" | "phase">) => {
  if (isAudioOnly) return t("editor-export-saving-audio");
  if (!isRecording) return t("editor-export-saving-screenshot");
  if (phase === "camera") return t("editor-export-saving-camera");
  if (phase === "finalizing") return t("editor-export-finalizing");
  return t("editor-export-saving-recording");
};

/**
 * The running save, shown where the export form was.
 *
 * The window owns none of it: the editor is doing the work and publishes how
 * far it has come, and Cancel is a request back to that editor.
 */
export function ExportProgress({
  cancellable,
  etaSeconds,
  isAudioOnly,
  isCancelingSave,
  isRecording,
  onCancel,
  phase,
  progress,
}: ExportProgressProps) {
  return (
    <ProgressPanel
      action={
        cancellable ? (
          <ConfirmActionButton
            armedIcon={<Check />}
            armedLabel={t("editor-export-confirm-cancel")}
            idleIcon={<X />}
            idleLabel={
              isCancelingSave
                ? t("editor-export-canceling")
                : t("editor-export-cancel")
            }
            isDisabled={isCancelingSave}
            onConfirm={onCancel}
            variant="text"
          />
        ) : undefined
      }
      label={saveLabel({ isAudioOnly, isRecording, phase })}
      progress={progress}
      progressLabel={t("editor-export-progress")}
      secondary={etaSeconds === null ? undefined : formatEta(etaSeconds)}
    />
  );
}
