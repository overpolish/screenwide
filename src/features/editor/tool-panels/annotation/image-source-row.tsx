// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { browseAnnotationImage } from "../../images/image-api";

import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * The picture file an image annotation shows. With one placed, Replace gives
 * it another picture and keeps its place, size and turn, and Flip mirrors the
 * picture across its upright axis; with none, Choose places the picture in
 * the middle of the layer in hand, and the next press of the tool shows it
 * too.
 */
export function ImageSourceRow({
  change,
  isLocked,
  isPlaced,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  isPlaced: boolean;
}) {
  return (
    <ControlRow
      description={t("editor-panels-image-description")}
      title={t("annotation-tool-image")}
    >
      {(controlProps) => (
        <div {...controlProps} className="flex gap-control" role="group">
          <Button
            aria-label={
              isPlaced
                ? t("editor-panels-replace-image")
                : t("editor-panels-choose-image")
            }
            isDisabled={isLocked}
            onPress={() => {
              browseAnnotationImage()
                .then((art) => {
                  if (art) change({ annotationImage: art });
                })
                .catch((cause: unknown) => {
                  console.error("Could not use that picture", cause);
                });
            }}
          >
            {isPlaced ? t("editor-panels-replace") : t("editor-panels-choose")}
          </Button>
          {isPlaced ? (
            <Button
              aria-label={t("editor-panels-mirror-image")}
              isDisabled={isLocked}
              onPress={() => {
                change({ reverseAnnotation: true });
              }}
            >
              {t("editor-panels-flip")}
            </Button>
          ) : null}
        </div>
      )}
    </ControlRow>
  );
}
