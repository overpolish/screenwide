// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ColorButton } from "../../../../../composites/color-button/color-button";
import { ControlRow } from "../../../../../composites/control-row/control-row";
import { t } from "../../../../../i18n/i18n";
import { Button } from "../../../../../primitives/button/button";
import { useColorPicker } from "../../../../../primitives/input-fields/use-color-picker";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";

import { Background } from "./background";
import { MeshPalette } from "./mesh-palette";
import { useBackgroundEdit } from "./use-background-edit";

const backgroundKinds = () => [
  { id: "solid", label: t("background-kind-solid") },
  { id: "mesh", label: t("background-kind-mesh") },
];

/**
 * A background built by hand, and saved under a name if it is worth keeping.
 *
 * Returning to Mesh preserves its palette and generator while rerolling the
 * arrangement, just like pressing Mesh when it is already selected.
 */
export function BackgroundEditor({
  isDisabled,
  onChange,
  onSavePreset,
  value,
}: {
  onChange: (background: Background) => void;
  onSavePreset: (background: Background) => void;
  value: Background;
  isDisabled?: boolean;
}) {
  const edit = useBackgroundEdit({ onChange, value });
  const solidPicker = useColorPicker(edit.setSolid);

  return (
    <div className="flex flex-col gap-separate">
      <ControlRow title={t("background-type")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <PillGroup
              aria-label={t("background-type-label")}
              isDisabled={isDisabled}
              items={backgroundKinds().map((item) =>
                item.id === "mesh"
                  ? {
                      ...item,
                      onPress: () => {
                        if (edit.kind === "mesh") edit.reroll();
                      },
                    }
                  : item,
              )}
              onSelectionChange={(kind) => {
                edit.setKind(kind === "mesh" ? "mesh" : "solid");
              }}
              selected={edit.kind}
            />
          </div>
        )}
      </ControlRow>

      {edit.kind === "solid" || !edit.mesh ? (
        <ControlRow title={t("background-color")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <ColorButton
                aria-label={t("background-color-label")}
                isDisabled={isDisabled}
                onPress={(event) => {
                  solidPicker.open(edit.solidColor, event.target);
                }}
                value={edit.solidColor}
              />
            </div>
          )}
        </ControlRow>
      ) : (
        <MeshPalette
          colors={edit.palette}
          isDisabled={isDisabled}
          locked={edit.locked}
          onChange={edit.setColors}
          onLockedChange={edit.setLocked}
        />
      )}

      {/* A preset is its picture; it needs no name to be found again. */}
      <div className="flex justify-end">
        <Button
          isDisabled={isDisabled}
          onPress={() => {
            onSavePreset(value);
          }}
        >
          {t("background-save-preset")}
        </Button>
      </div>
    </div>
  );
}
