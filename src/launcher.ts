// Kimi Launcher TypeScript
import type { AppSettings, InjectResult, SettingsChangedEvent } from './types';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

/**
 * Launcher App Class - Encapsulates all launcher functionality
 * Provides a quick input interface for sending messages to Kimi
 */
class LauncherApp {
  // DOM elements
  private readonly input: HTMLInputElement | null;
  private readonly submitBtn: HTMLButtonElement | null;
  private readonly newChatToggle: HTMLElement | null;
  private readonly botModeToggle: HTMLElement | null;
  
  // State
  private focusTimeout: number | null = null;
  private isSubmitting = false;
  private newChatMode = true; // Default: start new conversations
  private botMode = false; // Default: normal chat mode
  
  // Constants
  private readonly MAX_MESSAGE_LENGTH = 5000;
  
  constructor() {
    // DOM elements
    this.input = document.getElementById('launcher-input') as HTMLInputElement | null;
    this.submitBtn = document.getElementById('submit-btn') as HTMLButtonElement | null;
    this.newChatToggle = document.getElementById('new-chat-toggle');
    this.botModeToggle = document.getElementById('bot-mode-toggle');
    
    // Validate elements
    if (!this.input || !this.submitBtn) {
      console.error('Critical UI elements not found. Launcher cannot initialize.');
      return;
    }
    
    // Set platform-aware modifier key labels
    this.initModifierKeys();
    
    // Initialize event listeners
    this.initEventListeners();
    this.initTauriListeners();
  }
  
  /**
   * Initialize platform-aware modifier key labels (⌘ on Mac, Ctrl on Windows/Linux)
   */
  private initModifierKeys(): void {
    const isMac = navigator.platform?.toUpperCase().includes('MAC')
      || navigator.userAgent?.includes('Mac');
    const modLabel = isMac ? '⌘' : 'Ctrl';
    
    // Update all .mod-key elements
    document.querySelectorAll('.mod-key').forEach(el => {
      el.textContent = modLabel;
    });
    
    // Update the new-chat toggle tooltip
    if (this.newChatToggle) {
      this.newChatToggle.title = `Start a new conversation (${modLabel}+K)`;
    }
    
    // Update the bot-mode toggle tooltip
    if (this.botModeToggle) {
      this.botModeToggle.title = `Open in Kimi Claw (${modLabel}+B)`;
    }
  }
  
  /**
   * Initialize DOM event listeners
   */
  private initEventListeners(): void {
    // Focus input on load
    window.addEventListener('DOMContentLoaded', () => this.focusInput());
    
    // Keyboard events
    document.addEventListener('keydown', (e) => this.handleKeyDown(e), { passive: false });
    
    // Submit button
    this.submitBtn?.addEventListener('click', () => this.submitMessage());
    
    // New chat toggle
    if (this.newChatToggle) {
      this.newChatToggle.addEventListener('click', () => this.toggleNewChat());
    }
    
    // Bot mode toggle
    if (this.botModeToggle) {
      this.botModeToggle.addEventListener('click', () => this.toggleBotMode());
    }
    
    // Window focus
    window.addEventListener('focus', () => this.handleWindowFocus());
    
    // Cleanup
    window.addEventListener('beforeunload', () => this.cleanup());
  }
  
  /**
   * Initialize Tauri event listeners
   */
  private initTauriListeners(): void {
    // Load new-chat default from settings
    this.loadNewChatDefault();
    
    // Listen for launcher-shown event from Rust to clear and focus input
    listen('launcher-shown', () => {
      if (this.input) {
        this.input.value = '';
        this.input.focus();
      }
      // Re-load setting in case it was changed
      this.loadNewChatDefault();
    }).catch((error: Error) => {
      console.error('Failed to listen for launcher-shown event:', error);
    });
    
    // Listen for settings-changed event
    listen<SettingsChangedEvent>('settings-changed', (event) => {
      const settings = event.payload;
      if (settings && typeof settings.new_chat_default === 'boolean') {
        this.newChatMode = settings.new_chat_default;
        if (this.newChatToggle) {
          this.newChatToggle.classList.toggle('active', this.newChatMode);
        }
        if (this.input) {
          this.input.placeholder = this.newChatMode
            ? 'Ask Kimi anything...'
            : 'Continue current chat...';
        }
      }
    }).catch((error: Error) => {
      console.error('Failed to listen for settings-changed event:', error);
    });
    
    // Listen for inject-result from the main window's injected JS
    listen<InjectResult>('inject-result', (event) => {
      const { success, error } = event.payload ?? {};
      if (!success && error) {
        console.error('[Kimi] Message injection failed:', error);
        this.showError(error);
      }
    }).catch((error: Error) => {
      console.error('Failed to listen for inject-result event:', error);
    });
  }
  
  /**
   * Show error state in the launcher
   */
  private async showError(_errorMessage: string): Promise<void> {
    // Briefly re-show the launcher with an error state
    try {
      await invoke('show_launcher');
    } catch (e) {
      // If we can't show the launcher, just log it
      console.error('Failed to show launcher for error display:', e);
      return;
    }
    
    const container = document.querySelector('.launcher-container');
    if (!container) return;
    
    // Show error in the input placeholder
    if (this.input) {
      this.input.value = '';
      this.input.placeholder = 'Failed to send — try again';
      this.input.focus();
    }
    
    // Add error class for visual feedback
    container.classList.add('launcher-error');
    
    // Remove error state after animation completes
    setTimeout(() => {
      container.classList.remove('launcher-error');
      this.updatePlaceholder();
    }, 2500);
  }

