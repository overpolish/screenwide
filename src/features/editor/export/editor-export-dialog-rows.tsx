// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useId } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { PopupSelect } from "../../popup-panel/popup-select";

import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { ControlRowControlProps } from "../../../composites/control-row/control-row";

/** Compression levels are the numbers the export backend takes, 0 through 4. */
const compressionOptions = () => [
  { id: "original", label: t("editor-export-quality-original") },
  { id: "high", label: t("editor-export-quality-high") },
  { id: "balanced", label: t("editor-export-quality-balanced") },
  { id: "smaller", label: t("editor-export-quality-smaller") },
  { id: "smallest", label: t("editor-export-quality-smallest") },
];

const subtitleOptions = (): { id: ExportSubtitles; label: string }[] => [
  { id: "none", label: t("editor-export-subtitles-none") },
  { id: "srt", label: t("editor-export-subtitles-srt") },
  { id: "vtt", label: t("editor-export-subtitles-vtt") },
];

/**
 * A row of a save sheet: the label right-aligned in the first column and the
 * control in the second. Rows share the sheet's subgrid so every label ends
 * and every control begins on the same line.
 *
 * The trailing layout is for a checkbox: it sits at the row's end with its
 * description just before it on the same line. The end layout is for a
 * control at its own width, such as a pill picker: it sits at the row's end,
 * its description under it and ending with it.
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
  layout?: "end" | "field" | "trailing";
}) {
  const id = useId();
  const titleId = `${id}-title`;
  const descriptionId = description ? `${id}-description` : undefined;

  const control = children({
    "aria-describedby": descriptionId,
    "aria-labelledby": titleId,
  });
  const descriptionText = description ? (
    <p className="text-body text-fg-secondary" id={descriptionId}>
      {description}
    </p>
  ) : null;

  return (
    <div className="col-span-2 grid grid-cols-subgrid">
      {/* The label sits with the first line of its control: 16px of text
          padded to the 24px a control is tall, so the two share a centre. */}
      <label
        className="text-body text-fg py-1 text-end whitespace-nowrap"
        id={titleId}
      >
        {title}
      </label>
      {layout === "trailing" ? (
        <div className="flex min-h-control-height min-w-0 items-center justify-end gap-control-inset text-end">
          {descriptionText}
          <div className="flex shrink-0 items-center">{control}</div>
        </div>
      ) : (
        <div
          className={cn(
            "flex min-w-0 flex-col gap-tight",
            layout === "end" && "items-end text-end",
          )}
        >
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

/** The subtitle file written beside a recording's video. It is made of the
 * transcript, so without one the row shows the None the export will act on. */
export function SubtitlesRow({
  hasTranscript,
  isDisabled,
  onChange,
  value,
}: {
  hasTranscript: boolean;
  value: ExportSubtitles;
  isDisabled?: boolean;
  onChange?: (subtitles: ExportSubtitles) => void;
}) {
  const options = subtitleOptions();
  return (
    <ExportRow
      description={
        hasTranscript
          ? t("editor-export-subtitles-description")
          : t("editor-export-subtitles-needs-transcript")
      }
      layout="end"
      title={t("editor-export-subtitles")}
    >
      {() => (
        <PillGroup
          aria-label={t("editor-export-subtitles")}
          display="label"
          isDisabled={isDisabled || !hasTranscript}
          items={options}
          onSelectionChange={(id) => {
            const chosen = options.find((option) => option.id === id);
            if (chosen) onChange?.(chosen.id);
          }}
          selected={hasTranscript ? value : "none"}
        />
      )}
    </ExportRow>
  );
}
