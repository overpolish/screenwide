// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Badge } from "../../../../components/base/badge/badge";
import { Button } from "../../../../components/base/button/button";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { formatBytes } from "../../duration";
import { ToolPanelMicrophone, ToolPanelModel } from "../tool-panel-store";

import { ToolProgress } from "./tool-progress";

type SwitchProps = {
  isLocked: boolean;
  onChange: (enabled: boolean) => void;
  progress: number;
  state: ToolPanelMicrophone["noise"];
  title: string;
  working: string;
};

/** A switch whose first turn can take a while, as the microphone's do, with
 * a bar under it while it works. */
export function WorkingSwitchRow({
  isLocked,
  onChange,
  progress,
  state,
  title,
  working,
}: SwitchProps) {
  return (
    <div>
      <ControlRow title={title}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={state !== "off"}
            onChange={onChange}
          />
        )}
      </ControlRow>
      <ToolProgress
        isWorking={state === "cleaning"}
        label={working}
        value={progress}
      />
    </div>
  );
}

/** A tool's row before its model is on this computer: the model's size and
 * a download, with the bar under it while the model comes down. */
export function ModelDownloadRow({
  description,
  isLocked,
  model,
  onDownload,
  title,
}: {
  description: string;
  isLocked: boolean;
  model: ToolPanelModel;
  onDownload: () => void;
  title: string;
}) {
  const isDownloading = model.status === "downloading";
  return (
    <div>
      <ControlRow
        description={description}
        title={title}
        titleAccessory={<Badge>{formatBytes(model.sizeBytes)}</Badge>}
      >
        {(controlProps) => (
          <Button
            {...controlProps}
            isDisabled={isLocked || isDownloading}
            onPress={onDownload}
          >
            {t("editor-panels-model-download")}
          </Button>
        )}
      </ControlRow>
      <ToolProgress
        isWorking={isDownloading}
        label={t("editor-panels-model-downloading")}
        value={model.progress}
      />
    </div>
  );
}

/** A switch that needs a downloaded model, as Studio sound and Remove
 * background do: the switch once the model is here, and before, a download
 * in its place. Nothing until the app has said. */
export function ModelSwitchRow({
  description,
  model,
  onDownload,
  ...switchProps
}: SwitchProps & {
  description: string;
  model: ToolPanelModel | null;
  onDownload: () => void;
}) {
  if (!model) return null;
  if (model.status === "downloaded")
    return <WorkingSwitchRow {...switchProps} />;
  return (
    <ModelDownloadRow
      description={description}
      isLocked={switchProps.isLocked}
      model={model}
      onDownload={onDownload}
      title={switchProps.title}
    />
  );
}
