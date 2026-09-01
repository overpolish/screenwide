// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** One labelled settings line: name and one-liner on the left, control right. */
export function SettingRow({
  children,
  description,
  label,
}: {
  children: React.ReactNode;
  description: string;
  label: string;
}) {
  return (
    <div className="flex min-h-15 items-center gap-4 py-3">
      <div className="min-w-0 grow">
        <div className="text-sm font-medium">{label}</div>
        <div className="mt-0.5 text-xs text-muted">{description}</div>
      </div>
      <div className="shrink-0 whitespace-nowrap">{children}</div>
    </div>
  );
}
