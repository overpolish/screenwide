// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useDateFormatter, useNumberFormatter } from "react-aria";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { GroupBox } from "../../components/base/group-box/group-box";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";
import { ProgressPanel } from "../../components/shared/progress-panel/progress-panel";
import { WindowHeader } from "../../components/shared/window-header/window-header";
import { WindowShell } from "../../components/shared/window-shell/window-shell";
import { t } from "../../i18n/i18n";

import { ReleaseNotes } from "./release-notes";

import type { UpdateStatus } from "./use-update";

export type UpdatePromptProps = {
  currentVersion: string | null;
  downloadProgress: number | null;
  error: string | null;
  onInstall: () => void;
  onRemindLater: () => void;
  onSkipVersion: () => void;
  releaseDate: string | null;
  releaseNotes: string | null;
  status: UpdateStatus;
  updateVersion: string | null;
};

const parseDate = (date: string | null) => {
  if (!date) return null;
  const parsed = new Date(date);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
};

const availabilityLine = (
  currentVersion: string | null,
  updateVersion: string | null,
) => {
  if (!updateVersion) return t("updates-available");
  if (!currentVersion) {
    return t("updates-available-version", { version: updateVersion });
  }
  return t("updates-available-versions", {
    current: currentVersion,
    version: updateVersion,
  });
};

export function UpdatePrompt({
  currentVersion,
  downloadProgress,
  error,
  onInstall,
  onRemindLater,
  onSkipVersion,
  releaseDate,
  releaseNotes,
  status,
  updateVersion,
}: UpdatePromptProps) {
  const dateFormatter = useDateFormatter({ dateStyle: "medium" });
  const percentFormatter = useNumberFormatter({ style: "percent" });
  const busy = status === "downloading";
  const released = parseDate(releaseDate);
  const percent =
    downloadProgress === null ? null : Math.round(downloadProgress * 100);

  return (
    <WindowShell
      header={
        <WindowHeader
          leadingSection={
            <img
              alt={t("updates-logo")}
              className="brightness-0 dark:invert"
              draggable={false}
              src={logoUrl}
            />
          }
          onClose={busy ? undefined : onRemindLater}
          title={t("updates-title")}
        />
      }
    >
      <div className="flex min-h-0 grow flex-col gap-section px-window-inset pb-window-inset">
        {busy ? (
          <ProgressPanel
            label={t("updates-downloading")}
            progress={percent}
            progressLabel={t("updates-download-progress")}
            secondary={
              percent === null
                ? undefined
                : percentFormatter.format(percent / 100)
            }
          />
        ) : (
          <div className="flex flex-col gap-tight">
            <Text>{availabilityLine(currentVersion, updateVersion)}</Text>
            {released ? (
              <Text className="text-content-fg-secondary" variant="subheadline">
                {t("updates-released", {
                  date: dateFormatter.format(released),
                })}
              </Text>
            ) : null}
          </div>
        )}

        {/* The box owns the notes' inset, so the scroll area reaches its inner
            edge and the scrollbar rides there rather than floating inside. */}
        <GroupBox
          className="flex min-h-0 grow flex-col [&>div]:min-h-0 [&>div]:grow [&>div>*]:px-0 [&>div>*]:py-0"
          title={t("updates-release-notes")}
        >
          <ScrollArea
            className="px-section"
            edgeEffect="none"
            rootClassName="min-h-0 grow"
          >
            {releaseNotes ? (
              <ReleaseNotes html={releaseNotes} />
            ) : (
              <Text>{t("updates-release-notes-fallback")}</Text>
            )}
          </ScrollArea>
        </GroupBox>

        {status === "error" && error ? (
          <Alert color="error">{t("updates-failed", { error })}</Alert>
        ) : null}

        <div className="flex shrink-0 items-center gap-control-inset">
          <Button
            className="mr-auto"
            isDisabled={busy}
            onPress={onSkipVersion}
            variant="ghost"
          >
            {t("updates-skip")}
          </Button>
          <Button isDisabled={busy} onPress={onRemindLater}>
            {t("updates-remind-later")}
          </Button>
          <Button color="primary" isDisabled={busy} onPress={onInstall}>
            {busy ? t("updates-installing") : t("updates-install")}
          </Button>
        </div>
      </div>
    </WindowShell>
  );
}
