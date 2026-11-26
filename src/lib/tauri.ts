import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AudioDeviceInfo,
  BootstrapPayload,
  PermissionStatus,
  SleepStatus,
  SwitchPack,
  TypingStats,
} from "../types";

export const api = {
  getBootstrap: () => invoke<BootstrapPayload>("get_bootstrap"),
  updateConfig: (newConfig: AppConfig) =>
    invoke<AppConfig>("update_config", { newConfig }),
  toggleEnabled: () => invoke<boolean>("toggle_enabled"),
  previewSwitch: (packId: string) => invoke<void>("preview_switch", { packId }),
  previewSprite: (spriteId: string, volume?: number) =>
    invoke<void>("preview_sprite", { spriteId, volume }),
  saveCustomPack: (pack: SwitchPack) =>
    invoke<SwitchPack[]>("save_custom_pack", { pack }),
  deleteCustomPack: (packId: string) =>
    invoke<SwitchPack[]>("delete_custom_pack", { packId }),
  listAudioDevices: () => invoke<AudioDeviceInfo[]>("list_audio_devices"),
  getStats: () => invoke<TypingStats>("get_stats"),
  resetStats: () => invoke<TypingStats>("reset_stats"),
  getSleepStatus: () => invoke<SleepStatus>("get_sleep_status"),
  getPermissions: () => invoke<PermissionStatus>("get_permissions"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
};
