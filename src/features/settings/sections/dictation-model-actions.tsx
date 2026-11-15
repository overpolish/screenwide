// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { Button } from "../../../primitives/button/button";
import { usePermissionStore } from "../../permissions/store";
import { useSettingsApi } from "../settings-api-context";

import { useDictationMissingAccess } from "./dictation-access";
import { TranscriptionModelActions } from "./transcription-model-actions";

import type { TranscriptionControls } from "./use-transcription";
import type { TranscriptionModel } from "../../transcription/types";

/**
 * The dictation model's actions: its download, offered once Screenwide has
 * the access dictation needs, and asking for that access until then, or
 * again once it is taken away. The system's answer arrives as the
 * permissions change, so Settings sees it without a restart.
 */
export function DictationModelActions({
  controls,
  model,
}: {
  controls: TranscriptionControls;
  model: TranscriptionModel;
}) {
  const { grantDictationAccess } = useSettingsApi();
  const permissions = usePermissionStore((state) => state.permissions);
  const missing = useDictationMissingAccess();
  if (model.status === "downloading" || !missing)
    return (
      <TranscriptionModelActions
        controls={controls}
        isRemovable
        model={model}
      />
    );
  return (
    <Button
      onPress={() => {
        void grantDictationAccess(missing, permissions[missing].canRequest);
      }}
    >
      {missing === "microphone"
        ? t("settings-transcription-dictation-allow-microphone")
        : t("settings-transcription-dictation-allow-accessibility")}
    </Button>
  );
}
