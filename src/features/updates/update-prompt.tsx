// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import logoUrl from "../../assets/screenwide-mark.svg";
import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { GroupBox } from "../../components/base/group-box/group-box";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";
import { ProgressPanel } from "../../components/shared/progress-panel/progress-panel";
import { WindowHeader } from "../../components/shared/window-header/window-header";
import { WindowShell } from "../../components/shared/window-shell/window-shell";

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

const displayDate = (date: string | null) => {
  if (!date) return null;
  const parsed = new Date(date);
  if (Number.isNaN(parsed.getTime())) return null;
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(
    parsed,
  );
};

const availabilityLine = (
  currentVersion: string | null,
  updateVersion: string | null,
) => {
  if (!updateVersion) return "A new version of Screenwide is available.";
  if (!currentVersion) return `Screenwide ${updateVersion} is available.`;
  return `Screenwide ${updateVersion} is available. You have ${currentVersion}.`;
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
  const busy = status === "downloading";
  const released = displayDate(releaseDate);
  const percent =
    downloadProgress === null ? null : Math.round(downloadProgress * 100);

  return (
    <WindowShell
      header={
        <WindowHeader
          leadingSection={
            <img
              alt="Screenwide"
              className="brightness-0 dark:invert"
              draggable={false}
              src={logoUrl}
            />
          }
          onClose={busy ? undefined : onRemindLater}
          title="Update Available"
        />
      }
    >
      <div className="flex min-h-0 grow flex-col gap-section px-window-inset pb-window-inset">
        {busy ? (
          <ProgressPanel
            label="Downloading and installing"
            progress={percent}
            progressLabel="Update download progress"
            secondary={percent === null ? undefined : `${String(percent)}%`}
          />
        ) : (
          <div className="flex flex-col gap-tight">
            <Text>{availabilityLine(currentVersion, updateVersion)}</Text>
            {released ? (
              <Text className="text-content-fg-secondary" variant="subheadline">
                Released {released}
              </Text>
            ) : null}
          </div>
        )}

        {/* The box owns the notes' inset, so the scroll area reaches its inner
            edge and the scrollbar rides there rather than floating inside. */}
        <GroupBox
          className="flex min-h-0 grow flex-col [&>div]:min-h-0 [&>div]:grow [&>div>*]:px-0 [&>div>*]:py-0"
          title="Release Notes"
        >
          <ScrollArea
            className="px-section"
            edgeEffect="none"
            rootClassName="min-h-0 grow"
          >
            {releaseNotes ? (
              <ReleaseNotes html={releaseNotes} />
            ) : (
              <Text>Improvements and fixes for Screenwide.</Text>
            )}
          </ScrollArea>
        </GroupBox>

        {status === "error" && error ? (
          <Alert color="error">
            The update could not be installed. {error}
          </Alert>
        ) : null}

        <div className="flex shrink-0 items-center gap-control-inset">
          <Button
            className="mr-auto"
            isDisabled={busy}
            onPress={onSkipVersion}
            variant="ghost"
          >
            Skip This Version
          </Button>
          <Button isDisabled={busy} onPress={onRemindLater}>
            Remind Me Later
          </Button>
          <Button color="primary" isDisabled={busy} onPress={onInstall}>
            {busy ? "Installing" : "Install Update"}
          </Button>
        </div>
      </div>
    </WindowShell>
  );
}
