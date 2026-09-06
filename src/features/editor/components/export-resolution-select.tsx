// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Badge } from "../../../components/base/badge/badge";
import { ListBoxItem } from "../../../components/base/listbox-item/listbox-item";
import { Select } from "../../../components/base/select/select";

import type { SettingControlProps } from "../../../components/shared/setting/setting";

export type ExportResolutionItem = {
  height: number;
  id: string;
  label: string;
  width: number;
};

/** Items belong to the current output canvas (or separate camera output). */
export function ExportResolutionSelect({
  isDisabled,
  items,
  onChange,
  value,
  ...props
}: SettingControlProps & {
  items: ExportResolutionItem[];
  value: string;
  isDisabled?: boolean;
  onChange?: (value: string) => void;
}) {
  const options = items.map((item) => ({
    ...item,
    dimensions: `${item.width.toString()} × ${item.height.toString()}`,
  }));
  return (
    <Select
      {...props}
      className="w-full"
      clearable={false}
      isDisabled={isDisabled}
      items={options}
      onChange={(key) => {
        onChange?.(String(key));
      }}
      renderValue={(item) =>
        item ? (
          <span className="gap-control flex w-full items-center justify-between">
            <span>{item.dimensions}</span>
            <Badge>{item.label}</Badge>
          </span>
        ) : null
      }
      value={value}
    >
      {(item) => (
        <ListBoxItem
          id={item.id}
          textValue={`${item.dimensions} | ${item.label}`}
        >
          <span className="gap-section flex items-center justify-between">
            <span>{item.dimensions}</span>
            <Badge>{item.label}</Badge>
          </span>
        </ListBoxItem>
      )}
    </Select>
  );
}
