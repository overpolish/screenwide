// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { AnnotationColorGrid } from "../../composites/annotation-style/annotation-color-grid";

import { useFittedPanel } from "./use-fitted-panel";
import { reportPopupPanelPick } from "./use-popup-panel-picker";

import type { PopupPanelColorsContent } from "./picker-content";

/**
 * A colour grid in the panel window: each colour pressed, or chosen in the
 * system Colours panel behind Custom, is reported to the window that opened
 * it, and the panel stays open to try another. A colour kept or forgotten is
 * reported the same way, since the list of them belongs to that window.
 */
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
    <div className="w-full p-control-inset" ref={contentRef}>
      <AnnotationColorGrid
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
