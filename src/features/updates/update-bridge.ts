// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { emitTo, listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { showUpdatePrompt } from "./api";

import type { useUpdate, UpdateStatus } from "./use-update";

export const UPDATE_REQUEST = "screenwide:update-request";
export const UPDATE_STATE = "screenwide:update-state";
export const UPDATE_READY = "screenwide:update-ready";
export type UpdateSnapshot = {
  currentVersion: string | null;
  error: string | null;
  status: UpdateStatus;
  updateVersion: string | null;
};
export type UpdateRequest = { force?: boolean; open?: boolean };

export function shouldRequestUpdateCheck({
  checkedAt,
  force = false,
  now = Date.now(),
  pending,
  status,
}: {
  checkedAt: number;
  pending: boolean;
  status: UpdateStatus;
  force?: boolean;
  now?: number;
}) {
  return (
    !pending &&
    status !== "checking" &&
    status !== "downloading" &&
    (force || now - checkedAt >= 60_000)
  );
}

// The update window owns plugin resources and installation. Other windows only
// request checks and receive serializable status, never run a second updater.
export function useUpdateBridge(update: ReturnType<typeof useUpdate>) {
  const latestRef = useRef(update);
  const checkedAtRef = useRef(0);
  const requestedCheckRef = useRef(false);

  useEffect(() => {
    latestRef.current = update;
    if (["up-to-date", "available", "development"].includes(update.status))
      checkedAtRef.current = Date.now();
    const snapshot: UpdateSnapshot = {
      currentVersion: update.currentVersion,
      error: update.error,
      status: update.status,
      updateVersion: update.updateVersion,
    };
    void emitTo("settings", UPDATE_STATE, snapshot).catch(() => undefined);
  }, [update]);

  useEffect(() => {
    let disposed = false;
    let off: (() => void) | undefined;
    const subscription = listen<UpdateRequest>(
      UPDATE_REQUEST,
      ({ payload }) => {
        const state = latestRef.current;
        const publish = () =>
          emitTo("settings", UPDATE_STATE, {
            currentVersion: state.currentVersion,
            error: state.error,
            status: state.status,
            updateVersion: state.updateVersion,
          } satisfies UpdateSnapshot);
        void publish().catch(() => undefined);
        if (
          payload.open &&
          (state.status === "available" || state.status === "downloading")
        ) {
          void showUpdatePrompt().catch(() => undefined);
          return;
        }
        if (
          !shouldRequestUpdateCheck({
            checkedAt: checkedAtRef.current,
            force: payload.force,
            pending: requestedCheckRef.current,
            status: state.status,
          })
        )
          return;
        requestedCheckRef.current = true;
        void state.checkForUpdates().finally(() => {
          requestedCheckRef.current = false;
        });
      },
    );
    void subscription
      .then((unsubscribe) => {
        if (disposed) unsubscribe();
        else {
          off = unsubscribe;
          void emitTo("settings", UPDATE_READY).catch(() => undefined);
        }
      })
      .catch((reason: unknown) => {
        console.error("Could not register update requests", reason);
      });
    return () => {
      disposed = true;
      off?.();
    };
  }, []);
}
