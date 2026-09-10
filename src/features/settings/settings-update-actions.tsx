// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRight } from "lucide-react";

import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { Setting } from "../../components/shared/setting/setting";

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
      <Setting
        className="tabular-nums"
        description={currentVersion ? `Version ${currentVersion}` : undefined}
        title="Screenwide"
      >
        {(controlProps) =>
          available ? (
            <Button
              aria-describedby={controlProps["aria-describedby"]}
              color="primary"
              onPress={onPress}
            >
              Update to v{updateVersion}
              <ArrowRight aria-hidden />
            </Button>
          ) : (
            <Button
              aria-describedby={controlProps["aria-describedby"]}
              isDisabled={!error && status === "checking"}
              onPress={onPress}
            >
              Check for Updates
            </Button>
          )
        }
      </Setting>
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
