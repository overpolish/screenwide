// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ban, Clock3, Download } from "lucide-react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Alert } from "../../components/base/alert/alert";
import { Button } from "../../components/base/button/button";
import { ScrollArea } from "../../components/base/scroll-area/scroll-area";
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
        leadingSection={
          <img
            alt="Screenwide"
            className="brightness-0 dark:invert"
            draggable={false}
            src={logoUrl}
          />
        }
        onClose={busy ? undefined : onRemindLater}
        title="Software Update"
      />
      <div className="px-window-inset pb-window-inset flex min-h-0 grow flex-col gap-4">
        <Alert color="neutral">
          <div>
            <p className="font-semibold">
              {updateVersion
                ? `Screenwide version ${updateVersion} is available`
                : "A new Screenwide version is available"}
            </p>
            {currentVersion || released ? (
              <p className="mt-0.5 text-muted">
                {currentVersion
                  ? `You are currently using version ${currentVersion}.`
                  : null}
                {released ? ` Released ${released}.` : null}
              </p>
            ) : null}
          </div>
        </Alert>

        <section className="flex min-h-0 grow flex-col overflow-hidden rounded-lg border border-muted/20 bg-neutral">
          <div className="border-b border-muted/15 px-4 py-2.5 text-xs font-semibold text-muted">
            What’s new
          </div>
          <ScrollArea className="px-4 py-3" rootClassName="min-h-0 grow">
            {releaseNotes ? (
              <ReleaseNotes html={releaseNotes} />
            ) : (
              <p className="text-xs leading-relaxed text-muted">
                This update includes improvements and fixes for Screenwide.
              </p>
            )}
          </ScrollArea>
        </section>

        {status === "downloading" ? (
          <div>
            <div className="mb-1.5 flex justify-between text-xs text-muted">
              <span>Downloading and installing…</span>
              {downloadProgress === null ? null : (
                <span>{Math.round(downloadProgress * 100)}%</span>
              )}
            </div>
            <div className="h-1.5 overflow-hidden rounded-full bg-muted/15">
              <div
                className={`h-full rounded-full bg-info transition-[width] ${
                  downloadProgress === null ? "w-1/3 animate-pulse" : ""
                }`}
                style={
                  downloadProgress === null
                    ? undefined
                    : {
                        width: `${String(Math.round(downloadProgress * 100))}%`,
                      }
                }
              />
            </div>
          </div>
        ) : null}

        {status === "error" && error ? (
          <Alert color="error">
            The update could not be installed. {error}
          </Alert>
        ) : null}

        <div className="flex items-center justify-end gap-2">
          <Button
            isDisabled={busy}
            onPress={onSkipVersion}
            size="compact"
            variant="ghost"
          >
            <Ban size={14} />
            Skip this version
          </Button>
          <Button isDisabled={busy} onPress={onRemindLater} size="compact">
            <Clock3 size={14} />
            Not right now
          </Button>
          <Button
            color="primary"
            isDisabled={busy}
            onPress={onInstall}
            size="compact"
          >
            <Download size={14} />
            {busy ? "Installing…" : "Update and restart"}
          </Button>
        </div>
      </div>
    </main>
  );
}
