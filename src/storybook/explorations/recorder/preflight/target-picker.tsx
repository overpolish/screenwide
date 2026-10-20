// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { type TargetId, TARGETS } from "../shared/recorder-data";

// Four labelled choices, each icon over its name as in the macOS screenshot
// toolbar, so they fit the panel's width without dropping their labels.
// Number keys 1 to 4 pick them.

export function TargetPicker({
  onChange,
  value,
}: {
  onChange: (target: TargetId) => void;
  value: TargetId;
}) {
  return (
    <RadioGroup
      aria-label="Record"
      className="grid grid-cols-4 gap-tight"
      onChange={(id) => {
        onChange(id as TargetId);
      }}
      orientation="horizontal"
      value={value}
    >
      {TARGETS.map(({ icon: Icon, id, label }) => (
        <RadioField key={id} value={id}>
          <RadioButton className="flex w-full flex-col items-center gap-tight rounded-control py-tight text-footnote text-content-fg-secondary outline-none data-[focus-visible]:ring-2 data-[focus-visible]:ring-primary data-[hovered]:bg-control-subtle-hover data-[selected]:bg-fill data-[selected]:text-content-fg">
            <Icon className="size-icon" />
            {label}
          </RadioButton>
        </RadioField>
      ))}
    </RadioGroup>
  );
}
