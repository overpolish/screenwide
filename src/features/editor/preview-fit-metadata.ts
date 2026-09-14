// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { previewFitWidth } from "./components/preview-transform";
import { applyBackdropMask, Hole } from "./preview-backdrop";

export const updatePreviewFitMetadata = (
  viewport: HTMLElement,
  width: number,
  height: number,
) => {
  const fitWidth = previewFitWidth(
    width,
    height,
    viewport.getBoundingClientRect().height,
  );
  if (fitWidth !== undefined)
    viewport.dataset.previewFitWidth = fitWidth.toString();
};

export const updatePreviewBackdropMasks = (viewportRect: DOMRect) => {
  for (const element of document.querySelectorAll<HTMLElement>(
    "[data-preview-backdrop]",
  )) {
    const elementRect = element.getBoundingClientRect();
    const holes: Hole[] =
      viewportRect.width >= 1 && viewportRect.height >= 1
        ? [
            {
              height: Math.round(viewportRect.height * 100) / 100,
              width: Math.round(viewportRect.width * 100) / 100,
              x: Math.round((viewportRect.left - elementRect.left) * 100) / 100,
              y: Math.round((viewportRect.top - elementRect.top) * 100) / 100,
            },
          ]
        : [];
    applyBackdropMask(element, holes);
  }
};
