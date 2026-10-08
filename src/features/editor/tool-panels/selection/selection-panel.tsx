// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { Switch } from "../../../../components/base/switch/switch";
import { Text } from "../../../../components/base/text/text";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { Dimensions } from "../../../../components/shared/dimensions/dimensions";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import { EditorKind } from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { AudioSelectionRows } from "./audio-selection-rows";
import { ShortcutSelectionRows } from "./shortcut-selection-rows";

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
 * An audio track is heard rather than seen: there is nothing to place, so the
 * panel offers how loud it is played back, and for a microphone the speech
 * tools.
 *
 * A keyboard shortcut is drawn rather than placed: it has no source pixels to
 * be sized against, no corners and no pad, so it is shown as the share of the
 * canvas it takes and the point it is centred on.
 */
export function SelectionPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { isLocked, selection } = snapshot;

  if (!selection) {
    return <Text variant="body">{t("editor-panels-nothing-selected")}</Text>;
  }

  if (selection.kind === "shortcut") {
    return (
      <ShortcutSelectionRows
        change={change}
        isLocked={isLocked}
        selection={selection}
      />
    );
  }

  if (selection.kind === "audio") {
    return (
      <AudioSelectionRows
        change={change}
        isLocked={isLocked}
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
      <div className={isLocked ? "pointer-events-none opacity-50" : ""}>
        <Dimensions
          height={selection.height}
          initialLinked
          label={t("editor-panels-size")}
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

      <ControlRow title={t("annotation-radius")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("annotation-radius")}
              className="w-48"
              formatOptions={{
                maximumFractionDigits: 1,
                minimumFractionDigits: 1,
              }}
              isDisabled={isLocked}
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
        )}
      </ControlRow>

      {/* A camera track is placed in the screen's picture rather than padded
          against a colour of its own, so it is offered no padding. */}
      {selection.kind === "camera" ? null : (
        <ControlRow title={t("editor-panels-inset")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              {/* The bridge carries one kind of request, so every value the
                  knob passes is committed as it is reached; the draft holds
                  the one just sent until the editor answers with it. */}
              <SliderNumberField
                aria-label={t("editor-panels-inset")}
                className="w-48"
                isDisabled={isLocked}
                maxValue={Number.MAX_SAFE_INTEGER}
                minValue={0}
                onChange={(inset) => {
                  change({ selectionInset: inset });
                }}
                rightSection="px"
                sliderMaxValue={Math.max(1, selection.insetMaximum)}
                value={selection.inset}
              />
            </div>
          )}
        </ControlRow>
      )}

      <ControlRow title={t("editor-panels-drop-shadow")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={selection.dropShadow}
            onChange={(next) => {
              change({ selectionDropShadow: next });
            }}
          />
        )}
      </ControlRow>

      {selection.kind === "camera" ? null : (
        <div className="flex justify-end">
          <Button
            isDisabled={isLocked}
            onPress={() => {
              change({ recenterSelection: true });
            }}
          >
            {t("editor-panels-recenter")}
          </Button>
        </div>
      )}
    </div>
  );
}
