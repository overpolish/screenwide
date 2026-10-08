// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationShuffleSwitch } from "../../../../components/shared/annotation-style/annotation-shuffle-switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import type { EditorKind } from "../../types";
import type { ToolPanelPatch } from "../tool-panel-patch";
import type { ToolPanelAnnotation } from "../tool-panel-selection";

/**
 * Whether a recording gives a placed image a slight, slow turn and drift,
 * so a picture that points at nothing still feels alive. While it sways,
 * the dice gives it another sway of its own. Other annotations mark
 * something on the picture and stay put; a screenshot is one instant with
 * nothing to sway over, and a fresh image always arrives still.
 */
export function ImageSwayRow({
  annotation,
  change,
  isLocked,
  workspace,
}: {
  annotation: ToolPanelAnnotation;
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  workspace: EditorKind;
}) {
  if (
    annotation.kind !== "image" ||
    annotation.isDraft === true ||
    workspace !== "recording"
  )
    return null;
  return (
    <ControlRow title={t("editor-panels-sway")}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <AnnotationShuffleSwitch
            isDisabled={isLocked}
            isSelected={annotation.sway === true}
            label={t("editor-panels-sway")}
            onChange={(next) => {
              change({ annotationImageSway: next });
            }}
            onRandomise={() => {
              change({ shuffleAnnotation: true });
            }}
          />
        </div>
      )}
    </ControlRow>
  );
}
