// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

const ELLIPSIS = "...";

/**
 * Middle-truncate a raw string, keeping its head and tail around an ellipsis.
 * Used as the graceful fallback for paths that are really one long segment (or
 * whose final segment alone already overflows the cap).
 */
const middleTruncate = (
  value: string,
  maxLength: number,
  suffixLength = 0,
): string => {
  if (value.length <= maxLength) return value;
  if (maxLength <= ELLIPSIS.length) return value.slice(0, maxLength);
  const keep = maxLength - ELLIPSIS.length;
  const back = Math.min(keep, Math.max(Math.floor(keep / 2), suffixLength));
  const front = keep - back;
  return value.slice(0, front) + ELLIPSIS + value.slice(value.length - back);
};

/**
 * A destination path shortened for the titlebar's folder button.
 *
 * Paths within `maxLength` are returned unchanged. Longer ones keep the root
 * segment and as many trailing segments as fit around an ellipsis, e.g.
 * `/Users/.../2026/August` or `C:\...\Some Folder\Final Folder`, so the user
 * still recognizes both the volume/root and the leaf they are exporting into.
 *
 * The path's own separator is preserved so it reads natively on each platform:
 * a Windows path (any `\` present) is rejoined with `\`, otherwise `/`. A
 * leading root separator is kept too, so an absolute POSIX path shows
 * `/Users/...` rather than losing its leading slash; Windows paths (`C:\...`)
 * have no leading separator and keep the volume (`C:`) as their root.
 *
 * The ellipsis lands only at separator boundaries, never inside a segment, so
 * the result is stable regardless of where a name happens to contain a space -
 * unlike CSS `text-overflow: ellipsis`, which WebKit renders with a stray gap
 * when the clip point falls on whitespace.
 */
export const truncatePath = (
  directory: string,
  maxLength = 30,
  kind: "file" | "folder" = "folder",
): string => {
  if (!Number.isInteger(maxLength) || maxLength < 4)
    throw new RangeError("Path label length must be an integer of at least 4.");
  if (directory.length <= maxLength) return directory;

  // Present the path with its own separator (Windows `\`, otherwise `/`) so it
  // matches how the OS shows it; a leading separator is kept for POSIX roots.
  const separator = directory.includes("\\") ? "\\" : "/";
  const rootSeparator = /^[\\/]/.test(directory) ? separator : "";
  const segments = directory.split(/[\\/]/).filter(Boolean);
  const leaf = segments[segments.length - 1] ?? directory;
  const dot = leaf.lastIndexOf(".");
  const suffixLength = kind === "file" && dot > 0 ? leaf.length - dot : 0;
  const budget = Math.max(maxLength, suffixLength + 4);
  if (segments.length <= 1)
    return middleTruncate(directory, budget, suffixLength);

  const unc = /^[\\/]{2}[^\\/]/.test(directory) && segments.length >= 2;
  const rootCount = unc ? 2 : 1;
  const root = unc
    ? separator.repeat(2) + segments.slice(0, 2).join(separator)
    : rootSeparator + segments[0];
  // UNC roots and file extensions are preserved even if they exceed the budget.
  if (unc && segments.length === 2) return root;
  const last = segments[segments.length - 1];

  // Floor: root + ellipsis + leaf. Grow inward from the leaf while it fits.
  const tail = [last];
  for (let index = segments.length - 2; index >= rootCount; index--) {
    const candidate = [root, ELLIPSIS, segments[index], ...tail].join(
      separator,
    );
    if (candidate.length > maxLength) break;
    tail.unshift(segments[index]);
  }

  const result = [root, ELLIPSIS, ...tail].join(separator);
  // Even root + ellipsis + leaf can overflow when the leaf is huge; degrade to
  // a plain middle-truncation of the whole path so something legible remains.
  if (result.length <= maxLength) return result;
  if (unc) {
    const prefix = root + separator + ELLIPSIS + separator;
    return (
      prefix +
      middleTruncate(
        leaf,
        Math.max(suffixLength + 4, maxLength - prefix.length),
        suffixLength,
      )
    );
  }
  return middleTruncate(directory, budget, suffixLength);
};
