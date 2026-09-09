// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Stands in for a captured display in stories: a two-colour gradient at the
 * given aspect, as a data URI the real `<img>` loads exactly as it loads an
 * asset URL from the cache.
 */
export const displayThumbnail = (
  from: string,
  to: string,
  ratio = 16 / 10,
): string => {
  const height = 100;
  const width = Math.round(height * ratio);
  const id = `g${from.replace(/[^a-z0-9]/gi, "")}`;
  const svg = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${String(width)} ${String(height)}">`,
    `<defs><linearGradient id="${id}" x1="0" x2="1" y1="0" y2="1">`,
    `<stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/>`,
    `</linearGradient></defs>`,
    `<rect width="${String(width)}" height="${String(height)}" fill="url(#${id})"/>`,
    `</svg>`,
  ].join("");
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
};
