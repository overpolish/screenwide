// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type PictureTone = "dark" | "light";

/** The corner the card's badges cover, as shares of the picture: from 55%
 * across and 78% down to its bottom-right. */
const CORNER = { height: 0.22, width: 0.45 };
/** Small enough to read back at once; the corner's average is all we need. */
const SAMPLE = { height: 18, width: 32 };

/**
 * Whether the bottom-right of a loaded picture is light or dark, so what is
 * drawn over it can take the matching appearance. Null when the picture
 * cannot be read, such as one served without leave to read its pixels.
 */
export function pictureCornerTone(image: HTMLImageElement): PictureTone | null {
  const canvas = document.createElement("canvas");
  canvas.width = SAMPLE.width;
  canvas.height = SAMPLE.height;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context || image.naturalWidth === 0) return null;
  context.drawImage(image, 0, 0, SAMPLE.width, SAMPLE.height);
  const x = Math.floor(SAMPLE.width * (1 - CORNER.width));
  const y = Math.floor(SAMPLE.height * (1 - CORNER.height));
  let pixels: Uint8ClampedArray;
  try {
    pixels = context.getImageData(
      x,
      y,
      SAMPLE.width - x,
      SAMPLE.height - y,
    ).data;
  } catch {
    return null;
  }
  let luminance = 0;
  let count = 0;
  for (let index = 0; index < pixels.length; index += 4) {
    // Transparent parts of a still show the card, not the picture.
    const alpha = pixels[index + 3] / 255;
    if (alpha === 0) continue;
    luminance +=
      (0.2126 * pixels[index] +
        0.7152 * pixels[index + 1] +
        0.0722 * pixels[index + 2]) /
      255;
    count += 1;
  }
  if (count === 0) return null;
  return luminance / count > 0.5 ? "light" : "dark";
}
