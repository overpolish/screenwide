// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { ControlRow } from "../../components/shared/control-row/control-row";
import { t } from "../../i18n/i18n";

import { useSettingsUpdate } from "./use-settings-update";

import type { UpdateSnapshot } from "../updates/update-bridge";

/** The Software Update row of the General pane, as System Settings lists one:
 * the app with its version beneath, and a single button trailing. Figures are
 * tabular so a version does not shift as the numbers change. */
export function SoftwareUpdateSetting({
  currentVersion,
  error,
  onPress,
  status,
  updateVersion,
}: UpdateSnapshot & { onPress: () => void }) {
  const available =
    (status === "available" || status === "downloading") && updateVersion;
  return (
    <>
      <ControlRow
        className="tabular-nums"
        description={
          currentVersion
            ? t("settings-version", { version: currentVersion })
            : undefined
        }
        title={t("settings-app-name")}
      >
        {(controlProps) =>
          available ? (
            <Button
              aria-describedby={controlProps["aria-describedby"]}
              color="primary"
              onPress={onPress}
            >
              {t("settings-update-to", { version: updateVersion })}
            </Button>
          ) : (
            <Button
              aria-describedby={controlProps["aria-describedby"]}
              isDisabled={!error && status === "checking"}
              onPress={onPress}
            >
              {t("settings-check-updates")}
            </Button>
          )
        }
      </ControlRow>
      {error ? (
        <Alert color="error" role="alert">
          {error}
        </Alert>
      ) : null}
    </>
  );
}

export function LiveSoftwareUpdateSetting() {
  const { onPress, ...snapshot } = useSettingsUpdate();
  return <SoftwareUpdateSetting {...snapshot} onPress={onPress} />;
}
