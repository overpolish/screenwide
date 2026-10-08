// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useId } from "react";

import { Text } from "../../../components/base/text/text";
import { t } from "../../../i18n/i18n";
import { PopupSelect } from "../../popup-panel/popup-select";

import type { ControlRowControlProps } from "../../../components/shared/control-row/control-row";

/** Compression levels are the numbers the export backend takes, 0 through 4. */
const compressionOptions = () => [
  { id: "original", label: t("editor-export-quality-original") },
  { id: "high", label: t("editor-export-quality-high") },
  { id: "balanced", label: t("editor-export-quality-balanced") },
  { id: "smaller", label: t("editor-export-quality-smaller") },
  { id: "smallest", label: t("editor-export-quality-smallest") },
];

/**
 * A row of a save sheet: the label right-aligned in the first column and the
 * control in the second. Rows share the sheet's subgrid so every label ends
 * and every control begins on the same line.
 *
 * The trailing layout is for a checkbox: it sits at the row's end with its
 * description just before it on the same line.
 */
export function ExportRow({
  children,
  description,
  layout = "field",
  title,
}: {
  children: (controlProps: ControlRowControlProps) => ReactNode;
  title: string;
  description?: string;
  layout?: "field" | "trailing";
}) {
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  const control = children({
    "aria-describedby": descriptionId,
    "aria-labelledby": titleId,
  });
  const descriptionText = description ? (
    <Text id={descriptionId} variant="subheadline">
      {description}
    </Text>
  ) : null;

  return (
    <div className="col-span-2 grid grid-cols-subgrid">
      {/* The label sits with the first line of its control: 16px of text
          padded to the 24px a control is tall, so the two share a centre. */}
      <Text as="label" className="py-1 text-end whitespace-nowrap" id={titleId}>
        {title}
      </Text>
      {layout === "trailing" ? (
        <div className="flex min-h-control-height min-w-0 items-center justify-end gap-control-inset text-end">
          {descriptionText}
          <div className="flex shrink-0 items-center">{control}</div>
        </div>
      ) : (
        <div className="flex min-w-0 flex-col gap-tight">
          {control}
          {descriptionText}
        </div>
      )}
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
}: ControlRowControlProps & {
  /** Names the pop-up to the panel window; unique per control. */
  id: string;
  value: number;
  isDisabled?: boolean;
  onChange?: (compression: number) => void;
}) {
  const options = compressionOptions();
  return (
    <PopupSelect
      {...props}
      id={id}
      isDisabled={isDisabled}
      items={options}
      label={t("editor-export-quality")}
      onSelectionChange={(item) => {
        const index = options.findIndex((option) => option.id === item.id);
        if (index >= 0) onChange?.(index);
      }}
      placeholder={t("editor-export-quality")}
      selectedId={options[value]?.id ?? options[0].id}
    />
  );
}
