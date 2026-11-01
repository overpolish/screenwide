// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The native video panes render BELOW the webview. Every element that paints
 * a background over the preview area declares `data-preview-backdrop`, and
 * this mask punches holes over the pane rects so the video shows through
 * while all DOM controls stay naturally on top. The holes are plain gradient
 * layers: an image resource (e.g. an SVG data URI) would be re-fetched every
 * time a hole changes size mid-pan, and the async load flashes the whole
 * background out. Rounded output corners are covered by the container's
 * colour-matched backstop behind the panes. The viewport's own rounded
 * corners, where it meets the window's chrome, round the hole, and the
 * native drawables are clipped to the same curve.
 */

/** How round each corner of the viewport is, in CSS pixels. */
export type ViewportCorners = {
  bottomLeft: number;
  bottomRight: number;
  topLeft: number;
  topRight: number;
};

/** The viewport's corners as its CSS rounds them. */
export const viewportCorners = (viewport: HTMLElement): ViewportCorners => {
  const style = getComputedStyle(viewport);
  const radius = (value: string) => Number.parseFloat(value) || 0;
  return {
    bottomLeft: radius(style.borderBottomLeftRadius),
    bottomRight: radius(style.borderBottomRightRadius),
    topLeft: radius(style.borderTopLeftRadius),
    topRight: radius(style.borderTopRightRadius),
  };
};

type Hole = {
  corners: ViewportCorners;
  height: number;
  width: number;
  x: number;
  y: number;
};

const applyBackdropMask = (element: HTMLElement, holes: Hole[]) => {
  const bounds = element.getBoundingClientRect();
  const key = JSON.stringify({
    height: Math.round(bounds.height * 100) / 100,
    holes,
    width: Math.round(bounds.width * 100) / 100,
  });
  if (element.dataset.previewBackdropKey === key) return;
  element.dataset.previewBackdropKey = key;
  if (holes.length === 0) {
    element.style.removeProperty("clip-path");
    element.style.removeProperty("-webkit-clip-path");
    element.style.removeProperty("mask-image");
    element.style.removeProperty("mask-size");
    element.style.removeProperty("mask-position");
    element.style.removeProperty("mask-repeat");
    element.style.removeProperty("mask-composite");
    return;
  }
  const rounded = holes.some(({ corners }) =>
    Object.values(corners).some((radius) => radius > 0),
  );
  if (rounded) {
    const roundedRect = (hole: Hole) => {
      const fit = (radius: number) =>
        Math.min(Math.max(0, radius), hole.width / 2, hole.height / 2);
      const topLeft = fit(hole.corners.topLeft);
      const topRight = fit(hole.corners.topRight);
      const bottomRight = fit(hole.corners.bottomRight);
      const bottomLeft = fit(hole.corners.bottomLeft);
      const right = hole.x + hole.width;
      const bottom = hole.y + hole.height;
      const arc = (radius: number, x: number, y: number) =>
        `A ${radius.toString()} ${radius.toString()} 0 0 1 ${x.toString()} ${y.toString()}`;
      return [
        `M ${(hole.x + topLeft).toString()} ${hole.y.toString()}`,
        `H ${(right - topRight).toString()}`,
        arc(topRight, right, hole.y + topRight),
        `V ${(bottom - bottomRight).toString()}`,
        arc(bottomRight, right - bottomRight, bottom),
        `H ${(hole.x + bottomLeft).toString()}`,
        arc(bottomLeft, hole.x, bottom - bottomLeft),
        `V ${(hole.y + topLeft).toString()}`,
        arc(topLeft, hole.x + topLeft, hole.y),
        "Z",
      ].join(" ");
    };
    const path = [
      `M 0 0 H ${bounds.width.toString()} V ${bounds.height.toString()} H 0 Z`,
      ...holes.map(roundedRect),
    ].join(" ");
    const clipPath = `path(evenodd, '${path}')`;
    if (
      CSS.supports("clip-path", clipPath) ||
      CSS.supports("-webkit-clip-path", clipPath)
    ) {
      element.style.setProperty("clip-path", clipPath);
      element.style.setProperty("-webkit-clip-path", clipPath);
      element.style.removeProperty("mask-image");
      element.style.removeProperty("mask-size");
      element.style.removeProperty("mask-position");
      element.style.removeProperty("mask-repeat");
      element.style.removeProperty("mask-composite");
      return;
    }
  }
  element.style.removeProperty("clip-path");
  element.style.removeProperty("-webkit-clip-path");
  element.style.maskImage = [
    ...holes.map(() => "linear-gradient(#fff,#fff)"),
    "linear-gradient(#fff,#fff)",
  ].join(", ");
  element.style.maskSize = [
    ...holes.map(
      (hole) => `${hole.width.toString()}px ${hole.height.toString()}px`,
    ),
    "100% 100%",
  ].join(", ");
  element.style.maskPosition = [
    ...holes.map((hole) => `${hole.x.toString()}px ${hole.y.toString()}px`),
    "0 0",
  ].join(", ");
  element.style.maskRepeat = "no-repeat";
  element.style.maskComposite = [...holes.map(() => "exclude"), "add"].join(
    ", ",
  );
};

