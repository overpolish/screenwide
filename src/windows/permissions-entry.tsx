// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PermissionSync } from "../features/permissions/permission-sync";
import { PermissionsWindow } from "../features/permissions/permissions-window";

/** The window, with the stores it reads kept in step with Rust and the other windows. */
export function PermissionsEntry() {
  return (
    <>
      <PermissionSync />
      <PermissionsWindow />
    </>
  );
}
