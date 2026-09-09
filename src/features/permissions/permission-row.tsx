// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { ReactNode } from "react";
import { TooltipTrigger } from "react-aria-components";

import { Badge } from "../../components/base/badge/badge";
import { Button } from "../../components/base/button/button";
import { Tooltip } from "../../components/base/tooltip/tooltip";
import { Setting } from "../../components/shared/setting/setting";
import { cn } from "../../lib/styling";

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
    <Setting
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
      titleAccessory={isOptional ? <Badge>Optional</Badge> : undefined}
    >
      {(controlProps) =>
        status.granted ? (
          <div className="flex h-control-height shrink-0 items-center">
            <Check className="size-icon text-success" />
          </div>
        ) : (
          <TooltipTrigger isDisabled={status.canRequest}>
            <Button {...controlProps} onPress={grant}>
              {status.canRequest ? "Grant" : "Open System Settings"}
            </Button>
            <Tooltip>Enable manually</Tooltip>
          </TooltipTrigger>
        )
      }
    </Setting>
  );
}
