// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useId } from "react";

import { Text } from "../../../components/base/text/text";
import { cn } from "../../../lib/styling";
import { PopupSelect } from "../../popup-panel/popup-select";

import type { SettingControlProps } from "../../../components/shared/setting/setting";

/** Compression levels are the numbers the export backend takes, 0 through 4. */
const compressionOptions = [
  { id: "original", label: "Original" },
  { id: "high", label: "High quality" },
  { id: "balanced", label: "Balanced" },
  { id: "smaller", label: "Smaller file" },
  { id: "smallest", label: "Smallest file" },
];

/**
 * A row of a save sheet: the label right-aligned in the first column and the
 * control in the second. Rows share the sheet's subgrid so every label ends
 * and every control begins on the same line.
 */
export function ExportRow({
  children,
  controlClassName,
  description,
  title,
}: {
  children: (controlProps: SettingControlProps) => ReactNode;
  title: string;
  controlClassName?: string;
  description?: string;
}) {
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  return (
    <div className="col-span-2 grid grid-cols-subgrid">
      {/* The label sits with the first line of its control: 16px of text
          padded to the 24px a control is tall, so the two share a centre. */}
      <Text
        as="label"
        className="py-1 text-right whitespace-nowrap"
        id={titleId}
      >
        {title}
      </Text>
      <div className={cn("flex min-w-0 flex-col gap-tight", controlClassName)}>
        {children({
          "aria-describedby": descriptionId,
          "aria-labelledby": titleId,
        })}
        {description ? (
          <Text
            className="text-content-fg-secondary"
            id={descriptionId}
            variant="subheadline"
          >
            {description}
          </Text>
        ) : null}
      </div>
    </div>
  );
}

/** A pop-up button whose choices open in the popup panel window. */
export function CompressionSelect({
  id,
  isDisabled,
  onChange,
  value,
  ...props
}: SettingControlProps & {
  /** Names the pop-up to the panel window; unique per control. */
  id: string;
  value: number;
  isDisabled?: boolean;
  onChange?: (compression: number) => void;
}) {
  return (
    <PopupSelect
      {...props}
      id={id}
      isDisabled={isDisabled}
      items={compressionOptions}
      label="Quality"
      onSelectionChange={(item) => {
        const index = compressionOptions.findIndex(
          (option) => option.id === item.id,
        );
        if (index >= 0) onChange?.(index);
      }}
      placeholder="Quality"
      selectedId={compressionOptions[value]?.id ?? compressionOptions[0].id}
    />
  );
}
