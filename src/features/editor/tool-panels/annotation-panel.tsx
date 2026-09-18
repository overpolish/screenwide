// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowLeftRight } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { Switch } from "../../../components/base/switch/switch";
import { Text } from "../../../components/base/text/text";
import { AnnotationAngleDial } from "../../../components/shared/annotation-style/annotation-angle-dial";
import { AnnotationColorGrid } from "../../../components/shared/annotation-style/annotation-color-grid";
import { AnnotationHeadGroup } from "../../../components/shared/annotation-style/annotation-head-group";
import { AnnotationWidthSlider } from "../../../components/shared/annotation-style/annotation-width-slider";
import { annotationSizes } from "../../../components/shared/annotation-style/widths";
import { EditorKind } from "../types";

import { useAnnotationColorMenu } from "./use-annotation-color-menu";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The chosen annotation's own controls: how big it is drawn, whether it arrives
 * over its clip, and what colour it is - plus the controls its shape has. An
 * arrow carries heads and can be turned round; a counter is a disc with a
 * number in it, which leaves it nothing to reverse and no head to choose.
 * Animating takes a clip to animate over, so that row is the recording editor's
 * alone.
 *
 * This panel belongs to the annotation rather than to a tool. It comes up the
 * moment one is chosen, in any tool that can choose one, and goes away when the
 * choice does. Every change is committed through the same path the drag on the
 * picture uses, so it lands in the edit history as one edit, and becomes the
 * dress the next annotation is drawn in.
 *
 * The head and the colours are the shared controls the live overlay's toolbar
 * carries, so an annotation is dressed the same way wherever it is drawn. A
 * colour the palette does not hold is kept the moment the system panel closes
 * on it, and forgotten from the same right-click menu a saved background is.
 */
export function AnnotationPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { annotation, annotationColors, isSaving } = snapshot;
  const showColorMenu = useAnnotationColorMenu((color) => {
    change({ removeAnnotationColor: color });
  });

  if (!annotation) return <Text variant="body">Nothing selected</Text>;

  const { color, head, width } = annotation.style;
  const isCounter = annotation.kind === "counter";

  return (
    <div className="flex flex-col gap-section">
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">
          {isCounter ? "Size" : "Width"}
        </span>
        <AnnotationWidthSlider
          isDisabled={isSaving}
          label={isCounter ? "Size" : "Width"}
          onChange={(next) => {
            change({ annotationStyle: { width: next } });
          }}
          presets={annotationSizes(annotation.kind)}
          value={width}
        />
      </div>

      {/* A counter is aimed by its tail, on the picture or here. The dial's
          notch stands where the tail does, and a held Shift snaps it to the
          same eighth of a turn a drag on the picture snaps to. */}
      {isCounter ? (
        <div className="flex items-center justify-between gap-section">
          <span className="text-body text-content-fg">Angle</span>
          <AnnotationAngleDial
            isDisabled={isSaving}
            onChange={(next) => {
              change({ annotationAngle: next });
            }}
            value={annotation.angle ?? 0}
          />
        </div>
      ) : null}

      {/* Drawing in and out happens over a clip, and only a recording has
          one: a screenshot is one instant, so there is no time for an annotation to
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

      {isCounter ? null : (
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
      )}

      {/* The head rides the end point, so turning the annotation round points it
          the other way without redrawing it: the same commit path the drag
          on the picture uses. A counter is aimed by its tail instead, on the
          picture itself. */}
      {isCounter ? null : (
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
      )}

      {/* The swatches say what they are, so the row carries no heading; it
          keeps the section gap its labelled neighbours sit on. */}
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
