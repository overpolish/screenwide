// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { EditorKind } from "../../types";
import { DimensionsField } from "../dimensions/dimensions-field";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

/**
 * The Crop tool's own controls: how much of the capture is kept.
 *
 * The numbers are source pixels - what the camera or the screen actually
 * recorded - rather than output pixels, because a crop chooses part of the
 * original and the finished picture's size is the Frame tool's business. A
 * size typed here moves the same rectangle the crop handles drag, about its
 * own centre, and Reset gives the whole capture back.
 */
export function CropPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { crop, isLocked } = snapshot;

  if (!crop) return null;

  return (
    <div className="flex flex-col gap-separate">
      <ControlRow title={t("editor-panels-crop")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <DimensionsField
              height={crop.height}
              isDisabled={isLocked}
              onChange={(cropSize) => {
                change({ cropSize });
              }}
              width={crop.width}
            />
          </div>
        )}
      </ControlRow>
      <div className="flex justify-end">
        <Button
          isDisabled={isLocked}
          onPress={() => {
            change({ resetCrop: true });
          }}
        >
          {t("editor-panels-reset")}
        </Button>
      </div>
    </div>
  );
}
