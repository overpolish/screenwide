// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ColorButton } from "../../../../composites/color-button/color-button";
import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { usePopupPanelPicker } from "../../../popup-panel/use-popup-panel-picker";

import {
  styleColor,
  styleColorLabel,
  TextColorRole,
  offeredColors,
  withStyleColor,
} from "./text-templates";

import type { TextStyle } from "../../../../bindings/TextStyle";

/** The colours of your own, and keeping or forgetting one from the grid. */
export type SavedColors = {
  colors: string[];
  onRemove: (color: string) => void;
  onSave: (color: string) => void;
};

/** The colour grid's own size: two rows of ten small swatches, with the
 * panel's inset. */
const GRID_SIZE = { height: 58, width: 266 };

/** One of the style's colours as a well beside its name: a press opens the
 * colour grid under it, which stays open to try one colour after another. */
function CaptionColorWell({
  isDisabled,
  onChange,
  role,
  savedColors,
  value,
}: {
  isDisabled: boolean;
  onChange: (color: string) => void;
  role: TextColorRole;
  savedColors: SavedColors;
  value: string;
}) {
  const id = `caption-color:${role}`;
  const triggerRef = useRef<HTMLButtonElement>(null);
  const picker = usePopupPanelPicker({
    content: { kind: "colors", savedColors: savedColors.colors, value },
    id,
    label: styleColorLabel(role),
    onAction: (action, color) => {
      if (action === "save") savedColors.onSave(color);
      else savedColors.onRemove(color);
    },
    onPick: onChange,
  });

  return (
    <div
      data-popup-panel-trigger={id}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
    >
      <ColorButton
        aria-expanded={picker.isOpen}
        isDisabled={isDisabled}
        label={styleColorLabel(role)}
        onPress={(event) => {
          const trigger = triggerRef.current;
          if (trigger)
            void picker.toggle(
              trigger,
              GRID_SIZE,
              ["keyboard", "virtual"].includes(event.pointerType),
            );
        }}
        ref={triggerRef}
        value={value}
      />
    </div>
  );
}

/**
 * The colours the style draws in, of the ones its template offers, each a
 * well opening the colour grid: the palette, your own colours after it, and
 * the system's Colours panel behind Custom.
 */
export function TextColorRows({
  isDisabled,
  onChange,
  savedColors,
  style,
}: {
  isDisabled: boolean;
  onChange: (style: Partial<TextStyle>) => void;
  savedColors: SavedColors;
  style: TextStyle;
}) {
  return (
    <ControlRow title={t("editor-panels-text-color")}>
      {(controlProps) => (
        <div
          {...controlProps}
          className="flex flex-wrap justify-end gap-control"
          role="group"
        >
          {offeredColors(style).map((role) => (
            <CaptionColorWell
              isDisabled={isDisabled}
              key={role}
              onChange={(color) => {
                onChange(withStyleColor(style, role, color));
              }}
              role={role}
              savedColors={savedColors}
              value={styleColor(style, role)}
            />
          ))}
        </div>
      )}
    </ControlRow>
  );
}
