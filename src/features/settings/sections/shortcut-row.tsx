// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { ControlRow } from "../../../composites/control-row/control-row";
import { HotkeyField } from "../hotkey-field/hotkey-field";

type HotkeyFieldProps = ComponentProps<typeof HotkeyField>;

// One shortcut in the Shortcuts pane: what it does, and the field that
// records it. The field takes the row's title as its name.
export function ShortcutRow({
  captureMode,
  defaultValue,
  description,
  isDisabled,
  label,
  onCaptureChange,
  onChange,
  value,
}: Pick<
  HotkeyFieldProps,
  | "captureMode"
  | "defaultValue"
  | "isDisabled"
  | "onCaptureChange"
  | "onChange"
  | "value"
> & { label: string; description?: string }) {
  return (
    <ControlRow description={description} title={label}>
      {(controlProps) => (
        <HotkeyField
          aria-describedby={controlProps["aria-describedby"]}
          aria-label={label}
          captureMode={captureMode}
          defaultValue={defaultValue}
          isDisabled={isDisabled}
          onCaptureChange={onCaptureChange}
          onChange={onChange}
          value={value}
        />
      )}
    </ControlRow>
  );
}
