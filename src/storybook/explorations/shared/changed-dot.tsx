// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A row whose value differs from the default carries a dot after its name;
// pressing the dot resets the row. A sketch, offered beside a row's title,
// until it is decided whether Control Row takes it on.
export function ChangedDot({ setting }: { setting: string }) {
  return (
    <span
      aria-label={`Reset ${setting}`}
      className="size-1.5 rounded-full bg-primary"
      role="button"
    />
  );
}
