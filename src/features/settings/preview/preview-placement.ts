// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { PictureSize } from "./settings-preview-api";
import type { PreviewArea } from "../../../bindings/PreviewArea";
import type { PreviewFade } from "../../../bindings/PreviewFade";
import type { PreviewPlacement } from "../../../bindings/PreviewPlacement";

// Where a native preview goes: the page's box, the part of the page it can
// be seen in, and the fade of the scroll area it is in, which a CSS mask
// cannot carry down to a native layer.

const areaOf = (rect: DOMRect): PreviewArea => ({
  height: rect.height,
  width: rect.width,
  x: rect.left,
  y: rect.top,
});

/** The ancestors that clip `element`, found once: the page's layout does
 * not move a box between scroll areas. */
export function clippingAncestors(element: HTMLElement) {
  const ancestors: HTMLElement[] = [];
  for (let node = element.parentElement; node; node = node.parentElement) {
    const style = getComputedStyle(node);
    if (style.overflowX !== "visible" || style.overflowY !== "visible") {
      ancestors.push(node);
    }
  }
  return ancestors;
}

function visibleArea(ancestors: HTMLElement[]): PreviewArea {
  let left = 0;
  let top = 0;
  let right = window.innerWidth;
  let bottom = window.innerHeight;
  for (const ancestor of ancestors) {
    const rect = ancestor.getBoundingClientRect();
    left = Math.max(left, rect.left);
    top = Math.max(top, rect.top);
    right = Math.min(right, rect.right);
    bottom = Math.min(bottom, rect.bottom);
  }
  return {
    height: Math.max(0, bottom - top),
    width: Math.max(0, right - left),
    x: left,
    y: top,
  };
}

/** The fade of the nearest scroll area the box is in (`ScrollArea` marks
 * its viewport with the px it fades before and after). */
function edgeFade(ancestors: HTMLElement[]): PreviewFade | null {
  const viewport = ancestors.find((ancestor) => ancestor.dataset.fadeAxis);
  if (!viewport) return null;
  const start = Number(viewport.dataset.fadeBefore ?? 0);
  const end = Number(viewport.dataset.fadeAfter ?? 0);
  if (start === 0 && end === 0) return null;
  return {
    area: areaOf(viewport.getBoundingClientRect()),
    end,
    start,
    vertical: viewport.dataset.fadeAxis === "vertical",
  };
}

/** Where the preview in `element` goes now. A `picture` size draws the
 * picture at that size centred on the box, so a resize only moves it;
 * without one it fills the box. */
export function placementOf(
  element: HTMLElement,
  ancestors: HTMLElement[],
  { picture, visible }: { picture: PictureSize | null; visible: boolean },
): PreviewPlacement {
  const frame = areaOf(element.getBoundingClientRect());
  return {
    clip: visibleArea(ancestors),
    fade: edgeFade(ancestors),
    frame,
    picture: picture
      ? {
          ...picture,
          x: frame.x + (frame.width - picture.width) / 2,
          y: frame.y + (frame.height - picture.height) / 2,
        }
      : frame,
    radius: parseFloat(getComputedStyle(element).borderTopLeftRadius),
    visible,
  };
}
