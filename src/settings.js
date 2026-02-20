const { invoke } = window.__TAURI__.core;
/**
 * Settings App Class - Manages application settings UI
 * Handles loading and saving settings via Tauri commands
 */
class SettingsApp {
    newChatDefault;
    notificationsEnabled;
    constructor() {
        this.newChatDefault = document.getElementById('new-chat-default');
        this.notificationsEnabled = document.getElementById('notifications-enabled');
        this.initEventListeners();
        this.loadSettings();
    }
    /**
     * Initialize event listeners for settings changes
     */
    initEventListeners() {
        if (this.newChatDefault) {
            this.newChatDefault.addEventListener('change', () => this.saveSettings());
        }
        if (this.notificationsEnabled) {
            this.notificationsEnabled.addEventListener('change', () => this.saveSettings());
        }
    }
    /**
     * Load settings from backend and update UI
     */
    async loadSettings() {
        try {
            const settings = await invoke('get_settings');
            if (this.newChatDefault) {
                this.newChatDefault.checked = settings.new_chat_default ?? true;
            }
            if (this.notificationsEnabled) {
                this.notificationsEnabled.checked = settings.notifications_enabled ?? true;
            }
        }
        catch (error) {
            console.error('Failed to load settings:', error);
        }
    }
    /**
     * Save current settings to backend
     */
    async saveSettings() {
        const settings = {
            new_chat_default: this.newChatDefault?.checked ?? true,
            notifications_enabled: this.notificationsEnabled?.checked ?? true,
        };
        try {
            await invoke('save_settings', { settings });
        }
        catch (error) {
            console.error('Failed to save settings:', error);
        }
    }
}
// Initialize the settings app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    new SettingsApp();
});
export {};
//# sourceMappingURL=settings.js.map