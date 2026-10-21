// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Takes both separators, for macOS and Windows paths, and ignores a trailing
// one. A root has no name of its own, so it is returned whole.
export const folderName = (path: string) => {
  const trimmed = path.replace(/(?<=.)[\\/]+$/, "");
  const unc = /^\\\\[^\\]+\\[^\\]+$/.exec(trimmed);
  if (unc) return trimmed;
  const name = trimmed.split(/[\\/]/).pop() ?? "";
  if (name === "" || /^[A-Za-z]:$/.test(name)) return path;
  return name;
};
