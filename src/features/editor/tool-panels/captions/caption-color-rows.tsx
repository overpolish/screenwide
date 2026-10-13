// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { AnnotationColorGrid } from "../../../../components/shared/annotation-style/annotation-color-grid";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

import {
  CAPTION_COLOR_KEYS,
  captionColorLabel,
  CaptionColorRole,
  offeredColors,
} from "./caption-templates";

import type { CaptionStyle } from "../../../../bindings/CaptionStyle";

/**
 * The colours the style draws in: which of them to change, of the ones its
 * template offers, and the colour grid for it, with your own colours after
 * the palette and the system's Colours panel behind Custom.
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
  const offered = offeredColors(style);
  const [chosen, setChosen] = useState<CaptionColorRole>("text");
  // A colour the style stopped offering, as when the highlight is turned
  // off, gives way to the type's.
  const role = offered.includes(chosen) ? chosen : "text";
  const key = CAPTION_COLOR_KEYS[role];
  const set = (color: string) => {
    onChange({ [key]: color });
  };

  return (
    <>
      {offered.length > 1 ? (
        <ControlRow title={t("editor-panels-captions-color")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("editor-panels-captions-color")}
                display="label"
                isDisabled={isDisabled}
                items={offered.map((id) => ({
                  id,
                  label: captionColorLabel(id),
                }))}
                onSelectionChange={(next) => {
                  setChosen(next as CaptionColorRole);
                }}
                selected={role}
              />
            </div>
          )}
        </ControlRow>
      ) : null}
      <AnnotationColorGrid
        isDisabled={isDisabled}
        onChange={set}
        onSettled={set}
        savedColors={savedColors}
        value={style[key]}
      />
    </>
  );
}
