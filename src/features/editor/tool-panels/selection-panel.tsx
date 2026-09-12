// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { Text } from "../../../components/base/text/text";
import { Dimensions } from "../../../components/shared/dimensions/dimensions";
import { EditorKind } from "../types";

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
 */
export function SelectionPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { isSaving, selection } = snapshot;

  if (!selection) {
    return <Text variant="body">Nothing selected</Text>;
  }

  const place = (placement: { height?: number; width?: number }) => {
    console.debug("[selection-panel] change", placement, {
      shown: { height: selection.height, width: selection.width },
    });
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
    </div>
  );
}
