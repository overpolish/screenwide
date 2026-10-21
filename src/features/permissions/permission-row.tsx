// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { ReactNode } from "react";

import { ControlRow } from "../../composites/control-row/control-row";
import { NativeTooltipTrigger } from "../../composites/native-tooltip/native-tooltip-trigger";
import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { Badge } from "../../primitives/badge/badge";
import { Button } from "../../primitives/button/button";

import { PermissionKind, PermissionStatus } from "./types";

type PermissionRowProps = {
  color: string;
  icon: ReactNode;
  onGrant: (permission: PermissionKind, status: PermissionStatus) => void;
  permission: PermissionKind;
  status: PermissionStatus;
  title: string;
  description?: string;
  isOptional?: boolean;
};

export function PermissionRow({
  color,
  description,
  icon,
  isOptional,
  onGrant,
  permission,
  status,
  title,
}: PermissionRowProps) {
  const grant = () => {
    onGrant(permission, status);
  };

  return (
    <ControlRow
      description={description}
      leading={
        <div
          className={cn(
            "flex size-8 items-center justify-center rounded-control text-white [&_svg]:size-icon",
            color,
          )}
        >
          {icon}
        </div>
      }
      title={title}
      titleAccessory={
        isOptional ? <Badge>{t("permissions-optional")}</Badge> : undefined
      }
    >
      {(controlProps) =>
        status.granted ? (
          <div className="flex h-control-height shrink-0 items-center">
            <Check className="size-icon text-success" />
          </div>
        ) : (
          <NativeTooltipTrigger
            isDisabled={status.canRequest}
            tooltip={t("permissions-enable-manually")}
          >
            <Button {...controlProps} onPress={grant}>
              {status.canRequest
                ? t("permissions-grant")
                : t("permissions-open-settings")}
            </Button>
          </NativeTooltipTrigger>
        )
      }
    </ControlRow>
  );
}
