use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub path: String,
}

#[derive(Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: String,
}

pub fn get_workspace_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("VSTWorkshop")
}

pub fn get_output_path() -> PathBuf {
    get_workspace_path().join("output")
}

fn get_projects_path() -> PathBuf {
    get_workspace_path().join("projects")
}

/// Ensure the workspace directories exist and workspace Cargo.toml is set up
pub fn ensure_workspace() -> Result<(), String> {
    let workspace = get_workspace_path();
    let projects = get_projects_path();
    let output = workspace.join("output");
    let xtask_dir = workspace.join("xtask/src");

    fs::create_dir_all(&projects).map_err(|e| format!("Failed to create projects dir: {}", e))?;
    fs::create_dir_all(&output).map_err(|e| format!("Failed to create output dir: {}", e))?;
    fs::create_dir_all(&xtask_dir).map_err(|e| format!("Failed to create xtask dir: {}", e))?;

    // Create workspace root Cargo.toml if it doesn't exist
    let workspace_cargo = workspace.join("Cargo.toml");
    if !workspace_cargo.exists() {
        let cargo_content = r#"[workspace]
members = ["projects/*", "xtask"]
resolver = "2"
"#;
        fs::write(&workspace_cargo, cargo_content)
            .map_err(|e| format!("Failed to create workspace Cargo.toml: {}", e))?;
    }

    // Create shared xtask Cargo.toml if it doesn't exist
    let xtask_cargo = workspace.join("xtask/Cargo.toml");
    if !xtask_cargo.exists() {
        let xtask_content = r#"[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git" }
"#;
        fs::write(&xtask_cargo, xtask_content)
            .map_err(|e| format!("Failed to create xtask Cargo.toml: {}", e))?;
    }

    // Create shared xtask main.rs if it doesn't exist
    let xtask_main = workspace.join("xtask/src/main.rs");
    if !xtask_main.exists() {
        let main_content = r#"fn main() -> nih_plug_xtask::Result<()> {
    nih_plug_xtask::main()
}
"#;
        fs::write(&xtask_main, main_content)
            .map_err(|e| format!("Failed to create xtask main.rs: {}", e))?;
    }

    // Create .cargo/config.toml with xtask alias if it doesn't exist
    let cargo_config_dir = workspace.join(".cargo");
    fs::create_dir_all(&cargo_config_dir)
        .map_err(|e| format!("Failed to create .cargo dir: {}", e))?;

    let cargo_config = cargo_config_dir.join("config.toml");
    if !cargo_config.exists() {
        let config_content = r#"[alias]
xtask = "run --package xtask --release --"
"#;
        fs::write(&cargo_config, config_content)
            .map_err(|e| format!("Failed to create cargo config: {}", e))?;
    }

    Ok(())
}

/// Validate plugin name (lowercase, no spaces, valid Rust identifier)
fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.len() > 50 {
        return Err("Name too long (max 50 chars)".to_string());
    }
    if !name.chars().next().unwrap().is_ascii_lowercase() {
        return Err("Name must start with a lowercase letter".to_string());
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
        return Err("Name can only contain lowercase letters, numbers, hyphens, and underscores".to_string());
    }
    Ok(())
}

/// Convert name to valid Rust identifier (snake_case)
fn to_snake_case(name: &str) -> String {
    name.replace('-', "_")
}

