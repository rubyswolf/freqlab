# Notification When Claude Finishes

## Overview

Add native macOS notifications when Claude finishes working on a project, but only when the freqlab window is not focused. Users can toggle this on/off in Settings > General, and are prompted to enable during the setup wizard.

## Requirements

- Native macOS notification via Tauri notification plugin
- Only trigger when app window is not focused (`document.hidden`)
- Simple message: "Claude finished working on {project name}"
- Toggle setting in General settings tab
- Permission request during setup wizard (opt-in)

## Implementation

### Dependencies

Add `tauri-plugin-notification = "2"` to `src-tauri/Cargo.toml`

### Files Modified

| File | Change |
|------|--------|
| `src-tauri/Cargo.toml` | Add notification plugin dependency |
| `src-tauri/src/lib.rs` | Register notification plugin |
| `src/stores/settingsStore.ts` | Add `setShowNotifications` setter |
| `src/components/Setup/WelcomeWizard.tsx` | Add notification opt-in toggle |
| `src/components/Settings/ThemePicker.tsx` | Add notification toggle |
| `src/components/Chat/ChatPanel.tsx` | Send notification on completion |

### Notification Logic

```typescript
// In ChatPanel.tsx finally block after clearClaudeBusy
if (showNotifications && document.hidden) {
  sendNotification({
    title: 'freqlab',
    body: `Claude finished working on ${project.name}`
  });
}
```

### Permission Flow

1. Setup wizard "complete" step shows toggle for notifications
2. When toggled on, request permission via `requestPermission()`
3. If granted, save `showNotifications: true`
4. If denied, show toast explaining they can enable in System Settings
5. Users can also toggle in Settings > General anytime
