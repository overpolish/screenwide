// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, Update } from "@tauri-apps/plugin-updater";
import { useCallback, useEffect, useRef, useState } from "react";

import { fetchReleaseNotesHtml } from "./github-release";
import { updateDebug } from "./update-debug";

export type UpdateStatus =
  | "available"
  | "checking"
  | "development"
  | "downloading"
  | "error"
  | "idle"
  | "up-to-date";

/** Where the build's updates come from (`updates::update_source`): its own
 * updater, the Microsoft Store for the Store package, which the app checks
 * but leaves the Store to install, or nowhere for a development build. */
type UpdateSource = "development" | "store" | "updater";

/** An update the Microsoft Store has (`updates::store_update`), with its
 * version when the Store's catalog said which. */
type StoreUpdate = { version: string | null };

const errorMessage = (reason: unknown) =>
  reason instanceof Error ? reason.message : String(reason);

export function useUpdate({ autoCheck = true } = {}) {
  const checkRequestRef = useRef(0);
  // The request a check is running for. A check whose request was superseded,
  // by unmounting or a discard, no longer blocks a new one: its answer is
  // dropped anyway. Under StrictMode's remount it would otherwise hold off the
  // remounted check for good, leaving the status at "checking".
  const checkInFlightRef = useRef<number | null>(null);
  const sourceRef = useRef<UpdateSource | null>(null);
  const updateRef = useRef<Update | null>(null);
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<number | null>(null);
  const [installsFromStore, setInstallsFromStore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [releaseDate, setReleaseDate] = useState<string | null>(null);
  const [releaseNotes, setReleaseNotes] = useState<string | null>(null);
  const [status, setStatus] = useState<UpdateStatus>(
    autoCheck ? "checking" : "idle",
  );
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);

  const checkForUpdates = useCallback(async () => {
    if (checkInFlightRef.current === checkRequestRef.current) return;
    const request = ++checkRequestRef.current;
    checkInFlightRef.current = request;
    updateDebug("Starting update check", { request });
    setError(null);
    setStatus("checking");
    try {
      const [version, source] = await Promise.all([
        getVersion(),
        invoke<UpdateSource>("update_source"),
      ]);
      if (request !== checkRequestRef.current) return;
      setCurrentVersion(version);

      sourceRef.current = source;
      setInstallsFromStore(source === "store");

      if (source !== "updater") {
        const found =
          source === "store"
            ? await invoke<StoreUpdate | null>("store_update")
            : null;
        const foundVersion = found?.version ?? null;
        const notes = foundVersion
          ? await fetchReleaseNotesHtml(foundVersion)
          : null;
        if (request !== checkRequestRef.current) return;
        updateDebug(
          source === "development"
            ? "Skipping updater check"
            : found
              ? "Microsoft Store update available"
              : "No Microsoft Store update available",
          { currentVersion: version, source, updateVersion: foundVersion },
        );
        if (updateRef.current) await updateRef.current.close();
        updateRef.current = null;
        setDownloadProgress(null);
        setReleaseDate(null);
        setReleaseNotes(notes);
        setUpdateVersion(foundVersion);
        setStatus(
          source === "development"
            ? "development"
            : found
              ? "available"
              : "up-to-date",
        );
        return;
      }

      const available = await check({ timeout: 30_000 });
      const notes = available
        ? await fetchReleaseNotesHtml(available.version)
        : null;
      if (request !== checkRequestRef.current) {
        if (available) await available.close();
        return;
      }
      if (updateRef.current) await updateRef.current.close();
      updateRef.current = available;
      setDownloadProgress(null);
      setReleaseDate(available?.date ?? null);
      setReleaseNotes(notes);
      setUpdateVersion(available?.version ?? null);
      setStatus(available ? "available" : "up-to-date");
      updateDebug(
        available ? "Update available" : "No newer update available",
        {
          currentVersion: version,
          updateVersion: available?.version,
        },
      );
    } catch (reason: unknown) {
      if (request !== checkRequestRef.current) return;
      updateDebug("Update check failed", { error: errorMessage(reason) });
      setError(errorMessage(reason));
      setStatus("error");
    } finally {
      if (checkInFlightRef.current === request) checkInFlightRef.current = null;
    }
  }, []);

  const discardUpdate = useCallback(async () => {
    updateDebug("Discarding the pending update");
    checkRequestRef.current += 1;
    const update = updateRef.current;
    updateRef.current = null;
    if (update) await update.close();
    setStatus("idle");
  }, []);

  /** Installs the update and relaunches, or for the Store package opens its
   * page in the Microsoft Store. Resolves true once the Store has it. */
  const installUpdate = useCallback(async () => {
    if (sourceRef.current === "store") {
      setError(null);
      setStatus("available");
      updateDebug("Opening the update in the Microsoft Store");
      try {
        await invoke("open_store_page");
        return true;
      } catch (reason: unknown) {
        updateDebug("The Microsoft Store did not open", {
          error: errorMessage(reason),
        });
        setError(errorMessage(reason));
        setStatus("error");
        return false;
      }
    }

    const update = updateRef.current;
    if (!update) return false;

    setDownloadProgress(null);
    setError(null);
    setStatus("downloading");
    updateDebug("Downloading and installing update", {
      updateVersion: update.version,
    });
    let downloaded = 0;
    let total: number | undefined;
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength;
          updateDebug("Update download started", { contentLength: total });
          setDownloadProgress(total ? 0 : null);
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setDownloadProgress(total ? Math.min(downloaded / total, 1) : null);
        } else {
          setDownloadProgress(1);
          updateDebug("Update download finished");
        }
      });
      updateDebug("Update installed; relaunching Screenwide");
      await relaunch();
    } catch (reason: unknown) {
      updateDebug("Update installation failed", {
        error: errorMessage(reason),
      });
      setError(errorMessage(reason));
      setStatus("error");
    }
    return false;
  }, []);

  useEffect(() => {
    if (autoCheck) void checkForUpdates();
    return () => {
      checkRequestRef.current += 1;
      const update = updateRef.current;
      updateRef.current = null;
      if (update) void update.close();
    };
  }, [autoCheck, checkForUpdates]);

  return {
    checkForUpdates,
    currentVersion,
    discardUpdate,
    downloadProgress,
    error,
    installUpdate,
    installsFromStore,
    releaseDate,
    releaseNotes,
    status,
    updateVersion,
  };
}
