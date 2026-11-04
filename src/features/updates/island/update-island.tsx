// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlertCircle, CircleArrowDown } from "lucide-react";
import { useDateFormatter, useNumberFormatter } from "react-aria";

import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { Island } from "../../../primitives/island/island";
import { IslandParts } from "../../../primitives/island/island-parts";
import { type IslandProps } from "../../../primitives/island/island-props";
import { IslandRow } from "../../../primitives/island/island-row";
import { IslandRowTitle } from "../../../primitives/island/row-title/island-row-title";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { Text } from "../../../primitives/text/text";
import { IslandWorkLine } from "../../editor/island/island-work-line";
import { ReleaseNotes } from "../release-notes";

import type { UpdateStatus } from "../use-update";

// A new version offered on the island: what it is at the row's start, with
// What's new, Later and Install at its end. What's new hangs the release
// notes under the row, scrolling, with Skip this version as a quiet action at
// their foot. Install clears the row into the download, its progress under
// it, then the install, after which the app restarts; a failure says so with
// Try again.

export type UpdateIslandProps = Pick<
  IslandProps,
  "isShown" | "notch" | "onHidden" | "onShapeChange"
> & {
  currentVersion: string | null;
  /** How far the download has got, 0 to 1; null while it cannot say. */
  downloadProgress: number | null;
  error: string | null;
  isNotesOpen: boolean;
  onInstall: () => void;
  onLater: () => void;
  onNotesOpenChange: (isOpen: boolean) => void;
  onSkip: () => void;
  releaseDate: string | null;
  /** The release notes as GitHub's sanitized HTML. */
  releaseNotes: string | null;
  status: UpdateStatus;
  updateVersion: string | null;
};

type Phase = "available" | "downloading" | "error" | "installing";

const phaseOf = ({
  downloadProgress,
  status,
}: Pick<UpdateIslandProps, "downloadProgress" | "status">): Phase => {
  if (status === "downloading")
    return downloadProgress === 1 ? "installing" : "downloading";
  return status === "error" ? "error" : "available";
};

const parseDate = (date: string | null) => {
  if (!date) return null;
  const parsed = new Date(date);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
};

/** The release notes, scrolling at the island's reading width, and the
 * quiet way never to be offered this version again. */
function Notes({
  onSkip,
  releaseNotes,
}: Pick<UpdateIslandProps, "onSkip" | "releaseNotes">) {
  return (
    <div className="flex w-96 flex-col items-start gap-inline">
      <ScrollArea
        className="px-inset-sm"
        constrainHeight
        rootClassName="max-h-64 w-full"
      >
        {releaseNotes ? (
          <ReleaseNotes html={releaseNotes} />
        ) : (
          <Text variant="body">{t("updates-release-notes-fallback")}</Text>
        )}
      </ScrollArea>
      <Button onPress={onSkip} variant="ghost">
        {t("updates-skip")}
      </Button>
    </div>
  );
}

export function UpdateIsland(props: UpdateIslandProps) {
  const {
    downloadProgress,
    error,
    isNotesOpen,
    isShown = true,
    notch,
    onHidden,
    onInstall,
    onLater,
    onNotesOpenChange,
    onShapeChange,
    releaseDate,
    updateVersion,
  } = props;
  const dateFormatter = useDateFormatter({ dateStyle: "medium" });
  const percentFormatter = useNumberFormatter({ style: "percent" });
  const phase = phaseOf(props);
  const version = updateVersion ?? "";
  const released = parseDate(releaseDate);

  const titles: Record<Phase, string> = {
    available: t("updates-available-version", { version }),
    downloading: t("updates-downloading", { version }),
    error: t("updates-failed", { version }),
    installing: t("updates-installing", { version }),
  };
  const end =
    phase === "available" ? (
      <span className="flex items-center gap-1">
        <Button
          aria-expanded={isNotesOpen}
          onPress={() => {
            onNotesOpenChange(!isNotesOpen);
          }}
          variant="ghost"
        >
          {t("updates-whats-new")}
        </Button>
        <Button onPress={onLater}>{t("updates-later")}</Button>
        <Button color="primary" onPress={onInstall}>
          {t("updates-install")}
        </Button>
      </span>
    ) : phase === "error" ? (
      <span className="flex items-center gap-1">
        <Button onPress={onLater}>{t("updates-later")}</Button>
        <Button color="primary" onPress={onInstall}>
          {t("updates-try-again")}
        </Button>
      </span>
    ) : null;

  let section = null;
  if (phase === "available" && isNotesOpen)
    section = (
      <div key="notes">
        <Notes onSkip={props.onSkip} releaseNotes={props.releaseNotes} />
      </div>
    );
  else if (phase === "downloading")
    section = (
      <div key="work">
        <IslandWorkLine
          label={
            downloadProgress === null
              ? t("updates-download-starting")
              : t("updates-download-percent", {
                  percent: percentFormatter.format(downloadProgress),
                })
          }
          value={downloadProgress}
        />
      </div>
    );
  else if (phase === "installing")
    section = (
      <div key="work">
        <IslandWorkLine label={t("updates-restart-note")} value={null} />
      </div>
    );
  else if (phase === "error" && error)
    section = (
      <div className="w-0 min-w-full" key="error">
        <Alert color="error" role="alert">
          {error}
        </Alert>
      </div>
    );

  return (
    <Island
      aria-label={t("updates-island")}
      isShown={isShown}
      notch={notch}
      onHidden={onHidden}
      onShapeChange={onShapeChange}
      row={
        <IslandRow
          end={
            <IslandParts direction="row">
              {end ? (
                <span className="flex" key={phase}>
                  {end}
                </span>
              ) : null}
            </IslandParts>
          }
          start={
            <IslandParts direction="row">
              <IslandRowTitle
                glyph={
                  phase === "error" ? (
                    <AlertCircle aria-hidden className="text-error" />
                  ) : (
                    <CircleArrowDown aria-hidden />
                  )
                }
                key={phase}
                note={
                  phase === "available" && released
                    ? t("updates-released", {
                        date: dateFormatter.format(released),
                      })
                    : undefined
                }
                title={<span role="status">{titles[phase]}</span>}
              />
            </IslandParts>
          }
        />
      }
      section={section ? <IslandParts>{section}</IslandParts> : undefined}
    />
  );
}
