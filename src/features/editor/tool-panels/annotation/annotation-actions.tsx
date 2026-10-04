// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowLeftRight, Eraser, FlipHorizontal2 } from "lucide-react";

import { Button } from "../../../../components/base/button/button";

import type { AnnotationKind } from "../../../../components/shared/annotation-style/types";
import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * The one-shot actions at the foot of the annotation panel, in a row of their
 * own. An arrow is turned round and a sticker mirrored, both through the same
 * commit path the drag on the picture uses; neither is offered before
 * anything is drawn. Clearing belongs to the pen rather than to one stroke,
 * so it is offered before anything is drawn too; one undo brings every
 * stroke back.
 */
export function AnnotationActions({
  canClearDrawings,
  change,
  isDraft,
  isLocked,
  kind,
  reversible,
}: {
  canClearDrawings: boolean;
  change: (values: ToolPanelPatch) => void;
  isDraft: boolean;
  isLocked: boolean;
  kind: AnnotationKind;
  reversible: boolean;
}) {
  if (reversible && !isDraft) {
    const sticker = kind === "sticker";
    return (
      <div className="flex justify-end">
        <Button
          aria-label={sticker ? "Mirror the sticker" : "Reverse the arrow"}
          isDisabled={isLocked}
          onPress={() => {
            change({ reverseAnnotation: true });
          }}
        >
          {sticker ? (
            <FlipHorizontal2 aria-hidden="true" />
          ) : (
            <ArrowLeftRight aria-hidden="true" />
          )}
          {sticker ? "Flip" : "Reverse"}
        </Button>
      </div>
    );
  }
  if (kind !== "draw") return null;
  return (
    <div className="flex justify-end">
      <Button
        aria-label="Clear all drawings"
        isDisabled={isLocked || !canClearDrawings}
        onPress={() => {
          change({ clearDrawings: true });
        }}
      >
        <Eraser aria-hidden="true" />
        Clear all
      </Button>
    </div>
  );
}
