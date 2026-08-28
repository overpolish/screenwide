// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Circle, Lock } from "lucide-react";

import { IconButton } from "../../../components/base/button/icon-button";

type RecordingBarRecordActionProps = {
  canRecord: boolean;
  isLocked: boolean;
  isRecordBlockedByExport: boolean;
  onFocusPendingExport?: () => void;
  onRecord?: () => void;
  onRequiredPermissionsPress?: () => void;
};

export function RecordingBarRecordAction({
  canRecord,
  isLocked,
  isRecordBlockedByExport,
  onFocusPendingExport,
  onRecord,
  onRequiredPermissionsPress,
}: RecordingBarRecordActionProps) {
  return (
    <IconButton
      aria-label={
        isLocked
          ? "Open permissions"
          : isRecordBlockedByExport
            ? "Show export window"
            : "Start recording"
      }
      color="primary"
      iconSize="prominent"
      isDisabled={!canRecord && !isRecordBlockedByExport && !isLocked}
      onPress={
        isLocked
          ? onRequiredPermissionsPress
          : isRecordBlockedByExport
            ? onFocusPendingExport
            : onRecord
      }
    >
      {isLocked ? <Lock /> : <Circle />}
    </IconButton>
  );
}
