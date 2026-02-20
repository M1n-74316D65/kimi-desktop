/// <reference types="@tauri-apps/api" />

// Type definitions for Kimi Desktop App

declare global {
  interface Window {
    __TAURI__: {
      core: {
        invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      };
      event: {
        emit: (event: string, payload?: unknown) => void;
        listen: <T = unknown>(event: string, handler: (event: { payload: T }) => void) => Promise<() => void>;
      };
    };
  }
}

// Tauri Command Types
export interface AppSettings {
  new_chat_default: boolean;
  notifications_enabled: boolean;
}

export interface SubmitMessageArgs {
  message: string;
  newChat: boolean;
  botMode: boolean;
}

export interface InjectResult {
  success: boolean;
  error?: string | null;
}

// Launcher App Types
export interface LauncherElements {
  input: HTMLInputElement | null;
  submitBtn: HTMLButtonElement | null;
  newChatToggle: HTMLElement | null;
  botModeToggle: HTMLElement | null;
}

export interface LauncherState {
  focusTimeout: number | null;
  isSubmitting: boolean;
  newChatMode: boolean;
  botMode: boolean;
}

// Settings App Types  
export interface SettingsElements {
  newChatDefault: HTMLInputElement | null;
  notificationsEnabled: HTMLInputElement | null;
}

// Event Payload Types
export interface SettingsChangedEvent {
  new_chat_default: boolean;
  notifications_enabled: boolean;
}

export type LauncherShownEvent = void;
export type ResponseCompleteEvent = void;

export {};
