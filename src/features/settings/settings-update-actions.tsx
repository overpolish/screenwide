// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRight, RefreshCw } from "lucide-react";
import { TooltipTrigger } from "react-aria-components";

import { Button } from "../../components/base/button/button";
import { IconButton } from "../../components/base/button/icon-button";
import { Text } from "../../components/base/text/text";
import { Tooltip } from "../../components/base/tooltip/tooltip";

import { useSettingsUpdate } from "./use-settings-update";

import type { UpdateSnapshot } from "../updates/update-bridge";

export function SettingsUpdateActions({
  currentVersion,
  error,
  onPress,
  status,
  updateVersion,
}: UpdateSnapshot & { onPress: () => void }) {
  const available = status === "available" || status === "downloading";
  const version = currentVersion ? `v${currentVersion}` : "Version";
  if (available && updateVersion) {
    return (
      <Button
        aria-label={`Update Screenwide from ${currentVersion ?? "the current version"} to ${updateVersion}. Open software update.`}
        color="primary"
        onPress={onPress}
        size="compact"
      >
        <span className="font-mono">{version}</span>
        <ArrowRight aria-hidden />
        <span className="font-mono">v{updateVersion}</span>
      </Button>
    );
  }
  return (
    <div className="gap-control-inset flex items-center">
      <Text as="span" className="font-mono" variant="help">
        {version}
      </Text>
      {error ? (
        <TooltipTrigger>
          <IconButton
            aria-label="Retry update check"
            isDisabled={status === "checking"}
            onPress={onPress}
            size="compact"
          >
            <RefreshCw />
          </IconButton>
          <Tooltip>{error} Retry update check.</Tooltip>
        </TooltipTrigger>
      ) : null}
    </div>
  );
}

export function LiveSettingsUpdateActions() {
  const { onPress, ...snapshot } = useSettingsUpdate();
  return <SettingsUpdateActions {...snapshot} onPress={onPress} />;
}
