// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

type PictureTone = "dark" | "light";

/** The parts of a card's picture something is drawn over, as shares of it
 * measured from the side text starts on: the selection box at the top start,
 * the tracks badge at the top end, and the length and size badges at the
 * bottom end. */
const CORNERS = {
  bottomEnd: { height: 0.22, width: 0.45, x: 0.55, y: 0.78 },
  topEnd: { height: 0.22, width: 0.45, x: 0.55, y: 0 },
  topStart: { height: 0.22, width: 0.2, x: 0, y: 0 },
};

export type PictureCorner = keyof typeof CORNERS;
export type PictureTones = Record<PictureCorner, PictureTone | null>;

/** Small enough to read back at once; each corner's average is all we need. */
const SAMPLE = { height: 18, width: 32 };

/**
 * Whether each corner of a loaded picture is light or dark, so what is drawn
 * over it can take the matching appearance. The corners turn with the writing
 * `direction`, as the badges over them do. Null when the picture cannot be
 * read, such as one served without leave to read its pixels.
 */
export function pictureTones(
  image: HTMLImageElement,
  direction: "ltr" | "rtl",
): PictureTones | null {
  const canvas = document.createElement("canvas");
  canvas.width = SAMPLE.width;
  canvas.height = SAMPLE.height;
  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context || image.naturalWidth === 0) return null;
  context.drawImage(image, 0, 0, SAMPLE.width, SAMPLE.height);
  let pixels: Uint8ClampedArray;
  try {
    pixels = context.getImageData(0, 0, SAMPLE.width, SAMPLE.height).data;
  } catch {
    return null;
  }

  const toneOf = ({ height, width, x, y }: (typeof CORNERS)[PictureCorner]) => {
    if (direction === "rtl") x = 1 - x - width;
    const left = Math.floor(SAMPLE.width * x);
    const top = Math.floor(SAMPLE.height * y);
    const right = Math.ceil(SAMPLE.width * (x + width));
    const bottom = Math.ceil(SAMPLE.height * (y + height));
    let luminance = 0;
    let count = 0;
    for (let row = top; row < bottom; row += 1) {
      for (let column = left; column < right; column += 1) {
        const index = (row * SAMPLE.width + column) * 4;
        // Transparent parts of a still show the card, not the picture.
        if (pixels[index + 3] === 0) continue;
        luminance +=
          (0.2126 * pixels[index] +
            0.7152 * pixels[index + 1] +
            0.0722 * pixels[index + 2]) /
          255;
        count += 1;
      }
    }
    if (count === 0) return null;
    return luminance / count > 0.5 ? "light" : "dark";
  };

  return {
    bottomEnd: toneOf(CORNERS.bottomEnd),
    topEnd: toneOf(CORNERS.topEnd),
    topStart: toneOf(CORNERS.topStart),
  };
}
