// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Circle, Lock } from "lucide-react";

import { IconButton } from "../../../components/base/button/icon-button";

type RecordingBarRecordActionProps = {
  canRecord: boolean;
  isLocked: boolean;
  isRecordBlockedByEditor: boolean;
  onFocusPendingEditor?: () => void;
  onRecord?: () => void;
  onRequiredPermissionsPress?: () => void;
};

export function RecordingBarRecordAction({
  canRecord,
  isLocked,
  isRecordBlockedByEditor,
  onFocusPendingEditor,
  onRecord,
  onRequiredPermissionsPress,
}: RecordingBarRecordActionProps) {
  return (
    <IconButton
      aria-label={
        isLocked
          ? "Open permissions"
          : isRecordBlockedByEditor
            ? "Show editor window"
            : "Start recording"
      }
      color="primary"
      iconSize="prominent"
      isDisabled={!canRecord && !isRecordBlockedByEditor && !isLocked}
      onPress={
        isLocked
          ? onRequiredPermissionsPress
          : isRecordBlockedByEditor
            ? onFocusPendingEditor
            : onRecord
      }
    >
      {isLocked ? <Lock /> : <Circle />}
    </IconButton>
  );
}
