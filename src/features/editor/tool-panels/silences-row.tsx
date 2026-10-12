// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { Button } from "../../../components/base/button/button";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { formatDuration } from "../duration";

import { ToolProgress } from "./selection/tool-progress";
import { ToolPanelSilences } from "./tool-panel-selection";

const silencesDescription = ({
  count,
  durationMs,
  status,
}: ToolPanelSilences) => {
  if (status === "none-found") return t("editor-panels-silences-none");
  return status === "idle" && count > 0
    ? t("editor-panels-silences-removed", {
        count,
        duration: formatDuration(durationMs),
      })
    : undefined;
};

/**
 * Remove silences, one feature shown in two places: the microphone's Select
 * panel and the Transcript panel read the same state and ask the same work
 * of the editor. Its own Restore all appears where `onRestore` is given; the
 * Transcript panel leaves it out, since its Restore all brings silences back
 * with the words. Restore all stays in place while there is nothing to
 * restore, as auto zoom's Clear all does, so the row never shifts as cuts
 * come and go. A bar under the row shows how far listening has got.
 */
export function SilencesRow({
  isLocked,
  onPreview,
  onRemove,
  onRestore,
  silences,
}: {
  isLocked: boolean;
  onRemove: () => void;
  silences: ToolPanelSilences;
  /** Told whether Remove is hovered or focused, so a panel that can show
   * what Remove acts on shows it while that is so. */
  onPreview?: (isPreviewing: boolean) => void;
  onRestore?: () => void;
}) {
  const isFinding = silences.status === "finding";
  const [isHovered, setIsHovered] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const isPreviewing = (isHovered || isFocused) && !isLocked && !isFinding;
  useEffect(() => {
    onPreview?.(isPreviewing);
  }, [isPreviewing, onPreview]);
  return (
    <div>
      <ControlRow
        controlClassName="gap-control"
        description={silencesDescription(silences)}
        title={t("editor-panels-silences")}
      >
        {(controlProps) => (
          <>
            <Button
              {...controlProps}
              isDisabled={isLocked || isFinding}
              onFocusChange={setIsFocused}
              onHoverChange={setIsHovered}
              onPress={onRemove}
            >
              {t("editor-panels-remove")}
            </Button>
            {onRestore ? (
              <Button
                aria-label={t("editor-panels-restore-silences")}
                isDisabled={isLocked || isFinding || silences.count === 0}
                onPress={onRestore}
              >
                {t("editor-panels-restore-all")}
              </Button>
            ) : null}
          </>
        )}
      </ControlRow>
      <ToolProgress
        isWorking={isFinding && silences.progress !== null}
        label={t("editor-panels-silences-finding")}
        value={silences.progress ?? 0}
      />
    </div>
  );
}
