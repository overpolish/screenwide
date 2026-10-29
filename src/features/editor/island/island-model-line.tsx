// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Badge } from "../../../primitives/badge/badge";
import { Button } from "../../../primitives/button/button";
import { ProgressBar } from "../../../primitives/progress-bar/progress-bar";
import { formatBytes } from "../duration";

import type { ToolPanelModel } from "../tool-panels/tool-panel-selection";

/** A tool whose model is not on this computer yet, fetched from right here
 * rather than from Settings, on every island alike: the tool's name and the
 * model's size, and the download, which gives way to the bar while the
 * model comes. The bar keeps to the island's text column, where a button's
 * own padding would have kept its label. */
export function IslandModelLine({
  model,
  onDownload,
  title,
}: {
  model: ToolPanelModel;
  onDownload: () => void;
  /** The tool the model is for, such as Remove background. */
  title: string;
}) {
  return (
    <ControlRow
      title={title}
      titleAccessory={<Badge>{formatBytes(model.sizeBytes)}</Badge>}
    >
      {(controlProps) =>
        model.status === "downloading" ? (
          <ProgressBar
            aria-label={t("editor-island-model-downloading")}
            className="me-inset-sm w-24"
            value={model.progress * 100}
          />
        ) : (
          <Button {...controlProps} onPress={onDownload}>
            {t("editor-island-model-download")}
          </Button>
        )
      }
    </ControlRow>
  );
}
