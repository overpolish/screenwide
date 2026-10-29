// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { AnnotationLine } from "../annotation/annotation-island-parts";

import { BackgroundGlyph } from "./camera-glyphs";

import type { ToolPanelCameraEffects } from "../../tool-panels/tool-panel-selection";

type CameraBackground = ToolPanelCameraEffects["background"];

const BACKGROUNDS = ["transparent", "blurred", "image"] as const;

const backgroundName = (background: (typeof BACKGROUNDS)[number]) =>
  background === "transparent"
    ? t("editor-panels-camera-background-none")
    : background === "blurred"
      ? t("editor-panels-camera-background-blur")
      : t("editor-panels-camera-background-picture");

/** What is behind the person once the room is taken away, a choice between
 * looks drawn as each looks: nothing, the room blurred, or a picture. With
 * a picture behind, choosing another is a one-shot action at the line's
 * end, acting on the line it sits in. Choosing a picture before one was
 * ever chosen asks for one rather than showing an empty one. */
export function CameraBackgroundLine({
  background,
  hasImage,
  onChange,
  onChoosePicture,
}: {
  background: CameraBackground;
  hasImage: boolean;
  onChange: (background: CameraBackground) => void;
  onChoosePicture: () => void;
}) {
  return (
    <AnnotationLine>
      <PillGroup
        aria-label={t("editor-panels-camera-background")}
        items={BACKGROUNDS.map((id) => ({
          ariaLabel: backgroundName(id),
          icon: <BackgroundGlyph background={id} />,
          id,
        }))}
        key="backgrounds"
        onSelectionChange={(next) => {
          if (next === "image" && !hasImage) {
            onChoosePicture();
            return;
          }
          onChange(next as CameraBackground);
        }}
        selected={background}
      />
      {background === "image" ? (
        <Button
          aria-label={t("editor-panels-replace-camera-picture")}
          key="replace"
          onPress={onChoosePicture}
        >
          {t("editor-panels-replace")}
        </Button>
      ) : null}
    </AnnotationLine>
  );
}
