// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { AnnotationLine } from "../annotation/annotation-island-parts";

import { cornerName } from "./scene-names";
import { CornerCameraGlyph, PairGlyph } from "./scene-option-glyphs";

import type { SceneFramingProps } from "./scene-framing";
import type { SceneCorner } from "../../../../bindings/SceneCorner";

// The choices of the layout in use, under its tiles: a pair's camera place
// and size, a picture in picture's corner and size, a custom layout's swap
// and saving it to use again, a saved layout's removal. A place or a size
// is a choice between looks, so it is drawn rather than named, on one line
// as the other islands' looks are; the names are for assistive technology.
// Left, right, top and bottom are the video's own, so they keep their order
// in every language.

const CORNERS: SceneCorner[] = [
  "top-left",
  "top-right",
  "bottom-left",
  "bottom-right",
];

const SIZES = ["small", "large"] as const;

const sizeName = (size: (typeof SIZES)[number]) =>
  size === "small"
    ? t("editor-island-scene-small")
    : t("editor-island-scene-large");

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
    const corner = variant.corner ?? "bottom-right";
    return (
      <AnnotationLine>
        <PositionGrid
          aria-label={t("editor-island-scene-corner")}
          columns={2}
          items={CORNERS.map((id) => ({ id, label: cornerName(id) }))}
          onChange={(chosen) => {
            onVariantChange({ corner: chosen });
          }}
          value={corner}
        />
        <PillGroup
          aria-label={t("editor-island-scene-size")}
          items={SIZES.map((size) => ({
            ariaLabel: sizeName(size),
            icon: <CornerCameraGlyph corner={corner} size={size} />,
            id: size,
          }))}
          onSelectionChange={(size) => {
            onVariantChange({ size: size === "small" ? "small" : "large" });
          }}
          selected={variant.size ?? "large"}
        />
      </AnnotationLine>
    );
  }
  if (layout.preset === "split-two-thirds" || layout.preset === "stacked") {
    const isStacked = layout.preset === "stacked";
    const isFirst = variant.swap === true;
    const isLarge = variant.cameraSize === "two-thirds";
    return (
      <AnnotationLine>
        <div dir="ltr">
          <PillGroup
            aria-label={t("editor-island-scene-camera-place")}
            items={[
              {
                ariaLabel: isStacked
                  ? t("editor-island-scene-top")
                  : t("editor-island-scene-left"),
                icon: (
                  <PairGlyph isFirst isLarge={isLarge} isStacked={isStacked} />
                ),
                id: "before",
              },
              {
                ariaLabel: isStacked
                  ? t("editor-island-scene-bottom")
                  : t("editor-island-scene-right"),
                icon: (
                  <PairGlyph
                    isFirst={false}
                    isLarge={isLarge}
                    isStacked={isStacked}
                  />
                ),
                id: "after",
              },
            ]}
            onSelectionChange={(place) => {
              onVariantChange({ swap: place === "before" });
            }}
            selected={isFirst ? "before" : "after"}
          />
        </div>
        <PillGroup
          aria-label={t("editor-island-scene-camera-size")}
          items={SIZES.map((size) => ({
            ariaLabel: sizeName(size),
            icon: (
              <PairGlyph
                isFirst={isFirst}
                isLarge={size === "large"}
                isStacked={isStacked}
              />
            ),
            id: size,
          }))}
          onSelectionChange={(size) => {
            onVariantChange({
              cameraSize: size === "large" ? "two-thirds" : "third",
            });
          }}
          selected={isLarge ? "large" : "small"}
        />
      </AnnotationLine>
    );
  }
  return null;
}
