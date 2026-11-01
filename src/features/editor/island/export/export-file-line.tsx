// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { PathField } from "../../../../composites/path-field/path-field";
import { t } from "../../../../i18n/i18n";
import { TextField } from "../../../../primitives/input-fields/text-field";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";

import type { ScreenshotFormat } from "../../../../bindings/ScreenshotFormat";

/** The file the export writes, first in the section: its name, typed in
 * place; the folder it goes in, which is chosen again from its button; and
 * its format, a choice for a screenshot, or for a recording, whose
 * container is set, the one format it is written in, chosen and disabled. */
export function ExportFileLine({
  directory,
  extension,
  fileStem,
  onBrowse,
  onFileStemChange,
  onScreenshotFormatChange,
  screenshotFormat,
}: {
  /** Null before a folder is known. */
  directory: string | null;
  extension: string;
  fileStem: string;
  onBrowse: () => void;
  onFileStemChange: (fileStem: string) => void;
  onScreenshotFormatChange: (format: ScreenshotFormat) => void;
  /** Null for a recording. */
  screenshotFormat: ScreenshotFormat | null;
}) {
  return (
    <div className="flex items-center gap-separate">
      <FileNameField onChange={onFileStemChange} value={fileStem} />
      <PathField
        aria-label={t("editor-export-folder")}
        className="w-36"
        onBrowse={onBrowse}
        value={directory}
      />
      {screenshotFormat ? (
        <PillGroup
          aria-label={t("editor-export-format")}
          className="ms-auto"
          items={[
            { id: "png", label: t("editor-export-format-png") },
            { id: "jpeg", label: t("editor-export-format-jpeg") },
          ]}
          onSelectionChange={(id) => {
            onScreenshotFormatChange(id as ScreenshotFormat);
          }}
          selected={screenshotFormat}
        />
      ) : (
        // A recording's container is set, so it is the one format offered,
        // chosen and not to be changed, drawn as a screenshot's choice is.
        <PillGroup
          aria-label={t("editor-export-format")}
          className="ms-auto"
          isDisabled
          items={[{ id: extension, label: extension.toUpperCase() }]}
          onSelectionChange={() => undefined}
          selected={extension}
        />
      )}
    </div>
  );
}

/** The name typed here goes to the editor and comes back with the next
 * snapshot, a moment later; while the field has focus, what was typed
 * stands, so a late echo never moves the caret or drops a letter. */
function FileNameField({
  onChange,
  value,
}: {
  onChange: (value: string) => void;
  value: string;
}) {
  const [typed, setTyped] = useState<string | null>(null);
  return (
    <TextField
      aria-label={t("editor-export-file-name")}
      className="w-44 shrink-0"
      isFramed
      onBlur={() => {
        setTyped(null);
      }}
      onChange={(next) => {
        setTyped(next);
        onChange(next);
      }}
      value={typed ?? value}
    />
  );
}
