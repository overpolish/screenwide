// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getVersion } from "@tauri-apps/api/app";
import { emitTo, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";

import {
  UPDATE_READY,
  UPDATE_REQUEST,
  UPDATE_STATE,
  type UpdateRequest,
  type UpdateSnapshot,
} from "../updates/update-bridge";

export function useSettingsUpdate() {
  const [snapshot, setSnapshot] = useState<UpdateSnapshot>({
    currentVersion: null,
    error: null,
    status: "idle",
    updateVersion: null,
  });
  const request = (payload: UpdateRequest = {}) => {
    void emitTo("update", UPDATE_REQUEST, payload).catch((reason: unknown) => {
      setSnapshot((current) => ({
        ...current,
        error: String(reason),
        status: "error",
      }));
    });
  };
  useEffect(() => {
    let disposed = false;
    const cleanups: (() => void)[] = [];
    const window = getCurrentWindow();
    const onVisible = () => {
      void window
        .isVisible()
        .then((visible) => {
          if (visible && !disposed)
            void emitTo("update", UPDATE_REQUEST, {}).catch(() => undefined);
        })
        .catch(() => undefined);
    };
    void getVersion()
      .then((currentVersion) => {
        if (!disposed)
          setSnapshot((current) => ({ ...current, currentVersion }));
      })
      .catch(() => undefined);
    const attach = async () => {
      for (const subscribe of [
        () =>
          listen<UpdateSnapshot>(UPDATE_STATE, ({ payload }) => {
            if (!disposed) setSnapshot(payload);
          }),
        () => listen(UPDATE_READY, onVisible),
        () =>
          window.onFocusChanged(({ payload }) => {
            if (payload) onVisible();
          }),
      ]) {
        const off = await subscribe();
        if (disposed) off();
        else cleanups.push(off);
      }
      onVisible();
    };
    void attach().catch((reason: unknown) => {
      if (!disposed)
        setSnapshot((current) => ({
          ...current,
          error: String(reason),
          status: "error",
        }));
    });
    return () => {
      disposed = true;
      cleanups.forEach((off) => {
        off();
      });
    };
  }, []);
  return {
    ...snapshot,
    onPress: () => {
      request({
        force: true,
        open:
          snapshot.status === "available" || snapshot.status === "downloading",
      });
    },
  };
}
