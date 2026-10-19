// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { t } from "../../../../i18n/i18n";
import { ToolPanelPatch } from "../tool-panel-patch";

/**
 * The one-shot actions that end the Scene panel for the scene under the
 * playhead: a custom scene kept as a template, the face brought back to the
 * middle of a camera that follows it, the scene's framing reset, or the
 * scene taken away.
 */
export function SceneActions({
  canRecenter,
  canSaveTemplate,
  change,
  isDisabled,
}: {
  /** Whether the camera follows the face and is the pane selected. */
  canRecenter: boolean;
  /** Whether the scene is custom, which a template keeps. */
  canSaveTemplate: boolean;
  change: (values: ToolPanelPatch) => void;
  isDisabled: boolean;
}) {
  return (
    <div className="flex justify-end gap-control">
      {canSaveTemplate ? (
        <Button
          isDisabled={isDisabled}
          onPress={() => {
            change({ saveSceneTemplate: true });
          }}
        >
          {t("editor-panels-save-template")}
        </Button>
      ) : null}
      {/* While the camera follows the face, its framing's middle is the
          face, so centring the framing centres the face. */}
      {canRecenter ? (
        <Button
          isDisabled={isDisabled}
          onPress={() => {
            change({ sceneFraming: { focusX: 0.5, focusY: 0.5 } });
          }}
        >
          {t("editor-panels-recenter")}
        </Button>
      ) : null}
      <Button
        isDisabled={isDisabled}
        onPress={() => {
          change({ resetSceneFraming: true });
        }}
      >
        {t("editor-panels-reset")}
      </Button>
      <Button
        isDisabled={isDisabled}
        onPress={() => {
          change({ deleteScene: true });
        }}
      >
        {t("editor-panels-delete")}
      </Button>
    </div>
  );
}
