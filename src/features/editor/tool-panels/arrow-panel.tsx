// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowLeftRight } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { Switch } from "../../../components/base/switch/switch";
import { Text } from "../../../components/base/text/text";
import { AnnotationColorGrid } from "../../../components/shared/annotation-style/annotation-color-grid";
import { AnnotationHeadGroup } from "../../../components/shared/annotation-style/annotation-head-group";
import { AnnotationWidthSlider } from "../../../components/shared/annotation-style/annotation-width-slider";
import { EditorKind } from "../types";

import { useAnnotationColorMenu } from "./use-annotation-color-menu";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The chosen mark's own controls: how heavy it is drawn, whether it draws
 * itself in over its clip, which ends carry a head, and what colour it is.
 * Animating takes a clip to animate over, so that row is the recording
 * editor's alone.
 *
 * This panel belongs to the mark rather than to a tool. It comes up the
 * moment an arrow is chosen, in either tool that can choose one, and goes
 * away when the choice does. Every change is committed through the same path
 * the drag on the picture uses, so it lands in the edit history as one edit,
 * and becomes the dress the next arrow is drawn in.
 *
 * The head and the colours are the shared controls the live overlay's toolbar
 * carries, so a mark is dressed the same way wherever it is drawn. A colour
 * the palette does not hold is kept the moment the system panel closes on it,
 * and forgotten from the same right-click menu a saved background is.
 */
export function ArrowPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { annotation, annotationColors, isSaving } = snapshot;
  const showColorMenu = useAnnotationColorMenu((color) => {
    change({ removeAnnotationColor: color });
  });

  if (!annotation) return <Text variant="body">Nothing selected</Text>;

  const { color, head, width } = annotation.style;

  return (
    <div className="flex flex-col gap-section">
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Width</span>
        <AnnotationWidthSlider
          isDisabled={isSaving}
          onChange={(next) => {
            change({ annotationStyle: { width: next } });
          }}
          value={width}
        />
      </div>

      {/* Drawing in and out happens over a clip, and only a recording has
          one: a screenshot is one instant, so there is no time for a mark to
          arrive over and the row is not offered there at all. */}
      {workspace === "recording" ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Animate</span>
          <Switch
            aria-label="Animate"
            isDisabled={isSaving}
            isSelected={annotation.animated}
            onChange={(next) => {
              change({ annotationAnimated: next });
            }}
          />
        </div>
      ) : null}

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Head</span>
        <AnnotationHeadGroup
          isDisabled={isSaving}
          onChange={(next) => {
            change({ annotationStyle: { head: next } });
          }}
          value={head}
        />
      </div>

      {/* The head rides the end point, so turning the mark round points it
          the other way without redrawing it: the same commit path the drag
          on the picture uses. */}
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Direction</span>
        <Button
          aria-label="Reverse the arrow"
          isDisabled={isSaving}
          onPress={() => {
            change({ reverseAnnotation: true });
          }}
        >
          <ArrowLeftRight aria-hidden="true" />
          Reverse
        </Button>
      </div>

      {/* The swatches say what they are, so the row carries no heading; it
          keeps the section gap its two labelled neighbours sit on. */}
      <AnnotationColorGrid
        isDisabled={isSaving}
        onChange={(next) => {
          change({ annotationStyle: { color: next } });
        }}
        onContextMenuColor={(kept, anchor) => {
          void showColorMenu(kept, anchor);
        }}
        onSettled={(settled) => {
          change({
            annotationStyle: { color: settled },
            saveAnnotationColor: settled,
          });
        }}
        savedColors={annotationColors}
        value={color}
      />
    </div>
  );
}
