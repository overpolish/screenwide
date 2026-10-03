// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { NumberField } from "../../../../components/base/input-fields/number-field";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { ToolPanelPatch } from "../tool-panel-patch";
import { ToolPanelShortcutSelection } from "../tool-panel-store";

/**
 * The Select tool's controls for the shortcut on screen: how big it is drawn
 * and where it is centred, both as a share of the finished picture.
 *
 * Every number takes the path the drag on the shortcut takes, so one typed
 * here and one dragged out there are the same edit. The two actions end the
 * group: the shortcut back where the recording drew it, or every other
 * shortcut moved to join this one.
 */
export function ShortcutSelectionRows({
  change,
  isLocked,
  selection,
}: {
  change: (values: ToolPanelPatch) => void;
  isLocked: boolean;
  selection: ToolPanelShortcutSelection;
}) {
  // The slider needs a range to run over, so a canvas narrow enough that the
  // shortcut's own floor meets its ceiling still leaves one to scrub.
  const maximum = Math.max(selection.maximumSizePercent, 10);
  const minimum = Math.min(selection.minimumSizePercent, maximum - 5);
  return (
    <div className="flex flex-col gap-section">
      <ControlRow title="Size">
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label="Size"
              className="w-48"
              isDisabled={isLocked}
              maxValue={maximum}
              minValue={minimum}
              onChange={(sizePercent) => {
                change({ shortcutPlacement: { sizePercent } });
              }}
              rightSection="%"
              step={5}
              value={Math.min(
                Math.max(selection.sizePercent, minimum),
                maximum,
              )}
            />
          </div>
        )}
      </ControlRow>

      <ControlRow title="Position">
        {(controlProps) => (
          <div {...controlProps} className="flex gap-control" role="group">
            <NumberField
              aria-label="Shortcut X position"
              className="w-20"
              isDisabled={isLocked}
              leftSection="X"
              maxValue={100}
              minValue={0}
              onChange={(positionXPercent) => {
                change({ shortcutPlacement: { positionXPercent } });
              }}
              rightSection="%"
              showSteppers={false}
              step={1}
              value={selection.positionXPercent}
            />
            <NumberField
              aria-label="Shortcut Y position"
              className="w-20"
              isDisabled={isLocked}
              leftSection="Y"
              maxValue={100}
              minValue={0}
              onChange={(positionYPercent) => {
                change({ shortcutPlacement: { positionYPercent } });
              }}
              rightSection="%"
              showSteppers={false}
              step={1}
              value={selection.positionYPercent}
            />
          </div>
        )}
      </ControlRow>

      <div className="flex justify-end gap-control">
        <Button
          isDisabled={isLocked}
          onPress={() => {
            change({ resetShortcut: true });
          }}
        >
          Reset
        </Button>
        <Button
          isDisabled={isLocked}
          onPress={() => {
            change({ applyShortcutToAll: true });
          }}
        >
          Apply to all
        </Button>
      </div>
    </div>
  );
}
