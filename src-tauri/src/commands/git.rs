use std::process::Command;

/// Initialize a git repository in the given path
pub fn init_repo(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["init"])
        .output()
        .map_err(|e| format!("Failed to run git init: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git init failed: {}", stderr));
    }

    // Configure git user for this repo (avoid global config issues)
    let _ = Command::new("git")
        .current_dir(path)
        .args(["config", "user.email", "freqlab@local"])
        .output();

    let _ = Command::new("git")
        .current_dir(path)
        .args(["config", "user.name", "freqlab"])
        .output();

    Ok(())
}

/// Create a .gitignore file with standard Rust ignores
pub fn create_gitignore(path: &str) -> Result<(), String> {
    let gitignore_content = r#"# Build artifacts
target/

# Cargo lock (optional for libraries)
# Cargo.lock

# IDE
.idea/
.vscode/
*.swp
*.swo

# macOS
.DS_Store

# App state (chat history, sessions) - not source code
.vstworkshop/
"#;

    std::fs::write(format!("{}/.gitignore", path), gitignore_content)
        .map_err(|e| format!("Failed to create .gitignore: {}", e))?;

    Ok(())
}

/// Stage all changes and commit with the given message
pub fn commit_changes(path: &str, message: &str) -> Result<String, String> {
    // Stage all changes
    let add_output = Command::new("git")
        .current_dir(path)
        .args(["add", "-A"])
        .output()
        .map_err(|e| format!("Failed to run git add: {}", e))?;

    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        return Err(format!("git add failed: {}", stderr));
    }

    // Check if there are changes to commit
    let status_output = Command::new("git")
        .current_dir(path)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;

    let status = String::from_utf8_lossy(&status_output.stdout);
    if status.trim().is_empty() {
        // Nothing to commit - return error so caller knows no new commit was made
        return Err("no_changes".to_string());
    }

    // Commit
    let commit_output = Command::new("git")
        .current_dir(path)
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| format!("Failed to run git commit: {}", e))?;

    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        // Ignore "nothing to commit" errors
        if !stderr.contains("nothing to commit") {
            return Err(format!("git commit failed: {}", stderr));
        }
    }

    // Return the commit hash
    get_current_commit(path)
}

/// Get the current HEAD commit hash
pub fn get_current_commit(path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to run git rev-parse: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git rev-parse failed: {}", stderr));
    }

    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(hash)
}

/// Revert files to a specific commit (non-destructive - creates new commit)
/// Only reverts source code files, not app state (.vstworkshop/)
#[tauri::command]
pub async fn revert_to_commit(
    project_path: String,
    commit_hash: String,
    original_prompt: String,
) -> Result<String, String> {
    eprintln!("[DEBUG] revert_to_commit: commit={}, path={}", commit_hash, project_path);

    // Verify the commit exists first
    let verify = Command::new("git")
        .current_dir(&project_path)
        .args(["cat-file", "-t", &commit_hash])
        .output()
        .map_err(|e| format!("Failed to verify commit: {}", e))?;

    if !verify.status.success() {
        return Err(format!("Commit {} does not exist", commit_hash));
    }

    // Checkout only source files from the target commit with force flag
    // Exclude .vstworkshop/ which contains chat history and session state
    let checkout_output = Command::new("git")
        .current_dir(&project_path)
        .args([
            "checkout",
            "-f",  // Force checkout
            &commit_hash,
            "--",
            "src/",
            "Cargo.toml",
        ])
        .output()
        .map_err(|e| format!("Failed to run git checkout: {}", e))?;

    eprintln!("[DEBUG] Checkout status: {}", checkout_output.status);
    eprintln!("[DEBUG] Checkout stderr: {}", String::from_utf8_lossy(&checkout_output.stderr));

    if !checkout_output.status.success() {
        let stderr = String::from_utf8_lossy(&checkout_output.stderr);
        // Ignore errors about paths not existing (e.g., Cargo.lock might not exist)
        if !stderr.contains("did not match any") && !stderr.contains("pathspec") {
            return Err(format!("git checkout failed: {}", stderr));
        }
    }

    // Also try Cargo.lock (optional, may not exist)
    let _ = Command::new("git")
        .current_dir(&project_path)
        .args(["checkout", "-f", &commit_hash, "--", "Cargo.lock"])
        .output();

    // Create a new commit for the revert (if there are changes)
    let revert_message = format!("Reverted to: {}", truncate_string(&original_prompt, 50));
    match commit_changes(&project_path, &revert_message) {
        Ok(hash) => {
            eprintln!("[DEBUG] Created revert commit: {}", hash);
            Ok(hash)
        }
        Err(e) if e == "no_changes" => {
            // No changes to commit - files were already at this state
            eprintln!("[DEBUG] No changes after checkout (files already at target state)");
            get_current_commit(&project_path)
        }
        Err(e) => Err(e),
    }
}

/// Ensure .vstworkshop/ is not tracked by git (for existing projects)
pub fn ensure_vstworkshop_ignored(path: &str) -> Result<(), String> {
    // Check if .gitignore exists and contains .vstworkshop/
    let gitignore_path = format!("{}/.gitignore", path);
    let mut needs_update = true;

    if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
        if content.contains(".vstworkshop/") || content.contains(".vstworkshop") {
            needs_update = false;
        }
    }

    if needs_update {
        // Append to .gitignore
        let mut content = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str("\n# App state (chat history, sessions) - not source code\n.vstworkshop/\n");
        std::fs::write(&gitignore_path, content)
            .map_err(|e| format!("Failed to update .gitignore: {}", e))?;
    }

    // Remove .vstworkshop from git tracking if it's tracked
    let _ = Command::new("git")
        .current_dir(path)
        .args(["rm", "-r", "--cached", ".vstworkshop/"])
        .output();
    // Ignore errors - it might not be tracked

    Ok(())
}

/// Check if a path is a git repository
pub fn is_git_repo(path: &str) -> bool {
    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "--git-dir"])
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