/// Convert name to PascalCase for struct names
fn to_pascal_case(name: &str) -> String {
    name.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Generate a unique VST3 class ID from the plugin name
fn generate_vst3_id(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();

    // Create a 16-byte ID string
    format!("VSTWorkshop{:05}", hash % 100000)
        .chars()
        .take(16)
        .collect()
}

#[tauri::command]
pub async fn create_project(input: CreateProjectInput) -> Result<ProjectMeta, String> {
    validate_name(&input.name)?;
    ensure_workspace()?;

    let project_path = get_projects_path().join(&input.name);

    if project_path.exists() {
        return Err(format!("Project '{}' already exists", input.name));
    }

    // Create directory structure
    fs::create_dir_all(project_path.join("src"))
        .map_err(|e| format!("Failed to create src dir: {}", e))?;
    fs::create_dir_all(project_path.join(".vstworkshop"))
        .map_err(|e| format!("Failed to create .vstworkshop dir: {}", e))?;

    let snake_name = to_snake_case(&input.name);
    let pascal_name = to_pascal_case(&input.name);
    let vst3_id = generate_vst3_id(&input.name);

    // Write Cargo.toml (project is a workspace member, no [workspace] section needed)
    let cargo_toml = format!(
        r#"[package]
name = "{snake_name}"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-only"
description = "{description}"

[lib]
crate-type = ["cdylib"]

[dependencies]
nih_plug = {{ git = "https://github.com/robbert-vdh/nih-plug.git", features = ["assert_process_allocs"] }}

[profile.release]
lto = "thin"
strip = "symbols"
"#,
        snake_name = snake_name,
        description = input.description.replace('"', "\\\"")
    );
    fs::write(project_path.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Write src/lib.rs (minimal passthrough template)
    let lib_rs = format!(
        r#"use nih_plug::prelude::*;
use std::sync::Arc;

/// {description}
struct {pascal_name} {{
    params: Arc<{pascal_name}Params>,
}}

#[derive(Params)]
struct {pascal_name}Params {{
    #[id = "gain"]
    pub gain: FloatParam,
}}

impl Default for {pascal_name} {{
    fn default() -> Self {{
        Self {{
            params: Arc::new({pascal_name}Params::default()),
        }}
    }}
}}

impl Default for {pascal_name}Params {{
    fn default() -> Self {{
        Self {{
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {{
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                }},
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }}
    }}
}}

impl Plugin for {pascal_name} {{
    const NAME: &'static str = "{display_name}";
    const VENDOR: &'static str = "VSTWorkshop";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {{
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }}];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {{
        self.params.clone()
    }}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {{
        for channel_samples in buffer.iter_samples() {{
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {{
                *sample *= gain;
            }}
        }}
        ProcessStatus::Normal
    }}
}}

impl ClapPlugin for {pascal_name} {{
    const CLAP_ID: &'static str = "com.vstworkshop.{snake_name}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{description}");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}}

impl Vst3Plugin for {pascal_name} {{
    const VST3_CLASS_ID: [u8; 16] = *b"{vst3_id}";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}}

nih_export_clap!({pascal_name});
nih_export_vst3!({pascal_name});
"#,
        pascal_name = pascal_name,
        snake_name = snake_name,
        display_name = to_pascal_case(&input.name),
        description = input.description.replace('"', "\\\""),
        vst3_id = vst3_id
    );
    fs::write(project_path.join("src/lib.rs"), lib_rs)
        .map_err(|e| format!("Failed to write lib.rs: {}", e))?;

    // Create metadata
    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().to_string();

    let metadata = ProjectMeta {
        id: id.clone(),
        name: input.name.clone(),
        description: input.description.clone(),
        created_at: now.clone(),
        updated_at: now,
        path: project_path.to_string_lossy().to_string(),
    };

    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    fs::write(project_path.join(".vstworkshop/metadata.json"), metadata_json)
        .map_err(|e| format!("Failed to write metadata.json: {}", e))?;

    // Initialize git repository for version control
    let project_path_str = project_path.to_string_lossy().to_string();
    super::git::init_repo(&project_path_str)?;
    super::git::create_gitignore(&project_path_str)?;
    super::git::commit_changes(&project_path_str, "Initial plugin template")?;

    Ok(metadata)
}

#[tauri::command]
pub async fn list_projects() -> Result<Vec<ProjectMeta>, String> {
    ensure_workspace()?;

    let projects_dir = get_projects_path();
    let mut projects = Vec::new();

    let entries = fs::read_dir(&projects_dir)
        .map_err(|e| format!("Failed to read projects dir: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let metadata_path = path.join(".vstworkshop/metadata.json");
        if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)
                .map_err(|e| format!("Failed to read metadata: {}", e))?;
            let meta: ProjectMeta = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse metadata: {}", e))?;
            projects.push(meta);
        }
    }

    // Sort by updated_at descending (most recent first)
    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(projects)
}

#[tauri::command]
pub async fn get_project(name: String) -> Result<ProjectMeta, String> {
    let project_path = get_projects_path().join(&name);
    let metadata_path = project_path.join(".vstworkshop/metadata.json");

    if !metadata_path.exists() {
        return Err(format!("Project '{}' not found", name));
    }

    let content = fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    let meta: ProjectMeta = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse metadata: {}", e))?;

    Ok(meta)
}

#[tauri::command]
pub async fn delete_project(name: String) -> Result<(), String> {
    let project_path = get_projects_path().join(&name);

    if !project_path.exists() {
        return Err(format!("Project '{}' not found", name));
    }

    fs::remove_dir_all(&project_path)
        .map_err(|e| format!("Failed to delete project: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn open_project_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_workspace_path_string() -> String {
    get_workspace_path().to_string_lossy().to_string()
}
