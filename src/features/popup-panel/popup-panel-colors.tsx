// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ColorGrid } from "../../composites/color-grid/color-grid";

import { useFittedPanel } from "./use-fitted-panel";
import { reportPopupPanelPick } from "./use-popup-panel-picker";

import type { PopupPanelColorsContent } from "./picker-content";

// The panel stays open to try another colour. A colour kept or forgotten is
// reported too, since the list of them belongs to the window that opened it.
/** A colour grid in a panel window, reporting each pick to its opener. */
export function PopupPanelColors({
  content,
  id,
}: {
  content: PopupPanelColorsContent;
  id: string;
}) {
  const contentRef = useRef<HTMLDivElement>(null);
  useFittedPanel(contentRef, id);
  const pick = (color: string) => {
    reportPopupPanelPick(id, color);
  };

  return (
    <div className="w-full p-inset-sm" ref={contentRef}>
      <ColorGrid
        offersCustom
        onChange={pick}
        onRemoveColor={(color) => {
          reportPopupPanelPick(id, color, "remove");
        }}
        onSaveColor={(color) => {
          reportPopupPanelPick(id, color, "save");
        }}
        savedColors={content.savedColors}
        value={content.value}
      />
    </div>
  );
}
