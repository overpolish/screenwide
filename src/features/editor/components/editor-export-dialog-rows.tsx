// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { ListBoxItem } from "../../../components/base/listbox-item/listbox-item";
import { Select } from "../../../components/base/select/select";
import { Setting } from "../../../components/shared/setting/setting";

import type { SettingControlProps } from "../../../components/shared/setting/setting";

/** Compression levels are the numbers the export backend takes, 0 through 4. */
const compressionOptions = [
  { id: "original", label: "Original" },
  { id: "high", label: "High quality" },
  { id: "balanced", label: "Balanced" },
  { id: "smaller", label: "Smaller file" },
  { id: "smallest", label: "Smallest file" },
];

/** Rows share the dialog's subgrid so labels and controls line up. */
export function ExportRow({
  children,
  controlClassName = "min-w-0 w-full",
  description,
  title,
}: {
  children: (controlProps: SettingControlProps) => ReactNode;
  title: string;
  controlClassName?: string;
  description?: string;
}) {
  return (
    <Setting
      className="col-span-2 grid grid-cols-subgrid"
      controlClassName={controlClassName}
      description={description}
      title={title}
    >
      {children}
    </Setting>
  );
}

export function CompressionSelect({
  isDisabled,
  onChange,
  value,
  ...props
}: SettingControlProps & {
  value: number;
  isDisabled?: boolean;
  onChange?: (compression: number) => void;
}) {
  return (
    <Select
      {...props}
      className="w-full"
      clearable={false}
      isDisabled={isDisabled}
      items={compressionOptions}
      onChange={(key) => {
        const index = compressionOptions.findIndex(
          (option) => option.id === String(key),
        );
        if (index >= 0) onChange?.(index);
      }}
      value={compressionOptions[value]?.id ?? compressionOptions[0].id}
    >
      {(item) => (
        <ListBoxItem id={item.id} textValue={item.label}>
          {item.label}
        </ListBoxItem>
      )}
    </Select>
  );
}
