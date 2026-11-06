// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useCallback, useEffect, useState } from "react";

import { useSettingsApi } from "../settings-api-context";

import type { AddKeyOutcome } from "../../../bindings/AddKeyOutcome";
import type { Checkout } from "../../../bindings/Checkout";
import type { Licences } from "../../../bindings/Licences";

export type LicencesControls = {
  add: (key: string) => void;
  /** What came of the last key added, until the field is typed in again. */
  added: AddKeyOutcome | null;
  buy: (checkout: Checkout) => void;
  clearAdded: () => void;
  isAdding: boolean;
  licences: Licences | null;
  remove: (id: string) => void;
};

// The licences as the app holds them, kept in step with changes made
// anywhere: a key added here, the launch's check, the trial starting.
export function useLicences(
  onError: (message: string) => void,
): LicencesControls {
  const api = useSettingsApi();
  const [licences, setLicences] = useState<Licences | null>(null);
  const [added, setAdded] = useState<AddKeyOutcome | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | undefined;
    api
      .getLicences()
      .then((current) => {
        if (!stopped) setLicences(current);
      })
      .catch((reason: unknown) => {
        onError(String(reason));
      });
    api
      .listenToLicences(setLicences)
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch((reason: unknown) => {
        onError(String(reason));
      });
    return () => {
      stopped = true;
      stop?.();
    };
  }, [api, onError]);

  const add = useCallback(
    (key: string) => {
      setIsAdding(true);
      setAdded(null);
      api
        .addLicenceKey(key)
        .then(async (outcome) => {
          setAdded(outcome);
          if (outcome.kind === "added") setLicences(await api.getLicences());
        })
        .catch((reason: unknown) => {
          onError(String(reason));
        })
        .finally(() => {
          setIsAdding(false);
        });
    },
    [api, onError],
  );

  const remove = useCallback(
    (id: string) => {
      api
        .removeLicenceKey(id)
        .then(setLicences)
        .catch((reason: unknown) => {
          onError(String(reason));
        });
    },
    [api, onError],
  );

  const buy = useCallback(
    (checkout: Checkout) => {
      api.openLicenceCheckout(checkout).catch((reason: unknown) => {
        onError(String(reason));
      });
    },
    [api, onError],
  );

  const clearAdded = useCallback(() => {
    setAdded(null);
  }, []);

  return { add, added, buy, clearAdded, isAdding, licences, remove };
}