  /**
   * Load new chat default setting from backend
   */
  private async loadNewChatDefault(): Promise<void> {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      this.newChatMode = settings.new_chat_default ?? true;
      if (this.newChatToggle) {
        this.newChatToggle.classList.toggle('active', this.newChatMode);
      }
      this.updatePlaceholder();
    } catch (error) {
      // Settings not available yet, use default
      console.warn('Failed to load settings, using defaults:', error);
    }
  }
  
  /**
   * Focus the input element
   */
  private focusInput(): void {
    this.input?.focus();
  }
  
  /**
   * Handle keyboard events
   * @param e - Keyboard event
   */
  private handleKeyDown(e: KeyboardEvent): void {
    try {
      // Escape to hide launcher
      if (e.key === 'Escape') {
        e.preventDefault();
        invoke('hide_launcher').catch((error: Error) => {
          console.error('Failed to hide launcher:', error);
        });
        return;
      }
      
      // Cmd/Ctrl+K to toggle new chat mode
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.toggleNewChat();
        return;
      }
      
      // Cmd/Ctrl+B to toggle bot mode
      if (e.key === 'b' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.toggleBotMode();
        return;
      }
      
      // Enter to submit
      if (e.key === 'Enter' && !e.shiftKey && this.input) {
        e.preventDefault();
        this.submitMessage();
        return;
      }
    } catch (error) {
      console.error('Error handling keyboard event:', error);
    }
  }
  
  /**
   * Handle window focus event
   */
  private handleWindowFocus(): void {
    if (this.focusTimeout) {
      window.clearTimeout(this.focusTimeout);
    }
    
    this.focusTimeout = window.setTimeout(() => {
      if (this.input) {
        this.input.focus();
        // Only select text if user hasn't already modified it
        if (this.input.value && document.activeElement === this.input) {
          this.input.select();
        }
      }
      this.focusTimeout = null;
    }, 100);
  }
  
  /**
   * Cleanup resources before unload
   */
  private cleanup(): void {
    if (this.focusTimeout) {
      window.clearTimeout(this.focusTimeout);
    }
  }
  
  /**
   * Toggle new chat mode
   */
  private toggleNewChat(): void {
    this.newChatMode = !this.newChatMode;
    if (this.newChatToggle) {
      this.newChatToggle.classList.toggle('active', this.newChatMode);
    }
    // Update placeholder to reflect mode
    this.updatePlaceholder();
  }
  
  /**
   * Toggle bot mode
   */
  private toggleBotMode(): void {
    this.botMode = !this.botMode;
    if (this.botModeToggle) {
      this.botModeToggle.classList.toggle('active', this.botMode);
    }
    // When bot mode is enabled, also enable new chat mode
    if (this.botMode && !this.newChatMode) {
      this.newChatMode = true;
      this.newChatToggle?.classList.add('active');
    }
    // Update placeholder to reflect mode
    this.updatePlaceholder();
  }
  
  /**
   * Update input placeholder based on current mode
   */
  private updatePlaceholder(): void {
    if (!this.input) return;
    
    if (this.botMode) {
      this.input.placeholder = 'Ask Kimi Claw...';
    } else if (this.newChatMode) {
      this.input.placeholder = 'Ask Kimi anything...';
    } else {
      this.input.placeholder = 'Continue current chat...';
    }
  }
  
  /**
   * Submit message to backend
   */
  private async submitMessage(): Promise<void> {
    // Prevent multiple submissions
    if (this.isSubmitting) {
      return;
    }
    
    this.isSubmitting = true;
    
    // Store original message outside try block so catch can access it
    let originalMessage = '';
    
    try {
      // Validate input element exists and has value
      if (!this.input) {
        console.error('Input element not found');
        return;
      }
      
      const message = this.input.value.trim();
      
      // Validate message content
      if (!message) {
        return;
      }
      
      // Additional validation: limit message length
      if (message.length > this.MAX_MESSAGE_LENGTH) {
        console.error(`Message too long (${message.length}/${this.MAX_MESSAGE_LENGTH})`);
        return;
      }
      
      // Store for restoration on error
      originalMessage = message;
      
      // Show submitting state
      this.submitBtn?.classList.add('launcher-submitting');
      if (this.submitBtn) {
        this.submitBtn.disabled = true;
      }
      
      // Clear input only after successful validation
      this.input.value = '';
      
      // Send message to Rust backend with timeout
      const args = {
        message,
        newChat: this.newChatMode,
        botMode: this.botMode
      };
      const submitPromise = invoke('submit_message', args as Record<string, unknown>);
      
      // Add timeout to prevent hanging
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error('Submit message timeout')), 10000);
      });
      
      await Promise.race([submitPromise, timeoutPromise]);
      
    } catch (error) {
      console.error('Failed to submit message:', error);
      
      // Restore message on error
      if (this.input && originalMessage) {
        this.input.value = originalMessage;
      }
      
      if (error instanceof Error && error.message === 'Submit message timeout') {
        console.error('Message submission timed out');
      }
    } finally {
      this.isSubmitting = false;
      this.submitBtn?.classList.remove('launcher-submitting');
      if (this.submitBtn) {
        this.submitBtn.disabled = false;
      }
    }
  }
}

// Initialize the launcher app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  new LauncherApp();
});
