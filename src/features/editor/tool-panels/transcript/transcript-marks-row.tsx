// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { Button } from "../../../../components/base/button/button";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";

/**
 * A row that cuts every word of one kind the transcript marks, fillers or
 * profanity, as one undo step. It says how many there are, and while Remove
 * is hovered or focused the panel shows what it would cut, as Remove silences
 * does.
 */
export function TranscriptMarksRow({
  count,
  isLocked,
  onPreview,
  onRemove,
  title,
}: {
  count: number;
  isLocked: boolean;
  /** Told whether Remove is hovered or focused. */
  onPreview: (isPreviewing: boolean) => void;
  onRemove: () => void;
  title: string;
}) {
  const [isHovered, setIsHovered] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const isDisabled = isLocked || count === 0;
  const isPreviewing = (isHovered || isFocused) && !isDisabled;
  useEffect(() => {
    onPreview(isPreviewing);
  }, [isPreviewing, onPreview]);
  return (
    <ControlRow
      description={
        count > 0 ? t("editor-panels-transcript-found", { count }) : undefined
      }
      title={title}
    >
      {(controlProps) => (
        <Button
          {...controlProps}
          isDisabled={isDisabled}
          onFocusChange={setIsFocused}
          onHoverChange={setIsHovered}
          onPress={onRemove}
        >
          {t("editor-panels-remove")}
        </Button>
      )}
    </ControlRow>
  );
}
