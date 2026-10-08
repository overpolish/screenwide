// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { RecognizedQrCode } from "../../bindings/RecognizedQrCode";
export type { RecognizedQrCode };

export const cancelTextRecognition = () =>
  invoke<null>("cancel_text_recognition");

export const startTextRecognition = () =>
  invoke<null>("start_text_recognition");

export const copyRecognitionContent = (text: string) =>
  invoke<null>("copy_recognition_content", { text });

export const getQrDetails = () => invoke<RecognizedQrCode>("get_qr_details");

export const closeQrDetails = () => invoke<null>("close_qr_details");
