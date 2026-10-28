// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";

import { cornerName } from "./scene-names";

import type { SceneFramingProps } from "./scene-framing";
import type { SceneCorner } from "../../../../bindings/SceneCorner";

// The choices of the layout in use, under its tiles: a pair's camera place
// and size, a picture in picture's corner and size, a custom layout's swap
// and saving it to use again, a saved layout's removal. They are words
// rather than drawings, since the chosen tile above redraws with each one
// and a drawing small enough for a row could not be read on a tall video.
// Left, right, top and bottom are the video's own, so they keep their order
// in every language.

const CORNERS: SceneCorner[] = [
  "top-left",
  "top-right",
  "bottom-left",
  "bottom-right",
];

const smallOrLarge = () => [
  { id: "small", label: t("editor-island-scene-small") },
  { id: "large", label: t("editor-island-scene-large") },
];

export function SceneLayoutOptions({
  layout,
  onRemoveSavedLayout,
  onSaveLayout,
  onSwapPanes,
  onVariantChange,
  variant,
}: SceneFramingProps) {
  // One-shot actions sit at the end of their own row, as wide as their
  // labels, as the custom layout's are.
  if (layout.kind === "saved") {
    return (
      <div className="flex justify-end">
        <ConfirmActionButton
          confirmLabel={t("editor-island-scene-confirm-remove-layout")}
          label={t("editor-island-scene-remove-layout")}
          onConfirm={() => {
            onRemoveSavedLayout(layout.id);
          }}
        />
      </div>
    );
  }
  if (layout.kind === "custom") {
    return (
      <div className="flex justify-end gap-separate">
        {layout.boxes.camera ? (
          <Button onPress={onSwapPanes}>{t("editor-island-scene-swap")}</Button>
        ) : null}
        <Button onPress={onSaveLayout}>
          {t("editor-island-scene-save-layout")}
        </Button>
      </div>
    );
  }
  if (layout.preset === "picture-in-picture") {
    return (
      <div className="flex flex-col gap-inline">
        <ControlRow title={t("editor-island-scene-corner")}>
          {(controlProps) => (
            <PositionGrid
              {...controlProps}
              columns={2}
              items={CORNERS.map((corner) => ({
                id: corner,
                label: cornerName(corner),
              }))}
              onChange={(corner) => {
                onVariantChange({ corner });
              }}
              value={variant.corner ?? "bottom-right"}
            />
          )}
        </ControlRow>
        <ControlRow title={t("editor-island-scene-size")}>
          {() => (
            <PillGroup
              aria-label={t("editor-island-scene-size")}
              items={smallOrLarge()}
              onSelectionChange={(size) => {
                onVariantChange({ size: size === "small" ? "small" : "large" });
              }}
              selected={variant.size ?? "large"}
            />
          )}
        </ControlRow>
      </div>
    );
  }
  if (layout.preset === "split-two-thirds" || layout.preset === "stacked") {
    const isStacked = layout.preset === "stacked";
    return (
      <div className="flex flex-col gap-inline">
        <ControlRow title={t("editor-island-scene-camera-place")}>
          {() => (
            <div dir="ltr">
              <PillGroup
                aria-label={t("editor-island-scene-camera-place")}
                items={[
                  {
                    id: "before",
                    label: isStacked
                      ? t("editor-island-scene-top")
                      : t("editor-island-scene-left"),
                  },
                  {
                    id: "after",
                    label: isStacked
                      ? t("editor-island-scene-bottom")
                      : t("editor-island-scene-right"),
                  },
                ]}
                onSelectionChange={(place) => {
                  onVariantChange({ swap: place === "before" });
                }}
                selected={variant.swap ? "before" : "after"}
              />
            </div>
          )}
        </ControlRow>
        <ControlRow title={t("editor-island-scene-camera-size")}>
          {() => (
            <PillGroup
              aria-label={t("editor-island-scene-camera-size")}
              items={smallOrLarge()}
              onSelectionChange={(size) => {
                onVariantChange({
                  cameraSize: size === "large" ? "two-thirds" : "third",
                });
              }}
              selected={variant.cameraSize === "two-thirds" ? "large" : "small"}
            />
          )}
        </ControlRow>
      </div>
    );
  }
  return null;
}
