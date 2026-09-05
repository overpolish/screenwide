// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Disc, Mic, PersonStanding, Video } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";

import logoUrl from "../../assets/screenwide-mark.svg";
import { Button } from "../../components/base/button/button";
import { WindowHeader } from "../../components/shared/window-header/window-header";

import {
  dismissPermissionsWindow,
  openPermissionSettings,
  requestPermission,
  restartApp,
} from "./api";
import { PermissionRow } from "./permission-row";
import { permissionsPreviewSnapshot } from "./permissions-preview";
import { usePermissionStore } from "./store";
import { PermissionKind, PermissionSnapshot, PermissionStatus } from "./types";

const ICON_SIZE = 40;
const gradients = {
  blue: "bg-linear-0 from-[#3B83F7] from-20% to-[#5DA3F8]",
  gray: "bg-linear-0 from-[#98989D] from-20% to-[#C0C0C4]",
  red: "bg-linear-0 from-[#EB5545] from-20% to-[#EE8176]",
};

type PermissionsWindowProps = {
  onClose?: () => void;
  onGrant?: (permission: PermissionKind, status: PermissionStatus) => void;
  onRestart?: () => void;
  permissions?: PermissionSnapshot;
};

const grantPermission = (
  permission: PermissionKind,
  status: PermissionStatus,
) => {
  const action = status.canRequest
    ? requestPermission(permission)
    : openPermissionSettings(permission);
  void action;
};

const permissionsPreviewEnabled =
  import.meta.env.DEV &&
  import.meta.env.VITE_SCREENWIDE_PERMISSIONS_PREVIEW === "1";

export function PermissionsWindow({
  onClose = () => void dismissPermissionsWindow(),
  onGrant = grantPermission,
  onRestart = () => void restartApp(),
  permissions: providedPermissions,
}: PermissionsWindowProps = {}) {
  const livePermissions = usePermissionStore((state) => state.permissions);
  const permissions =
    providedPermissions ??
    (permissionsPreviewEnabled ? permissionsPreviewSnapshot : livePermissions);
  const hasRequired =
    permissions.accessibility.granted && permissions.screenRecording.granted;

  return (
    <main className="window-surface gap-section flex h-full flex-col overflow-hidden">
      <WindowHeader
        actions={
          <AnimatePresence>
            {hasRequired ? (
              <motion.div
                animate={{ opacity: 1, scale: 1 }}
                className="flex"
                exit={{ opacity: 0 }}
                initial={{ opacity: 0, scale: 0 }}
              >
                <Button color="primary" onPress={onRestart} size="compact">
                  Restart Screenwide
                </Button>
              </motion.div>
            ) : null}
          </AnimatePresence>
        }
        leadingSection={
          <img
            alt="Screenwide"
            className="brightness-0 dark:invert"
            draggable={false}
            src={logoUrl}
          />
        }
        onClose={onClose}
        title="Permissions"
      />

      <div className="gap-section px-window-inset pb-window-inset flex flex-col">
        <PermissionRow
          color={gradients.blue}
          description="For capturing cursor events."
          icon={<PersonStanding size={ICON_SIZE} />}
          onGrant={onGrant}
          permission="accessibility"
          status={permissions.accessibility}
          title="Accessibility"
        />
        <PermissionRow
          color={gradients.red}
          icon={<Disc size={ICON_SIZE} />}
          onGrant={onGrant}
          permission="screenRecording"
          status={permissions.screenRecording}
          title="Screen Recording"
        />
        <PermissionRow
          color={gradients.gray}
          icon={<Video size={ICON_SIZE} />}
          isOptional
          onGrant={onGrant}
          permission="camera"
          status={permissions.camera}
          title="Camera"
        />
        <PermissionRow
          color={gradients.gray}
          icon={<Mic size={ICON_SIZE} />}
          isOptional
          onGrant={onGrant}
          permission="microphone"
          status={permissions.microphone}
          title="Microphone"
        />
      </div>
    </main>
  );
}
