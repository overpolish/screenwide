// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check, X } from "lucide-react";

import { ConfirmActionButton } from "../../../components/shared/confirm-action-button/confirm-action-button";
import { ProgressPanel } from "../../../components/shared/progress-panel/progress-panel";
import { formatEta } from "../duration";
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
  if (isAudioOnly) return "Saving audio";
  if (!isRecording) return "Saving screenshot";
  if (phase === "camera") return "Saving camera";
  if (phase === "finalizing") return "Finalizing recording";
  return "Saving recording";
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
            armedLabel="Confirm cancel"
            idleIcon={<X />}
            idleLabel={isCancelingSave ? "Canceling" : "Cancel"}
            isDisabled={isCancelingSave}
            onConfirm={onCancel}
            variant="text"
          />
        ) : undefined
      }
      label={saveLabel({ isAudioOnly, isRecording, phase })}
      progress={progress}
      progressLabel="Save progress"
      secondary={etaSeconds === null ? undefined : formatEta(etaSeconds)}
    />
  );
}