export type PreviewBackdrop = [number, number, number, number];

let backdropProbe: CanvasRenderingContext2D | null = null;
const backdropCache = new Map<string, PreviewBackdrop>();

const compositeBackdrop = (selector: string): PreviewBackdrop => {
  const layers = Array.from(
    document.querySelectorAll<HTMLElement>(selector),
    (element) => getComputedStyle(element).backgroundColor,
  );
  const key = `${selector}|${layers.join("|")}`;
  const cached = backdropCache.get(key);
  if (cached) return cached;
  if (!backdropProbe) {
    const canvas = document.createElement("canvas");
    canvas.width = 1;
    canvas.height = 1;
    backdropProbe = canvas.getContext("2d", { willReadFrequently: true });
    if (!backdropProbe) return [0, 0, 0, 1];
  }
  backdropProbe.clearRect(0, 0, 1, 1);
  backdropProbe.globalCompositeOperation = "source-over";
  for (const layer of layers) {
    backdropProbe.fillStyle = layer;
    backdropProbe.fillRect(0, 0, 1, 1);
  }
  const pixel = backdropProbe.getImageData(0, 0, 1, 1).data;
  const colour: PreviewBackdrop = [
    pixel[0] / 255,
    pixel[1] / 255,
    pixel[2] / 255,
    pixel[3] / 255,
  ];
  backdropCache.set(key, colour);
  return colour;
};

/**
 * The effective viewport backdrop: the translucent background layers
 * composited bottom-up over transparency. Windows gives this RGBA surface to
 * DirectComposition, so it is blended over the same live window material as
 * the neighbouring WebView pixels instead of approximating them over black.
 * A 1x1 canvas
 * does the compositing because computed backgrounds arrive in any CSS colour
 * syntax (`rgb(... / 0.92)`, `color(srgb ...)`), all of which `fillStyle`
 * understands.
 */
export const effectiveBackdrop = (): PreviewBackdrop => {
  return compositeBackdrop("[data-preview-backdrop]");
};

export const clearBackdropMasks = () => {
  for (const element of document.querySelectorAll<HTMLElement>(
    "[data-preview-backdrop]",
  )) {
    applyBackdropMask(element, []);
  }
};

export const updatePreviewBackdropMasks = (
  viewportRect: DOMRect,
  corners: ViewportCorners,
) => {
  for (const element of document.querySelectorAll<HTMLElement>(
    "[data-preview-backdrop]",
  )) {
    const elementRect = element.getBoundingClientRect();
    const holes: Hole[] =
      viewportRect.width >= 1 && viewportRect.height >= 1
        ? [
            {
              corners,
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
