# Phase 4: Version Control & Chat History Design

## Overview

Add invisible git-based version control for safety/undo, plus persistent chat history per project.

## Features

1. **Git Safety Net** - Auto-init, auto-commit after Claude edits
2. **Revert to Checkpoint** - Non-destructive revert to any past Claude response
3. **Persistent Chat History** - Save/restore chat per project

---

## 1. Git Safety Net

### On Project Creation

In `create_project`:
- Run `git init` in project folder
- Create `.gitignore`:
  ```
  target/
  Cargo.lock
  ```
- Initial commit: "Initial plugin template"

### After Each Claude Response

Detect if Claude modified files, then:
```bash
git add -A
git commit -m "Claude: {first 50 chars of user prompt}"
```

Store commit hash in chat message metadata for revert functionality.

### Implementation

- Add `git.rs` commands module with:
  - `init_repo(path)` - Initialize git repo
  - `commit_changes(path, message)` - Stage all and commit
  - `get_current_commit(path)` - Get HEAD commit hash
  - `revert_to_commit(path, hash)` - Restore files from commit

---

## 2. Revert to Checkpoint

### UI

Each assistant message with a commit hash shows:
```
[Message content]

↩ Revert to here
```

### Visual States

| State | Appearance |
|-------|------------|
| Normal | Full opacity, revert link visible |
| Dimmed | 50% opacity (after revert point), still clickable |
| Current | Full opacity, subtle "current" indicator |

### Revert Flow

1. User clicks "Revert to here"
2. Confirmation toast: "Revert to this version?" [Confirm] [Cancel]
3. On confirm:
   - `git checkout {hash} -- .` to restore files
   - `git add -A && git commit -m "Reverted to: {prompt}"`
   - Update message states (dim messages after revert point)
4. Success toast: "Code reverted"

### Non-Destructive

Revert creates a new commit - old commits remain accessible. User can revert to any checkpoint at any time.

---

## 3. Persistent Chat History

### Storage

```
~/VSTWorkshop/projects/{name}/.vstworkshop/chat.json
```

### Schema

```json
{
  "messages": [
    {
      "id": "uuid",
      "role": "user" | "assistant",
      "content": "message text",
      "timestamp": "2024-01-05T10:30:00Z",
      "commitHash": "abc123",
      "reverted": false
    }
  ],
  "lastUpdated": "2024-01-05T10:35:00Z"
}
```

### Save Triggers

- After each message added
- After revert action
- Debounced (100ms) to reduce disk I/O

### Load Triggers

- When project selected in sidebar
- On app startup (if project was previously active)

### Backend Commands

```rust
#[tauri::command]
fn save_chat_history(project_path: String, messages: Vec<ChatMessage>) -> Result<(), String>

#[tauri::command]
fn load_chat_history(project_path: String) -> Result<Vec<ChatMessage>, String>
```

---

## Files to Create/Modify

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/git.rs` | Git operations (init, commit, revert) |
| `src-tauri/src/commands/chat.rs` | Chat history save/load |
| `src/components/Chat/ChatMessage.tsx` | Add revert link, dimmed state |
| `src/components/Chat/ChatPanel.tsx` | Load/save history, handle reverts |
| `src/types/index.ts` | Update ChatMessage type with commitHash, reverted |

---

## Edge Cases

- **No git**: Should never happen (Xcode CLI includes git), but fail gracefully
- **Git command fails**: Show error toast, don't change UI state
- **Empty commit**: Skip commit if no files changed
- **Corrupt chat.json**: Start fresh, log warning
