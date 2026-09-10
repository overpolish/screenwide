// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupSelect } from "../../popup-panel/popup-select";

import type { SettingControlProps } from "../../../components/shared/setting/setting";

export type ExportResolutionItem = {
  height: number;
  id: string;
  label: string;
  width: number;
};

const dimensions = (item: ExportResolutionItem) =>
  `${item.width.toString()} × ${item.height.toString()}`;

/**
 * Items belong to the current output canvas (or separate camera output). A
 * pop-up button whose choices open in the popup panel window, so the list
 * can reach past the sheet's edge the way a native pop-up menu does.
 */
export function ExportResolutionSelect({
  id,
  isDisabled,
  items,
  onChange,
  value,
  ...props
}: SettingControlProps & {
  /** Names the pop-up to the panel window; unique per control. */
  id: string;
  items: ExportResolutionItem[];
  value: string;
  isDisabled?: boolean;
  onChange?: (value: string) => void;
}) {
  return (
    <PopupSelect
      {...props}
      id={id}
      isDisabled={isDisabled}
      items={items.map((item) => ({
        detail: item.label,
        id: item.id,
        label: dimensions(item),
      }))}
      label="Size"
      onSelectionChange={(item) => {
        onChange?.(item.id);
      }}
      placeholder="Size"
      selectedId={value}
    />
  );
}
