// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
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
    <ControlRow description="Paste or drag images in" title="Image">
      {(controlProps) => (
        <div {...controlProps} className="flex gap-control" role="group">
          <Button
            aria-label={isPlaced ? "Replace the image" : "Choose an image"}
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
            {isPlaced ? "Replace" : "Choose"}
          </Button>
          {isPlaced ? (
            <Button
              aria-label="Mirror the image"
              isDisabled={isLocked}
              onPress={() => {
                change({ reverseAnnotation: true });
              }}
            >
              Flip
            </Button>
          ) : null}
        </div>
      )}
    </ControlRow>
  );
}
