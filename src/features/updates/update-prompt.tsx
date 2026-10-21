// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useDateFormatter, useNumberFormatter } from "react-aria";

import logoUrl from "../../assets/screenwide-mark.svg";
import { ProgressPanel } from "../../composites/progress-panel/progress-panel";
import { WindowHeader } from "../../composites/window-header/window-header";
import { WindowShell } from "../../composites/window-shell/window-shell";
import { t } from "../../i18n/i18n";
import { Alert } from "../../primitives/alert/alert";
import { Button } from "../../primitives/button/button";
import { ScrollArea } from "../../primitives/scroll-area/scroll-area";

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
            <p className="text-body text-fg">
              {availabilityLine(currentVersion, updateVersion)}
            </p>
            {released ? (
              <p className="text-body text-fg-secondary text-content-fg-secondary">
                {t("updates-released", {
                  date: dateFormatter.format(released),
                })}
              </p>
            ) : null}
          </div>
        )}

        <section className="flex min-h-0 grow flex-col gap-inline">
          <h2 className="text-heading text-fg-secondary">
            {t("updates-release-notes")}
          </h2>
          <ScrollArea rootClassName="min-h-0 grow">
            {releaseNotes ? (
              <ReleaseNotes html={releaseNotes} />
            ) : (
              <p className="text-body text-fg">
                {t("updates-release-notes-fallback")}
              </p>
            )}
          </ScrollArea>
        </section>

        {status === "error" && error ? (
          <Alert color="error">{t("updates-failed", { error })}</Alert>
        ) : null}

        <div className="flex shrink-0 items-center gap-control-inset">
          <Button
            className="me-auto"
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
