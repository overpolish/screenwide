// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { ReactNode } from "react";
import { TooltipTrigger } from "react-aria-components";
import { twMerge } from "tailwind-merge";

import { Badge } from "../../components/base/badge/badge";
import { Button } from "../../components/base/button/button";
import { Tooltip } from "../../components/base/tooltip/tooltip";

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
    <div className="gap-section flex items-center">
      <div
        className={twMerge(
          "flex size-16 items-center justify-center rounded-2xl text-white",
          color,
        )}
      >
        {icon}
      </div>
      <div className="flex grow flex-col text-content-fg">
        <div className="gap-control flex items-center">
          <span className="font-semibold">{title}</span>
          {isOptional ? <Badge>Optional</Badge> : null}
        </div>
        {description ? (
          <span className="text-sm text-muted">{description}</span>
        ) : null}
      </div>

      {status.granted ? (
        <div className="flex w-[62px] justify-center">
          <Check className="text-success" size={32} />
        </div>
      ) : (
        <TooltipTrigger isDisabled={status.canRequest}>
          <Button onPress={grant}>
            {status.canRequest ? "Grant" : "Open System Settings"}
          </Button>
          <Tooltip>Enable manually</Tooltip>
        </TooltipTrigger>
      )}
    </div>
  );
}
