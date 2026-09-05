// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowRight, Ban, Clock3, Download } from "lucide-react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { CircularProgress } from "../../components/base/circular-progress/circular-progress";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
import { Text } from "../../components/base/text/text";
import { WindowHeader } from "../../components/shared/window-header/window-header";

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

  return (
    <main className="window-surface gap-section flex h-full w-full flex-col overflow-hidden rounded-window text-content-fg">
      <WindowHeader
        actions={
          currentVersion || updateVersion ? (
            <Text
              className="gap-control-inset flex shrink-0 items-center font-mono"
              variant="help"
            >
              {currentVersion ? <span>v{currentVersion}</span> : null}
              {currentVersion && updateVersion ? (
                <ArrowRight
                  aria-label="to"
                  className="size-icon-compact shrink-0"
                />
              ) : null}
              {updateVersion ? <span>v{updateVersion}</span> : null}
            </Text>
          ) : null
        }
        leadingSection={
          <img
            alt="Screenwide"
            className="brightness-0 dark:invert"
            draggable={false}
            src={logoUrl}
          />
        }
        onClose={busy ? undefined : onRemindLater}
        title="Update available"
      />
      <div className="gap-section px-window-inset pb-window-inset flex min-h-0 grow flex-col">
        <section
          aria-label="What's new"
          className="gap-section flex min-h-0 grow flex-col"
        >
          <div className="gap-section flex shrink-0 flex-wrap items-center justify-between">
            <h2 className="m-0 text-lg font-semibold">What’s new</h2>
            {released ? <Text variant="help">Released {released}</Text> : null}
          </div>
          <ScrollArea
            edgeEffect="inset"
            rootClassName="min-h-0 grow"
            scrollbarAutoHide="never"
          >
            {releaseNotes ? (
              <ReleaseNotes html={releaseNotes} />
            ) : (
              <Text>Improvements and fixes for Screenwide.</Text>
            )}
          </ScrollArea>
        </section>

        {status === "downloading" ? (
          <div
            className="gap-control-inset flex shrink-0 items-center"
            role="status"
          >
            <CircularProgress
              aria-label="Downloading and installing"
              isIndeterminate={downloadProgress === null}
              size="compact"
              value={
                downloadProgress === null ? undefined : downloadProgress * 100
              }
            />
            <Text variant="help">Downloading and installing</Text>
            {downloadProgress !== null ? (
              <Text className="ml-auto font-mono" variant="help">
                {Math.round(downloadProgress * 100)}%
              </Text>
            ) : null}
          </div>
        ) : null}

        {status === "error" && error ? (
          <Alert color="error">
            The update could not be installed. {error}
          </Alert>
        ) : null}

        <div className="gap-control-inset flex shrink-0 flex-wrap items-center justify-end">
          <Button isDisabled={busy} onPress={onSkipVersion} variant="ghost">
            <Ban />
            Skip this version
          </Button>
          <Button isDisabled={busy} onPress={onRemindLater}>
            <Clock3 />
            Later
          </Button>
          <Button color="primary" isDisabled={busy} onPress={onInstall}>
            <Download />
            {busy ? "Installing" : "Update and restart"}
          </Button>
        </div>
      </div>
    </main>
  );
}
