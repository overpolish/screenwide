// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { PermissionKind, PermissionSnapshot } from "./types";

export const getPermissionSnapshot = () =>
  invoke<PermissionSnapshot>("permission_snapshot");

export const requestPermission = (permission: PermissionKind) =>
  invoke<null>("request_permission", { permission }).then(() => undefined);

export const openPermissionSettings = (permission: PermissionKind) =>
  invoke<null>("open_permission_settings", { permission }).then(
    () => undefined,
  );

export const restartApp = () =>
  invoke<null>("restart_app").then(() => undefined);

export const dismissPermissionsWindow = () =>
  invoke<null>("dismiss_permissions_window").then(() => undefined);

export const openPermissionsWindow = () =>
  invoke<null>("open_permissions_window").then(() => undefined);
