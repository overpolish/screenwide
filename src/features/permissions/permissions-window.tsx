// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Disc, Mic, PersonStanding, Video } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useRef } from "react";

import { WindowHeader } from "../../composites/window-header/window-header";
import { WindowShell } from "../../composites/window-shell/window-shell";
import { t } from "../../i18n/i18n";
import { motionDurations, motionEasings } from "../../lib/motion";
import { useFitWindowHeight } from "../../lib/use-fit-window-height";
import { Button } from "../../primitives/button/button";

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

type PermissionsWindowProps = {
  onClose?: () => void;
  onGrant?: (permission: PermissionKind, status: PermissionStatus) => void;
  onRestart?: () => void;
  /** The state shown; the live one when left out. */
  permissions?: PermissionSnapshot;
};

// Required first, then what only some recordings need.
const rows = () => [
  {
    description: t("permissions-accessibility-description"),
    icon: <PersonStanding />,
    kind: "accessibility" as const,
    title: t("permissions-accessibility"),
  },
  {
    description: t("permissions-screen-recording-description"),
    icon: <Disc />,
    kind: "screenRecording" as const,
    title: t("permissions-screen-recording"),
  },
  {
    description: t("permissions-camera-description"),
    icon: <Video />,
    isOptional: true,
    kind: "camera" as const,
    title: t("permissions-camera"),
  },
  {
    description: t("permissions-microphone-description"),
    icon: <Mic />,
    isOptional: true,
    kind: "microphone" as const,
    title: t("permissions-microphone"),
  },
];

const grantPermission = (
  permission: PermissionKind,
  status: PermissionStatus,
) => {
  void (status.canRequest
    ? requestPermission(permission)
    : openPermissionSettings(permission));
};

const permissionsPreviewEnabled =
  import.meta.env.DEV &&
  import.meta.env.VITE_SCREENWIDE_PERMISSIONS_PREVIEW === "1";

export function PermissionsWindow({
  onClose = () => void dismissPermissionsWindow(),
  onGrant = grantPermission,
  onRestart = () => void restartApp(),
  permissions: providedPermissions,
}: PermissionsWindowProps) {
  const livePermissions = usePermissionStore((state) => state.permissions);
  const permissions =
    providedPermissions ??
    (permissionsPreviewEnabled ? permissionsPreviewSnapshot : livePermissions);
  // Screen Recording only takes effect in a new process, so once both
  // required permissions are in, restarting is what the window is for.
  const hasRequired =
    permissions.accessibility.granted && permissions.screenRecording.granted;
  // Fitted to the list like a native panel, rather than leaving space under it.
  const contentRef = useRef<HTMLElement>(null);
  useFitWindowHeight(contentRef);

  return (
    <WindowShell
      header={
        <WindowHeader
          actions={
            <AnimatePresence>
              {hasRequired ? (
                <motion.div
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  initial={{ opacity: 0 }}
                  transition={{
                    duration: motionDurations.state,
                    ease: motionEasings.out,
                  }}
                >
                  <Button color="primary" onPress={onRestart}>
                    {t("permissions-restart")}
                  </Button>
                </motion.div>
              ) : null}
            </AnimatePresence>
          }
          onClose={onClose}
          title={t("permissions-title")}
        />
      }
      height="content"
      ref={contentRef}
    >
      <div className="flex flex-col gap-separate px-inset pb-inset">
        {rows().map(({ description, icon, isOptional, kind, title }) => (
          <PermissionRow
            description={description}
            icon={icon}
            isOptional={isOptional}
            key={kind}
            onGrant={onGrant}
            permission={kind}
            status={permissions[kind]}
            title={title}
          />
        ))}
      </div>
    </WindowShell>
  );
}
