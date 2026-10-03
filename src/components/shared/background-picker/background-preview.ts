// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { CSSProperties, useEffect, useState } from "react";

import { Background } from "./background";
import { DEFAULT_GENERATOR_ID } from "./background-generators";

/**
 * A local file as the webview can show it. Outside the app, where there is no
 * asset protocol to convert against, the path is taken as a URL so a story can
 * hand the picker a picture.
 */
const backgroundImageSource = (path: string) =>
  isTauri() ? convertFileSrc(path) : path;

/**
 * A background as a tile draws it, without the renderer.
 *
 * The composition mesh is the same one the native renderer paints, laid out
 * in CSS: the first colour fills the tile and every blob is an ellipse of the
 * next one, at its own share of the tile and under its own rotation. The
 * other generators are shaders with no geometry to borrow, so their likeness
 * is their palette as a diagonal sweep. Either way it is a likeness rather
 * than the renderer's own picture, and stands in while the real swatch is
 * being rendered, and in a story, where there is no renderer.
 */
export const backgroundTileStyle = (background: Background): CSSProperties => {
  if (background.kind === "solid") return { backgroundColor: background.color };
  if (background.kind === "image")
    return {
      backgroundImage: `url("${backgroundImageSource(background.path)}")`,
      backgroundPosition: "center",
      backgroundSize: "cover",
    };
  if (background.generator !== DEFAULT_GENERATOR_ID) {
    const stops = background.colors.map((color, index) => {
      const position =
        background.colors.length > 1
          ? (index / (background.colors.length - 1)) * 100
          : 0;
      return `${color} ${position.toFixed(1)}%`;
    });
    return {
      backgroundColor: background.colors[0],
      backgroundImage: `linear-gradient(135deg, ${stops.join(", ")})`,
    };
  }
  const [base, ...blobColors] = background.colors;
  const points = background.points ?? [];
  const layers = blobColors.slice(0, points.length).map((color, index) => {
    const point = points[index];
    return (
      `radial-gradient(${point.radiusX.toFixed(1)}% ${point.radiusY.toFixed(1)}% ` +
      `at ${point.x.toFixed(1)}% ${point.y.toFixed(1)}%, ${color} 0%, transparent 72%)`
    );
  });
  return {
    backgroundColor: base,
    backgroundImage: layers.join(", "),
  };
};

/** The swatch's edge in device pixels, so a tile is crisp on a retina screen
 * without asking for a picture larger than a tile can show. */
const thumbnailPixels = (size: number) =>
  Math.round(size * Math.min(globalThis.devicePixelRatio || 1, 3));

/** A background and a size as one name: two tiles that ask for the same
 * background share one render. */
const thumbnailKey = (background: Background, size: number) =>
  `${JSON.stringify(background)}@${String(size)}`;

/** One rendered swatch per background and size, shared by every tile that
 * shows it: a grid asks for each mesh once, and a reopened panel finds it
 * already in hand. A picture is not held: the desktop's picture keeps its
 * path when the wallpaper changes, and the native side names the swatch by
 * the file's contents, so it is asked again each time a tile mounts. */
const thumbnails = new Map<string, Promise<string>>();

const renderThumbnail = (background: Background, size: number) =>
  invoke<string>("render_background_thumbnail", { background, size }).then(
    convertFileSrc,
  );

const thumbnailUrl = (background: Background, size: number) => {
  if (background.kind === "image") return renderThumbnail(background, size);
  const key = thumbnailKey(background, size);
  const held = thumbnails.get(key);
  if (held) return held;
  const rendered = renderThumbnail(background, size).catch((error: unknown) => {
    // A failed render leaves the CSS likeness in place rather than an empty
    // tile, and is not held, so the next look tries again.
    thumbnails.delete(key);
    throw error;
  });
  thumbnails.set(key, rendered);
  return rendered;
};

/**
 * The background painted by the code that paints it for real, at a tile's
 * size.
 *
 * Null until it arrives, and for the backgrounds a tile can draw itself: a
 * flat colour is a flat colour, and outside the app there is no renderer to
 * ask, so the CSS likeness stands in both times.
 */
export const useBackgroundThumbnail = (
  background: Background | undefined,
  size: number,
): string | null => {
  const pixels = thumbnailPixels(size);
  // The background travels as a value rather than as an identity, so a
  // rerender with an equal background asks for nothing new.
  const key = background ? thumbnailKey(background, pixels) : null;
  const [rendered, setRendered] = useState<{ key: string; url: string } | null>(
    null,
  );

  useEffect(() => {
    if (!key || !background || background.kind === "solid" || !isTauri())
      return;
    let current = true;
    void thumbnailUrl(background, pixels).then(
      (url) => {
        if (current) setRendered({ key, url });
      },
      () => {
        // The CSS likeness is already on the tile, and stays.
      },
    );
    return () => {
      current = false;
    };
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [key, pixels]);

  return rendered && rendered.key === key ? rendered.url : null;
};
