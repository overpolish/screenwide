// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../components/base/button/button";
import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { Text } from "../../../components/base/text/text";
import { Dimensions } from "../../../components/shared/dimensions/dimensions";
import { SliderNumberField } from "../../../components/shared/slider-number-field/slider-number-field";
import { EditorKind } from "../types";

import { ShortcutSelectionRows } from "./shortcut-selection-rows";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The Select tool's own controls: what is selected, how big it is in the
 * finished picture, and where it sits in it.
 *
 * Every number is in output pixels, the same units the size readout under the
 * preview uses, so a value typed here and a value dragged out there mean the
 * same thing. Dragging in the preview republishes the placement, and the panel
 * follows it live. The shadow is the layer's own, cast onto whatever the
 * canvas is wearing, so it is set here rather than with the canvas.
 *
 * Padding is the same idea one step out: the frame the layer is placed by can
 * be grown past the picture in it, filled with the colour behind that picture,
 * and the content inside it put back in the middle.
 *
 * A keyboard shortcut is drawn rather than placed: it has no source pixels to
 * be sized against, no corners and no pad, so it is shown as the share of the
 * canvas it takes and the point it is centred on.
 */
export function SelectionPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { isSaving, selection } = snapshot;

  if (!selection) {
    return <Text variant="body">Nothing selected</Text>;
  }

  if (selection.kind === "shortcut") {
    return (
      <ShortcutSelectionRows
        change={change}
        isSaving={isSaving}
        selection={selection}
      />
    );
  }

  const place = (placement: { height?: number; width?: number }) => {
    change({ selectionOutput: placement });
  };

  return (
    <div className="flex flex-col gap-section">
      {/* `Dimensions` has no disabled state of its own: a save takes the whole
          group out of reach the way the output controls do. */}
      <div className={isSaving ? "pointer-events-none opacity-50" : ""}>
        <Dimensions
          height={selection.height}
          initialLinked
          label="Size"
          layout="stacked"
          onReset={() => {
            change({ resetSelection: true });
          }}
          setDimensions={(width, height) => {
            place({ height, width });
          }}
          setHeight={(height) => {
            place({ height });
          }}
          setWidth={(width) => {
            place({ width });
          }}
          width={selection.width}
        />
      </div>

      <Checkbox
        isDisabled={isSaving}
        isSelected={selection.dropShadow}
        onChange={(next) => {
          change({ selectionDropShadow: next });
        }}
      >
        <span className="text-body">Drop shadow</span>
      </Checkbox>

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Radius</span>
        <SliderNumberField
          aria-label="Radius"
          className="w-48"
          formatOptions={{ maximumFractionDigits: 1, minimumFractionDigits: 1 }}
          isDisabled={isSaving}
          maxValue={50}
          minValue={0}
          onChange={(radius) => {
            change({ selectionRadius: radius });
          }}
          rightSection="%"
          step={0.1}
          value={selection.radius}
        />
      </div>

      {/* A camera track is placed in the screen's picture rather than padded
          against a colour of its own, so it is offered no padding. */}
      {selection.kind === "camera" ? null : (
        <div className="flex flex-col gap-section">
          <div className="flex items-center justify-between gap-section">
            <span className="text-body text-content-fg">Inset</span>
            {/* The bridge carries one kind of request, so every value the knob
                passes is committed as it is reached; the draft holds the one
                just sent until the editor answers with it. */}
            <SliderNumberField
              aria-label="Inset"
              className="w-48"
              isDisabled={isSaving}
              maxValue={Math.max(1, selection.insetMaximum)}
              minValue={0}
              onChange={(inset) => {
                change({ selectionInset: inset });
              }}
              rightSection="px"
              value={Math.min(selection.inset, selection.insetMaximum)}
            />
          </div>
          <div className="flex justify-end">
            <Button
              isDisabled={isSaving}
              onPress={() => {
                change({ recenterSelection: true });
              }}
            >
              Recenter
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
