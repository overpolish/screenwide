// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import {
  presentUpdateIsland,
  updateIslandShown,
  withdrawUpdateIsland,
} from "../api";
import { useUpdateBridge } from "../update-bridge";
import { updateDebug } from "../update-debug";
import {
  remindAboutUpdateLater,
  scheduledUpdateCheckAt,
  skipUpdateVersion,
  startupUpdateCheckDue,
  updateVersionWasSkipped,
} from "../update-preferences";
import { useUpdate } from "../use-update";

import type { UpdateIslandProps } from "./update-island";

/** How often a version found while the island was busy asks for it again. */
const RETRY_MS = 60_000;

const report = (action: string) => (cause: unknown) => {
  console.error(`Could not ${action}`, cause);
};

/**
 * The updater, owned by the island's window, and the offer it makes there.
 * A version found by a check is offered once, never one that was skipped:
 * the island comes out for it when nothing is recorded or exported and it is
 * out for nothing else, asking again until it is free. Nothing downloads
 * until Install. `isUp` while the island offers it; whatever takes the island
 * or puts it away ends the offer.
 */
export function useUpdateIsland(): {
  isUp: boolean;
  props: Omit<
    UpdateIslandProps,
    "isShown" | "notch" | "onHidden" | "onShapeChange"
  >;
} {
  const [checkOnLaunch] = useState(startupUpdateCheckDue);
  const reminderTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const offeredVersionRef = useRef<string | null>(null);
  const [isUp, setIsUp] = useState(false);
  const [isNotesOpen, setIsNotesOpen] = useState(false);
  // A version found while the island was busy, to offer once it is free.
  const [waiting, setWaiting] = useState<string | null>(null);
  const update = useUpdate({ autoCheck: checkOnLaunch });
  const { checkForUpdates, discardUpdate, status, updateVersion } = update;
  const busy = status === "downloading";

  const present = useCallback(async () => {
    const shown = await presentUpdateIsland();
    if (shown) setIsUp(true);
    return shown;
  }, []);

  useUpdateBridge(update, () => {
    setIsNotesOpen(true);
    present().catch(report("show the update on the island"));
  });

  useEffect(() => {
    if (!isTauri()) return;
    void updateIslandShown().then((shown) => {
      if (shown) setIsUp(true);
    });
  }, []);
  useTauriEvent("updates://island", (payload) => {
    setIsUp(payload as boolean);
  });
  // The island going away ends the offer; it tucks away showing it.
  useTauriEvent("island://hide", () => {
    setIsUp(false);
  });

  const scheduleUpdateCheck = useCallback(
    (at: number) => {
      clearTimeout(reminderTimerRef.current ?? undefined);
      reminderTimerRef.current = setTimeout(
        () => {
          reminderTimerRef.current = null;
          updateDebug("Reminder cooldown elapsed; checking again");
          void checkForUpdates();
        },
        Math.max(0, at - Date.now()),
      );
      updateDebug("Scheduled next update check", { at });
    },
    [checkForUpdates],
  );

  useEffect(() => {
    const scheduled = scheduledUpdateCheckAt();
    if (!checkOnLaunch && scheduled) scheduleUpdateCheck(scheduled);
    return () => {
      clearTimeout(reminderTimerRef.current ?? undefined);
    };
  }, [checkOnLaunch, scheduleUpdateCheck]);

  useEffect(() => {
    if (status !== "available" || !updateVersion) return;
    if (updateVersionWasSkipped(updateVersion)) {
      updateDebug("Suppressing the offer of a skipped version", {
        updateVersion,
      });
      void discardUpdate();
      return;
    }
    if (offeredVersionRef.current === updateVersion) return;
    offeredVersionRef.current = updateVersion;
    updateDebug("Offering the update on the island", { updateVersion });
    // The offer starts when the updater announces a new version, an event from
    // outside React, so its state is set here rather than while rendering.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setIsNotesOpen(false);
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setWaiting(updateVersion);
  }, [discardUpdate, status, updateVersion]);

  // Asks for the island now, and again while it is busy, until it comes out
  // or the version is no longer on offer.
  const offer =
    waiting && status === "available" && updateVersion === waiting
      ? waiting
      : null;
  useEffect(() => {
    if (!offer) return;
    let cancelled = false;
    const ask = () => {
      present().then((shown) => {
        if (shown && !cancelled) setWaiting(null);
      }, report("show the update on the island"));
    };
    ask();
    const timer = setInterval(ask, RETRY_MS);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [offer, present]);

  const later = () => {
    if (busy) return;
    updateDebug("Update put off with a reminder");
    offeredVersionRef.current = null;
    scheduleUpdateCheck(remindAboutUpdateLater());
    withdrawUpdateIsland().catch(report("put the update island away"));
  };

  return {
    isUp,
    props: {
      currentVersion: update.currentVersion,
      downloadProgress: update.downloadProgress,
      error: update.error,
      isNotesOpen,
      onInstall: () => void update.installUpdate(),
      onLater: later,
      onNotesOpenChange: setIsNotesOpen,
      onSkip: () => {
        if (!updateVersion || busy) return;
        updateDebug("Update version skipped from the island", {
          updateVersion,
        });
        skipUpdateVersion(updateVersion);
        withdrawUpdateIsland().catch(report("put the update island away"));
        void discardUpdate();
      },
      releaseDate: update.releaseDate,
      releaseNotes: update.releaseNotes,
      status,
      updateVersion,
    },
  };
}
