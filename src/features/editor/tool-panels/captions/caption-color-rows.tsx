// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ColorChip } from "../../../../components/base/input-fields/color-chip";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { usePopupPanelPicker } from "../../../popup-panel/use-popup-panel-picker";

import {
  captionColor,
  captionColorLabel,
  CaptionColorRole,
  offeredColors,
  withCaptionColor,
} from "./caption-templates";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

/** The colour grid's own width: the palette's row, with its padding. */
const GRID_SIZE = { height: 120, width: 340 };

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
  role: CaptionColorRole;
  savedColors: string[];
  value: string;
}) {
  const id = `caption-color:${role}`;
  const triggerRef = useRef<HTMLButtonElement>(null);
  const picker = usePopupPanelPicker({
    content: { kind: "colors", savedColors, value },
    id,
    label: captionColorLabel(role),
    onPick: onChange,
  });

  return (
    <div
      data-popup-panel-trigger={id}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
    >
      <ColorChip
        aria-expanded={picker.isOpen}
        isDisabled={isDisabled}
        label={captionColorLabel(role)}
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
export function CaptionColorRows({
  isDisabled,
  onChange,
  savedColors,
  style,
}: {
  isDisabled: boolean;
  onChange: (style: Partial<CaptionStyle>) => void;
  savedColors: string[];
  style: CaptionStyle;
}) {
  return (
    <ControlRow title={t("editor-panels-captions-color")}>
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
                onChange(withCaptionColor(style, role, color));
              }}
              role={role}
              savedColors={savedColors}
              value={captionColor(style, role)}
            />
          ))}
        </div>
      )}
    </ControlRow>
  );
}
