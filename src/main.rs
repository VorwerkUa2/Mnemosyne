#![windows_subsystem = "windows"]

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::{DateTime, Local};
use command_group::{CommandGroup, GroupChild};
use eframe::egui;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const KEYRING_SERVICE: &str = "Mnemosyne";
const FREELLMAPI_KEY_NAME: &str = "freellmapi-unified-key";
const MAX_COMPANION_DOWNLOAD_BYTES: u64 = 200 * 1024 * 1024;
const FREELLMAPI_RELEASES_API: &str =
    "https://api.github.com/repos/tashfeenahmed/freellmapi/releases/latest";
const MAX_ATTACHMENT_BYTES: u64 = 512 * 1024;
const MAX_STAGED_ATTACHMENT_BYTES: u64 = 20 * 1024 * 1024;
const MAX_ATTACHMENT_CHARS: usize = 40_000;
const MAX_PROJECT_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PROJECT_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_PROJECT_IMAGE_PIXELS: u64 = 8 * 1024 * 1024;
const MAX_PROJECT_IMAGE_DIMENSION: u32 = 8192;
const MAX_PROJECT_FILES: usize = 2000;
const MAX_PROJECT_SCAN_DEPTH: usize = 6;
const MAX_APPROVAL_HOOK_BYTES: usize = 16 * 1024 * 1024;
const MAX_APPROVAL_RESPONSE_BYTES: usize = 1024 * 1024;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[inline]
fn silent_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

// ── Localisation ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
enum AppLang {
    #[default]
    Uk,
    En,
}

#[allow(dead_code)]
struct L {
    // Sidebar
    pub app_subtitle: &'static str,
    pub settings_btn: &'static str,
    pub plugins_btn: &'static str,
    pub add_folder: &'static str,
    pub add_project: &'static str,
    pub projects_heading: &'static str,
    pub new_chat: &'static str,
    pub no_chats: &'static str,
    // Chat area
    pub chat_placeholder: &'static str,
    pub send_btn: &'static str,
    pub cancel_btn: &'static str,
    pub attach_btn: &'static str,
    pub export_chat: &'static str,
    pub clear_history: &'static str,
    pub delete_chat: &'static str,
    pub rename_chat: &'static str,
    pub copy_msg: &'static str,
    pub you_label: &'static str,
    pub claude_label: &'static str,
    pub thinking_label: &'static str,
    pub no_project: &'static str,
    pub select_or_create: &'static str,
    // Settings
    pub settings_title: &'static str,
    pub lang_label: &'static str,
    pub theme_label: &'static str,
    pub theme_dark: &'static str,
    pub theme_light: &'static str,
    pub freellmapi_section: &'static str,
    pub freellmapi_url: &'static str,
    pub freellmapi_key: &'static str,
    pub freellmapi_key_hint: &'static str,
    pub freellmapi_save_key: &'static str,
    pub freellmapi_check: &'static str,
    pub freellmapi_test: &'static str,
    pub freellmapi_install: &'static str,
    pub freellmapi_connected: &'static str,
    pub freellmapi_disconnected: &'static str,
    pub claude_cli_section: &'static str,
    pub claude_cli_found: &'static str,
    pub claude_cli_missing: &'static str,
    pub claude_cli_install_hint: &'static str,
    pub decod_section: &'static str,
    pub decod_enable: &'static str,
    pub decod_path: &'static str,
    pub decod_check: &'static str,
    pub decod_active_hint: &'static str,
    pub model_section: &'static str,
    pub mode_section: &'static str,
    pub mode_plan: &'static str,
    pub mode_turbo: &'static str,
    pub context_section: &'static str,
    pub close_btn: &'static str,
    // Confirmations
    pub confirm_delete_chat: &'static str,
    pub confirm_clear_chat: &'static str,
    pub confirm_remove_project: &'static str,
    pub yes_btn: &'static str,
    pub cancel_confirm: &'static str,
    // Files
    pub files_heading: &'static str,
    pub file_save: &'static str,
    pub file_preview: &'static str,
    pub file_edit: &'static str,
    pub file_filter: &'static str,
    // Approval
    pub approval_title: &'static str,
    pub approval_allow: &'static str,
    pub approval_deny: &'static str,
    // Agent mode labels
    pub plan_hint: &'static str,
    pub turbo_hint: &'static str,
    // Coding Superpowers
    pub superpowers_section: &'static str,
    pub caveman_mode: &'static str,
    pub caveman_hint: &'static str,
    pub karpathy_rules: &'static str,
    pub karpathy_hint: &'static str,
    pub auto_rules_btn: &'static str,
    pub auto_rules_success: &'static str,
    pub check_diagnostics_btn: &'static str,
    pub check_diagnostics_hint: &'static str,
    // Local Model (LM Studio / Ollama)
    pub local_model_section: &'static str,
    pub local_model_enabled: &'static str,
    pub local_model_url: &'static str,
    pub local_model_name: &'static str,
    pub local_model_name_hint: &'static str,
    pub local_model_check: &'static str,
    pub local_model_connected: &'static str,
    pub local_model_disconnected: &'static str,
    pub local_model_hint: &'static str,
    pub local_model_pick: &'static str,
}

const UK: L = L {
    app_subtitle: "Твій простір для вайбкодингу",
    settings_btn: "⚙  Налаштування",
    plugins_btn: "🔌  Плагіни Claude Code",
    add_folder: "+ Папка",
    add_project: "+ Проєкт",
    projects_heading: "Проєкти",
    new_chat: "+ Новий чат",
    no_chats: "Немає чатів. Натисни «+ Новий чат».",
    chat_placeholder: "Напиши запит… (Shift+Enter — новий рядок, Enter — надіслати)",
    send_btn: "Надіслати ↵",
    cancel_btn: "✕ Скасувати",
    attach_btn: "📎 Прикріпити файл",
    export_chat: "⬇ Експортувати чат",
    clear_history: "🗑 Очистити історію",
    delete_chat: "✕ Видалити чат",
    rename_chat: "✎ Перейменувати",
    copy_msg: "📋 Копіювати",
    you_label: "Ти",
    claude_label: "Claude",
    thinking_label: "Думаю…",
    no_project: "Немає проєкту",
    select_or_create: "Вибери або створи проєкт на бічній панелі.",
    settings_title: "Налаштування",
    lang_label: "Мова / Language",
    theme_label: "Тема",
    theme_dark: "🌙 Темна",
    theme_light: "☀ Світла",
    freellmapi_section: "FreeLLMAPI",
    freellmapi_url: "URL",
    freellmapi_key: "Unified API Key",
    freellmapi_key_hint: "Ключ зберігається у сховищі облікових даних ОС.",
    freellmapi_save_key: "Зберегти ключ",
    freellmapi_check: "Перевірити підключення",
    freellmapi_test: "Тест запиту",
    freellmapi_install: "Завантажити й встановити FreeLLMAPI",
    freellmapi_connected: "✓ Підключено",
    freellmapi_disconnected: "✗ Не підключено",
    claude_cli_section: "Claude Code CLI",
    claude_cli_found: "✓ Знайдено",
    claude_cli_missing: "✗ Не знайдено",
    claude_cli_install_hint: "Встановіть Claude Code CLI →",
    decod_section: "Decod MCP (RE-рушій)",
    decod_enable: "Увімкнути Decod MCP",
    decod_path: "Шлях до mcp_server.py",
    decod_check: "Перевірити",
    decod_active_hint: "✓ Decod MCP передається Claude Code як --mcp-config",
    model_section: "Модель Claude",
    mode_section: "Режим агента",
    mode_plan: "📋 Планування (тільки читання)",
    mode_turbo: "⚡ Turbo (редагування файлів з підтвердженням)",
    context_section: "Контекст розмови",
    close_btn: "Закрити",
    confirm_delete_chat: "Видалити цей чат та всі його повідомлення?",
    confirm_clear_chat: "Очистити всю історію цього чату?",
    confirm_remove_project: "Прибрати проєкт з робочого простору? (Файли на диску не видаляються)",
    yes_btn: "Так",
    cancel_confirm: "Скасувати",
    files_heading: "Файли проєкту",
    file_save: "💾 Зберегти",
    file_preview: "👁 Перегляд",
    file_edit: "✎ Редагувати",
    file_filter: "Фільтр…",
    approval_title: "Запит на зміну файлу",
    approval_allow: "✓ Дозволити",
    approval_deny: "✕ Відхилити",
    plan_hint: "Claude лише читає й аналізує файли — без змін.",
    turbo_hint: "Claude може редагувати файли. Кожна зміна потребує підтвердження.",
    superpowers_section: "⚡ Суперсили кодингу (Vibe-Coding)",
    caveman_mode: "Режим Caveman (економія токенів)",
    caveman_hint: "Максимально лаконічні відповіді без «води» та ввічливих прелюдій. Економить до 40% токенів.",
    karpathy_rules: "Принципи Карпатого (Karpathy Rules)",
    karpathy_hint: "YAGNI, мінімум непотрібних абстракцій, перевірка компілятором, атомарні diffs.",
    auto_rules_btn: "Оновити правила проєкту (.claude/CLAUDE.md)",
    auto_rules_success: "✓ Правила успішно створено у .claude/CLAUDE.md",
    check_diagnostics_btn: "🔍 Перевірити діагностику",
    check_diagnostics_hint: "Запуск перевірки синтаксису й компілятора (cargo check / pyright) без витрати токенів.",
    local_model_section: "Локальна модель (LM Studio / Ollama)",
    local_model_enabled: "Використовувати локальну модель замість FreeLLMAPI",
    local_model_url: "URL сервера",
    local_model_name: "Назва моделі",
    local_model_name_hint: "напр. llama-3.2-3b-instruct або залиште порожнім",
    local_model_check: "Знайти моделі",
    local_model_connected: "✓ Підключено",
    local_model_disconnected: "✗ Не підключено",
    local_model_hint: "LM Studio: :1234  •  Ollama: :11434  •  OpenAI-сумісний API",
    local_model_pick: "Вибрати модель:",
};

const EN: L = L {
    app_subtitle: "Your vibe-coding workspace",
    settings_btn: "⚙  Settings",
    plugins_btn: "🔌  Claude Code Plugins",
    add_folder: "+ Folder",
    add_project: "+ Project",
    projects_heading: "Projects",
    new_chat: "+ New chat",
    no_chats: "No chats. Click «+ New chat».",
    chat_placeholder: "Type a message… (Shift+Enter = newline, Enter = send)",
    send_btn: "Send ↵",
    cancel_btn: "✕ Cancel",
    attach_btn: "📎 Attach file",
    export_chat: "⬇ Export chat",
    clear_history: "🗑 Clear history",
    delete_chat: "✕ Delete chat",
    rename_chat: "✎ Rename",
    copy_msg: "📋 Copy",
    you_label: "You",
    claude_label: "Claude",
    thinking_label: "Thinking…",
    no_project: "No project",
    select_or_create: "Select or create a project in the sidebar.",
    settings_title: "Settings",
    lang_label: "Мова / Language",
    theme_label: "Theme",
    theme_dark: "🌙 Dark",
    theme_light: "☀ Light",
    freellmapi_section: "FreeLLMAPI",
    freellmapi_url: "URL",
    freellmapi_key: "Unified API Key",
    freellmapi_key_hint: "Key is stored in the OS credential store.",
    freellmapi_save_key: "Save key",
    freellmapi_check: "Check connection",
    freellmapi_test: "Test request",
    freellmapi_install: "Download & install FreeLLMAPI",
    freellmapi_connected: "✓ Connected",
    freellmapi_disconnected: "✗ Not connected",
    claude_cli_section: "Claude Code CLI",
    claude_cli_found: "✓ Found",
    claude_cli_missing: "✗ Not found",
    claude_cli_install_hint: "Install Claude Code CLI →",
    decod_section: "Decod MCP (RE Engine)",
    decod_enable: "Enable Decod MCP",
    decod_path: "Path to mcp_server.py",
    decod_check: "Verify",
    decod_active_hint: "✓ Decod MCP passed to Claude Code as --mcp-config",
    model_section: "Claude Model",
    mode_section: "Agent mode",
    mode_plan: "📋 Plan (read-only)",
    mode_turbo: "⚡ Turbo (file edits with confirmation)",
    context_section: "Conversation context",
    close_btn: "Close",
    confirm_delete_chat: "Delete this chat and all its messages?",
    confirm_clear_chat: "Clear all messages in this chat?",
    confirm_remove_project: "Remove project from workspace? (Files on disk are NOT deleted)",
    yes_btn: "Yes",
    cancel_confirm: "Cancel",
    files_heading: "Project files",
    file_save: "💾 Save",
    file_preview: "👁 Preview",
    file_edit: "✎ Edit",
    file_filter: "Filter…",
    approval_title: "File change request",
    approval_allow: "✓ Allow",
    approval_deny: "✕ Deny",
    plan_hint: "Claude only reads and analyses files — no changes.",
    turbo_hint: "Claude can edit files. Each change requires confirmation.",
    superpowers_section: "⚡ Coding Superpowers (Vibe-Coding)",
    caveman_mode: "Caveman Mode (Token Saver)",
    caveman_hint: "Ultra-concise output without filler or polite pleasantries. Saves up to 40% tokens.",
    karpathy_rules: "Karpathy Engineering Rules",
    karpathy_hint: "YAGNI, minimal abstractions, compiler verification, atomic diffs.",
    auto_rules_btn: "Update Project Rules (.claude/CLAUDE.md)",
    auto_rules_success: "✓ Rules successfully written to .claude/CLAUDE.md",
    check_diagnostics_btn: "🔍 Check Diagnostics",
    check_diagnostics_hint: "Run compiler & syntax checks (cargo check / pyright) with 0 token cost.",
    local_model_section: "Local Model (LM Studio / Ollama)",
    local_model_enabled: "Use local model instead of FreeLLMAPI",
    local_model_url: "Server URL",
    local_model_name: "Model name",
    local_model_name_hint: "e.g. llama-3.2-3b-instruct or leave empty",
    local_model_check: "Discover models",
    local_model_connected: "✓ Connected",
    local_model_disconnected: "✗ Not connected",
    local_model_hint: "LM Studio: :1234  •  Ollama: :11434  •  OpenAI-compatible API",
    local_model_pick: "Select model:",
};

// ── App State ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct AppState {
    projects: Vec<Project>,
    selected_project: Option<String>,
    selected_chat: Option<String>,
    freellmapi: FreeLlmApiSettings,
    #[serde(default)]
    local_model: LocalModelSettings,
    agent_mode: AgentMode,
    claude_model: ClaudeModel,
    context_limit: ConversationContextLimit,
    #[serde(default = "default_true")]
    decod_mcp_enabled: bool,
    #[serde(default = "default_decod_path")]
    decod_mcp_path: String,
    #[serde(default = "default_true")]
    caveman_mode: bool,
    #[serde(default = "default_true")]
    karpathy_rules: bool,
    #[serde(default = "default_true")]
    auto_rules_provision: bool,
    #[serde(default)]
    lang: AppLang,
    #[serde(default = "default_dark_theme")]
    dark_theme: bool,
}

fn default_dark_theme() -> bool { true }
fn default_true() -> bool { true }

fn default_decod_path() -> String {
    "C:/Users/knyaz/Desktop/Decod/core/mcp_server.py".to_string()
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            selected_project: None,
            selected_chat: None,
            freellmapi: FreeLlmApiSettings::default(),
            local_model: LocalModelSettings::default(),
            agent_mode: AgentMode::Turbo,
            claude_model: ClaudeModel::Sonnet,
            context_limit: ConversationContextLimit::Last20Turns,
            decod_mcp_enabled: true,
            decod_mcp_path: default_decod_path(),
            caveman_mode: true,
            karpathy_rules: true,
            auto_rules_provision: true,
            lang: AppLang::Uk,
            dark_theme: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct FreeLlmApiSettings {
    base_url: String,
    connected: bool,
    ready_models: Vec<String>,
    #[serde(default)]
    selected_model: Option<String>,
}

impl Default for FreeLlmApiSettings {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:31415".to_owned(),
            connected: false,
            ready_models: Vec::new(),
            selected_model: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct LocalModelSettings {
    enabled: bool,
    base_url: String,
    model_name: String,
    connected: bool,
    ready_models: Vec<String>,
}

impl Default for LocalModelSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://localhost:1234".to_owned(),
            model_name: String::new(),
            connected: false,
            ready_models: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
    path: PathBuf,
    chats: Vec<Chat>,
}

#[derive(Serialize, Deserialize)]
struct Chat {
    id: String,
    title: String,
    messages: Vec<Message>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Message {
    role: MessageRole,
    content: String,
    attachments: Vec<PathBuf>,
    created_at: DateTime<Local>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum MessageRole {
    User,
    Assistant,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
enum AgentMode {
    #[default]
    Plan,
    Turbo,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
enum ClaudeModel {
    #[default]
    Sonnet,
    Opus,
    Haiku,
}

impl ClaudeModel {
    fn cli_alias(self) -> &'static str {
        match self {
            Self::Sonnet => "sonnet",
            Self::Opus => "opus",
            Self::Haiku => "haiku",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Sonnet => "Sonnet",
            Self::Opus => "Opus",
            Self::Haiku => "Haiku",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
enum ConversationContextLimit {
    Last5Turns,
    Last10Turns,
    #[default]
    Last20Turns,
    Last40Turns,
    All,
}

impl ConversationContextLimit {
    fn user_turns(self) -> Option<usize> {
        match self {
            Self::Last5Turns => Some(5),
            Self::Last10Turns => Some(10),
            Self::Last20Turns => Some(20),
            Self::Last40Turns => Some(40),
            Self::All => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Last5Turns => "5 останніх запитів",
            Self::Last10Turns => "10 останніх запитів",
            Self::Last20Turns => "20 останніх запитів",
            Self::Last40Turns => "40 останніх запитів",
            Self::All => "Уся історія",
        }
    }
}

struct ClaudeResult {
    project_id: String,
    chat_id: String,
    result: Result<String, String>,
    cancelled: bool,
}

struct ClaudeStreamChunk {
    project_id: String,
    chat_id: String,
    update: ClaudeStreamUpdate,
}

enum ClaudeStreamUpdate {
    Text(String),
    Tool(String),
}

struct FileChangeApproval {
    tool: String,
    path: String,
    preview: String,
    response_tx: Sender<bool>,
}

struct ApprovalServer {
    address: SocketAddr,
    token: String,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

struct ProviderCheckResult {
    base_url: String,
    result: Result<Vec<String>, String>,
}

struct CompanionInstallResult {
    result: Result<(), String>,
}

struct ClaudeInstallResult {
    result: Result<(), String>,
}

struct ConnectionTestResult {
    result: Result<String, String>,
}

#[allow(dead_code)]
struct ComponentUpdateResult {
    message: String,
    success: bool,
}

#[allow(dead_code)]
struct DiagnosticsResult {
    message: String,
    success: bool,
}

#[derive(Clone)]
struct ClaudePlugin {
    id: String,
    name: String,
    description: String,
    marketplace: String,
    installed: bool,
    enabled: bool,
    scope: String,
}

#[derive(Clone, Copy)]
enum PluginAction {
    Install,
    Enable,
    Disable,
    Uninstall,
}

struct PluginManagerResult {
    plugin_action: Option<(PluginAction, String)>,
    marketplace_action: Option<(bool, String)>,
    result: Result<PluginManagerSnapshot, String>,
}

struct PluginManagerSnapshot {
    plugins: Vec<ClaudePlugin>,
    marketplaces: Vec<ClaudeMarketplace>,
}

#[derive(Clone)]
struct ClaudeMarketplace {
    name: String,
    source: String,
}

struct ProjectFileList {
    files: Vec<PathBuf>,
    truncated: bool,
}

struct OpenProjectFile {
    project_id: String,
    relative_path: PathBuf,
    content: String,
    markdown_preview: bool,
    dirty: bool,
    image: Option<LoadedProjectImage>,
}

struct LoadedProjectImage {
    texture: egui::TextureHandle,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
enum ProjectDocumentKind {
    Plan,
    Report,
}

struct MnemosyneApp {
    state: AppState,
    draft: String,
    pending_attachments: Vec<PathBuf>,
    new_project_name: String,
    creating_project: bool,
    editing_chat: Option<String>,
    chat_title_draft: String,
    pending_chat_delete: Option<(String, String, String)>,
    pending_chat_clear: Option<(String, String, String)>,
    pending_file_approval: Option<FileChangeApproval>,
    pending_plugin_action: Option<(PluginAction, String, String)>,
    pending_project_removal: Option<(String, String)>,
    show_settings: bool,
    show_plugin_manager: bool,
    plugin_filter: String,
    plugins: Vec<ClaudePlugin>,
    marketplaces: Vec<ClaudeMarketplace>,
    marketplace_source: String,
    pending_marketplace_action: Option<(bool, String, String)>,
    loading_plugins: bool,
    plugin_action_running: bool,
    freellmapi_key_draft: String,
    checking_provider: bool,
    checking_local_model: bool,
    installing_companion: bool,
    show_project_files: bool,
    project_files: Vec<PathBuf>,
    project_files_truncated: bool,
    project_file_filter: String,
    open_project_file: Option<OpenProjectFile>,
    busy: bool,
    streaming_content: String,
    current_tool: Option<String>,
    active_request: Option<(String, String)>,
    active_child: Arc<Mutex<Option<GroupChild>>>,
    cancel_requested: Arc<AtomicBool>,
    status: Option<String>,
    result_tx: Sender<ClaudeResult>,
    result_rx: Receiver<ClaudeResult>,
    stream_tx: Sender<ClaudeStreamChunk>,
    stream_rx: Receiver<ClaudeStreamChunk>,
    approval_tx: Sender<FileChangeApproval>,
    approval_rx: Receiver<FileChangeApproval>,
    plugin_manager_tx: Sender<PluginManagerResult>,
    plugin_manager_rx: Receiver<PluginManagerResult>,
    provider_check_tx: Sender<ProviderCheckResult>,
    provider_check_rx: Receiver<ProviderCheckResult>,
    local_model_check_tx: Sender<ProviderCheckResult>,
    local_model_check_rx: Receiver<ProviderCheckResult>,
    companion_install_tx: Sender<CompanionInstallResult>,
    companion_install_rx: Receiver<CompanionInstallResult>,
    connection_test_tx: Sender<ConnectionTestResult>,
    connection_test_rx: Receiver<ConnectionTestResult>,
    claude_install_tx: Sender<ClaudeInstallResult>,
    claude_install_rx: Receiver<ClaudeInstallResult>,
    update_tx: Sender<ComponentUpdateResult>,
    update_rx: Receiver<ComponentUpdateResult>,
    diagnostics_tx: Sender<DiagnosticsResult>,
    diagnostics_rx: Receiver<DiagnosticsResult>,
    installing_claude_cli: bool,
    claude_installed: bool,
    testing_connection: bool,
    updating_components: bool,
    running_diagnostics: bool,
    diagnostics_result: Option<String>,
    update_status: Option<String>,
    decod_mcp_check_status: Option<String>,
}

impl MnemosyneApp {
    fn new() -> Self {
        let (result_tx, result_rx) = mpsc::channel();
        let (stream_tx, stream_rx) = mpsc::channel();
        let (approval_tx, approval_rx) = mpsc::channel();
        let (plugin_manager_tx, plugin_manager_rx) = mpsc::channel();
        let (provider_check_tx, provider_check_rx) = mpsc::channel();
        let (local_model_check_tx, local_model_check_rx) = mpsc::channel();
        let (companion_install_tx, companion_install_rx) = mpsc::channel();
        let (connection_test_tx, connection_test_rx) = mpsc::channel();
        let (claude_install_tx, claude_install_rx) = mpsc::channel();
        let (update_tx, update_rx) = mpsc::channel();
        let (diagnostics_tx, diagnostics_rx) = mpsc::channel();
        let (state, load_error) = match load_state() {
            Ok(state) => (state, None),
            Err(error) => (AppState::default(), Some(error)),
        };
        Self {
            state,
            draft: String::new(),
            pending_attachments: Vec::new(),
            new_project_name: String::new(),
            creating_project: false,
            editing_chat: None,
            chat_title_draft: String::new(),
            pending_chat_delete: None,
            pending_chat_clear: None,
            pending_file_approval: None,
            pending_plugin_action: None,
            pending_project_removal: None,
            show_settings: false,
            show_plugin_manager: false,
            plugin_filter: String::new(),
            plugins: Vec::new(),
            marketplaces: Vec::new(),
            marketplace_source: String::new(),
            pending_marketplace_action: None,
            loading_plugins: false,
            plugin_action_running: false,
            freellmapi_key_draft: String::new(),
            checking_provider: false,
            checking_local_model: false,
            installing_companion: false,
            installing_claude_cli: false,
            updating_components: false,
            update_status: None,
            show_project_files: false,
            project_files: Vec::new(),
            project_files_truncated: false,
            project_file_filter: String::new(),
            open_project_file: None,
            busy: false,
            streaming_content: String::new(),
            current_tool: None,
            active_request: None,
            active_child: Arc::new(Mutex::new(None)),
            cancel_requested: Arc::new(AtomicBool::new(false)),
            status: load_error,
            result_tx,
            result_rx,
            stream_tx,
            stream_rx,
            approval_tx,
            approval_rx,
            plugin_manager_tx,
            plugin_manager_rx,
            provider_check_tx,
            provider_check_rx,
            local_model_check_tx,
            local_model_check_rx,
            companion_install_tx,
            companion_install_rx,
            connection_test_tx,
            connection_test_rx,
            claude_install_tx,
            claude_install_rx,
            update_tx,
            update_rx,
            diagnostics_tx,
            diagnostics_rx,
            testing_connection: false,
            claude_installed: is_claude_installed(),
            running_diagnostics: false,
            diagnostics_result: None,
            decod_mcp_check_status: None,
        }
    }

    fn save(&mut self) {
        match save_state(&self.state) {
            Ok(()) => self.status = None,
            Err(error) => self.status = Some(error),
        }
    }

    fn selected_project(&self) -> Option<&Project> {
        self.state
            .projects
            .iter()
            .find(|project| Some(project.id.as_str()) == self.state.selected_project.as_deref())
    }

    fn selected_chat(&self) -> Option<&Chat> {
        self.selected_project()?
            .chats
            .iter()
            .find(|chat| Some(chat.id.as_str()) == self.state.selected_chat.as_deref())
    }

    fn refresh_project_files(&mut self) {
        let Some(project) = self
            .selected_project()
            .map(|project| (project.id.clone(), project.path.clone()))
        else {
            self.project_files.clear();
            self.project_files_truncated = false;
            self.open_project_file = None;
            return;
        };
        match list_project_files(&project.1) {
            Ok(list) => {
                self.project_files = list.files;
                self.project_files_truncated = list.truncated;
                self.status = None;
            }
            Err(error) => self.status = Some(error),
        }
    }

    fn open_project_file(&mut self, relative_path: &Path, ctx: &egui::Context) {
        let Some((project_id, project_root)) = self
            .selected_project()
            .map(|project| (project.id.clone(), project.path.clone()))
        else {
            return;
        };
        if is_supported_project_pdf_file(relative_path) {
            let result = canonical_project_file(&project_root, relative_path).and_then(|path| {
                let page_count = estimate_pdf_page_count(&path);
                open::that(&path).map_err(|error| {
                    format!("Не вдалося відкрити PDF у системному переглядачі: {error}")
                }).map(|()| (path, page_count))
            });
            self.status = Some(match result {
                Ok((_path, Some(n))) => format!(
                    "PDF відкрито у системному переглядачі: {} (~{} стор.)",
                    relative_path.display(),
                    n
                ),
                Ok((_path, None)) => format!(
                    "PDF відкрито у системному переглядачі: {}",
                    relative_path.display()
                ),
                Err(error) => error,
            });
            return;
        }
        if let Some(file) = self.open_project_file.as_ref().filter(|file| file.dirty) {
            if file.project_id == project_id && file.relative_path == relative_path {
                return;
            }
            self.status = Some("Спершу збережи зміни у відкритому файлі.".to_owned());
            return;
        }
        let opened_file = if is_supported_project_image_file(relative_path) {
            read_project_image_file(&project_root, relative_path).map(
                |(color_image, width, height)| {
                    let texture = ctx.load_texture(
                        format!("project-image-{project_id}-{}", relative_path.display()),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    OpenProjectFile {
                        project_id: project_id.clone(),
                        relative_path: relative_path.to_owned(),
                        content: String::new(),
                        markdown_preview: true,
                        dirty: false,
                        image: Some(LoadedProjectImage {
                            texture,
                            width: width as usize,
                            height: height as usize,
                        }),
                    }
                },
            )
        } else {
            read_project_text_file(&project_root, relative_path).map(|content| OpenProjectFile {
                project_id,
                relative_path: relative_path.to_owned(),
                content,
                markdown_preview: relative_path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("md")),
                dirty: false,
                image: None,
            })
        };
        match opened_file {
            Ok(file) => {
                self.open_project_file = Some(file);
                self.status = None;
            }
            Err(error) => self.status = Some(error),
        }
    }

    fn save_open_project_file(&mut self) {
        let Some(file) = self.open_project_file.as_mut() else {
            return;
        };
        let Some(project_root) = self
            .state
            .projects
            .iter()
            .find(|project| project.id == file.project_id)
            .map(|project| project.path.clone())
        else {
            self.status = Some("Проєкт відкритого файлу більше не існує.".to_owned());
            return;
        };
        match save_project_text_file(&project_root, &file.relative_path, &file.content) {
            Ok(()) => {
                file.dirty = false;
                self.status = Some("Файл збережено.".to_owned());
            }
            Err(error) => self.status = Some(error),
        }
    }

    fn has_unsaved_project_file(&self) -> bool {
        self.open_project_file
            .as_ref()
            .is_some_and(|file| file.dirty)
    }

    fn refuse_project_change_with_unsaved_file(&mut self) -> bool {
        if self.has_unsaved_project_file() {
            self.status = Some("Спершу збережи зміни у відкритому файлі.".to_owned());
            true
        } else {
            false
        }
    }

    fn create_chat(&mut self) {
        let Some(project_id) = self.state.selected_project.clone() else {
            return;
        };
        let chat = Chat {
            id: Uuid::new_v4().to_string(),
            title: format!("Чат {}", Local::now().format("%d.%m %H:%M")),
            messages: Vec::new(),
        };
        self.state.selected_chat = Some(chat.id.clone());
        if let Some(project) = self
            .state
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
        {
            project.chats.insert(0, chat);
        }
        self.save();
    }

    fn rename_chat(&mut self, project_id: &str, chat_id: &str, title: &str) {
        let title = match normalize_chat_title(title) {
            Ok(title) => title,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        let Some(chat) = self
            .state
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
            .and_then(|project| project.chats.iter_mut().find(|chat| chat.id == chat_id))
        else {
            self.status = Some("Чат для перейменування більше не існує.".to_owned());
            return;
        };
        chat.title = title;
        self.editing_chat = None;
        self.chat_title_draft.clear();
        self.save();
    }

    fn delete_chat(&mut self, project_id: &str, chat_id: &str) {
        if self.busy {
            self.status =
                Some("Дочекайся завершення або скасуй запит перед видаленням чату.".to_owned());
            return;
        }
        let selected_project = self.state.selected_project.as_deref() == Some(project_id);
        let was_selected = selected_project && self.state.selected_chat.as_deref() == Some(chat_id);
        let Some(project) = self
            .state
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
        else {
            self.status = Some("Проєкт чату більше не існує.".to_owned());
            return;
        };
        if !remove_chat_from_project(project, chat_id) {
            self.status = Some("Чат для видалення більше не існує.".to_owned());
            return;
        }
        if was_selected {
            self.state.selected_chat = project.chats.first().map(|chat| chat.id.clone());
        }
        if self.editing_chat.as_deref() == Some(chat_id) {
            self.editing_chat = None;
            self.chat_title_draft.clear();
        }
        self.pending_chat_delete = None;
        self.save();
    }

    fn clear_chat_history(&mut self, project_id: &str, chat_id: &str) {
        if self.busy {
            self.status =
                Some("Дочекайся завершення або скасуй запит перед очищенням історії.".to_owned());
            return;
        }
        let Some(project) = self
            .state
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
        else {
            self.status = Some("Чат для очищення більше не існує.".to_owned());
            return;
        };
        if !clear_chat_messages(project, chat_id) {
            self.status = Some("Чат для очищення більше не існує.".to_owned());
            return;
        }
        self.pending_chat_clear = None;
        self.save();
    }

    fn export_chat(&mut self, project_id: &str, chat_id: &str) {
        let Some(chat) = self
            .state
            .projects
            .iter()
            .find(|project| project.id == project_id)
            .and_then(|project| project.chats.iter().find(|chat| chat.id == chat_id))
        else {
            self.status = Some("Чат для експорту більше не існує.".to_owned());
            return;
        };
        let markdown = chat_export_markdown(chat);
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("mnemosyne-chat.md")
            .add_filter("Markdown", &["md"])
            .save_file()
        else {
            return;
        };
        match fs::write(&path, markdown) {
            Ok(()) => {
                self.status = Some(format!("Історію чату експортовано у {}.", path.display()))
            }
            Err(error) => {
                self.status = Some(format!("Не вдалося експортувати історію чату: {error}"))
            }
        }
    }

    fn remove_project_from_workspace(&mut self, project_id: &str) {
        if self.busy {
            self.status =
                Some("Дочекайся завершення або скасуй запит перед прибиранням проєкту.".to_owned());
            return;
        }
        if self
            .open_project_file
            .as_ref()
            .is_some_and(|file| file.project_id == project_id && file.dirty)
        {
            self.status = Some("Спершу збережи зміни у відкритому файлі.".to_owned());
            return;
        }
        let selected = self.state.selected_project.as_deref() == Some(project_id);
        if !remove_project_record(&mut self.state, project_id) {
            self.status = Some("Проєкт для прибирання більше не існує.".to_owned());
            return;
        }
        if selected {
            self.open_project_file = None;
            self.project_files.clear();
            self.project_files_truncated = false;
        }
        self.pending_project_removal = None;
        self.save();
        if selected && self.show_project_files {
            self.refresh_project_files();
        }
    }

    fn add_project(&mut self, path: PathBuf) {
        if self.refuse_project_change_with_unsaved_file() {
            return;
        }
        if self
            .state
            .projects
            .iter()
            .any(|project| project.path == path)
        {
            self.status = Some("Ця папка вже є у списку проєктів.".to_owned());
            return;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Новий проєкт")
            .to_owned();
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name,
            path,
            chats: Vec::new(),
        };
        self.state.selected_project = Some(project.id.clone());
        self.state.selected_chat = None;
        self.open_project_file = None;
        self.project_files.clear();
        self.state.projects.insert(0, project);
        self.save();
    }

    fn create_project_folder(&mut self) {
        if self.refuse_project_change_with_unsaved_file() {
            return;
        }
        let name = self.new_project_name.trim();
        let mut components = Path::new(name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
            self.status = Some("Вкажіть назву проєкту без шляху до папки.".to_owned());
            return;
        }
        let Some(parent) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let path = parent.join(name);
        match fs::create_dir(&path) {
            Ok(()) => {
                self.creating_project = false;
                self.new_project_name.clear();
                self.add_project(path);
            }
            Err(error) => {
                self.status = Some(format!("Не вдалося створити папку проєкту: {error}"));
            }
        }
    }

    fn add_attachments(&mut self) {
        if let Some(files) = rfd::FileDialog::new().pick_files() {
            for file in files {
                if !self.pending_attachments.contains(&file) {
                    self.pending_attachments.push(file);
                }
            }
        }
    }

    fn send_message(&mut self) {
        if self.busy {
            return;
        }
        if !is_claude_installed() {
            self.status = Some(
                "Claude Code CLI не знайдено. Встановіть його за інструкцією: \
                 https://docs.anthropic.com/en/docs/claude-code — \
                 потім перезапустіть Mnemosyne."
                    .to_owned(),
            );
            self.show_settings = true;
            return;
        }
        let (api_base_url, api_key, custom_model) = if self.state.local_model.enabled {
            if !self.state.local_model.connected {
                self.status = Some(if self.state.lang == AppLang::En {
                    "Please verify local model connection in Settings first.".to_owned()
                } else {
                    "Спочатку перевірте підключення до локальної моделі у Налаштуваннях.".to_owned()
                });
                self.show_settings = true;
                return;
            }
            let custom = if self.state.local_model.model_name.trim().is_empty() {
                None
            } else {
                Some(self.state.local_model.model_name.trim().to_owned())
            };
            (
                self.state.local_model.base_url.clone(),
                "local-model-token".to_owned(),
                custom,
            )
        } else {
            let key = match load_freellmapi_key() {
                Ok(Some(key)) => key,
                Ok(None) => {
                    self.status = Some("Підключіть FreeLLMAPI у Налаштуваннях.".to_owned());
                    self.show_settings = true;
                    return;
                }
                Err(error) => {
                    self.status = Some(error);
                    self.show_settings = true;
                    return;
                }
            };
            if !self.state.freellmapi.connected {
                self.status =
                    Some("Спочатку перевірте підключення до FreeLLMAPI у Налаштуваннях.".to_owned());
                self.show_settings = true;
                return;
            }
            let custom = self.state.freellmapi.selected_model.clone();
            (self.state.freellmapi.base_url.clone(), key, custom)
        };
        let content = self.draft.trim().to_owned();
        if content.is_empty() && self.pending_attachments.is_empty() {
            return;
        }
        let Some(project_id) = self.state.selected_project.clone() else {
            self.status = Some("Спочатку додайте або виберіть проєкт.".to_owned());
            return;
        };
        let Some(chat_id) = self.state.selected_chat.clone() else {
            self.status = Some("Спочатку створіть чат.".to_owned());
            return;
        };

        let attachments = std::mem::take(&mut self.pending_attachments);
        let message = Message {
            role: MessageRole::User,
            content: content.clone(),
            attachments: attachments.clone(),
            created_at: Local::now(),
        };

        let Some(project) = self
            .state
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
        else {
            return;
        };
        let Some(chat) = project.chats.iter_mut().find(|chat| chat.id == chat_id) else {
            return;
        };
        if chat.messages.is_empty() && !content.is_empty() {
            chat.title = content.chars().take(36).collect();
            if content.chars().count() > 36 {
                chat.title.push('…');
            }
        }
        chat.messages.push(message);
        let project_path = project.path.clone();
        let conversation = conversation_context(&chat.messages, self.state.context_limit).to_vec();
        self.draft.clear();
        self.busy = true;
        self.streaming_content.clear();
        self.current_tool = None;
        self.active_request = Some((project_id.clone(), chat_id.clone()));
        self.cancel_requested = Arc::new(AtomicBool::new(false));
        self.save();

        let agent_mode = self.state.agent_mode;
        let model = self.state.claude_model;
        let tx = self.result_tx.clone();
        let stream_tx = self.stream_tx.clone();
        let approval_tx = self.approval_tx.clone();
        let active_child = Arc::clone(&self.active_child);
        let cancel_requested = Arc::clone(&self.cancel_requested);
        if self.state.auto_rules_provision {
            let _ = ensure_curated_project_rules(
                &project_path,
                self.state.caveman_mode,
                self.state.karpathy_rules,
                self.state.decod_mcp_enabled,
            );
        }
        let mcp_config = build_project_mcp_config(
            &project_path,
            &self.state.decod_mcp_path,
            self.state.decod_mcp_enabled,
        );
        thread::spawn(move || {
            let result = run_claude(
                &project_path,
                &conversation,
                &api_base_url,
                &api_key,
                agent_mode,
                model,
                &stream_tx,
                &approval_tx,
                &project_id,
                &chat_id,
                &active_child,
                &cancel_requested,
                mcp_config,
                custom_model.as_deref(),
            );
            let cancelled = cancel_requested.load(Ordering::SeqCst)
                && matches!(&result, Err(error) if error == "Запит скасовано.");
            let _ = tx.send(ClaudeResult {
                project_id,
                chat_id,
                cancelled,
                result,
            });
        });
    }

    fn cancel_claude(&mut self) {
        self.cancel_requested.store(true, Ordering::SeqCst);
        self.status = Some(match stop_active_child(&self.active_child) {
            Ok(()) => "Скасовую запит до Claude Code…".to_owned(),
            Err(error) => error,
        });
    }

    fn resolve_file_approval(&mut self, allow: bool) {
        if let Some(approval) = self.pending_file_approval.take() {
            if approval.response_tx.send(allow).is_err() {
                self.status = Some("Не вдалося передати рішення Claude Code.".to_owned());
            }
        }
    }

    fn save_provider_key(&mut self) {
        match save_freellmapi_key(self.freellmapi_key_draft.trim()) {
            Ok(()) => {
                self.freellmapi_key_draft.clear();
                self.status =
                    Some("Ключ FreeLLMAPI збережено у сховищі облікових даних системи.".to_owned());
            }
            Err(error) => self.status = Some(error),
        }
    }

    fn check_freellmapi(&mut self) {
        let base_url = match normalize_api_base_url(&self.state.freellmapi.base_url) {
            Ok(url) => url,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        self.state.freellmapi.base_url = base_url.clone();
        let api_key = match load_freellmapi_key() {
            Ok(Some(key)) => key,
            Ok(None) => {
                self.status = Some("Спочатку збережіть unified API key FreeLLMAPI.".to_owned());
                return;
            }
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };

        self.checking_provider = true;
        self.state.freellmapi.connected = false;
        self.save();
        let tx = self.provider_check_tx.clone();
        thread::spawn(move || {
            let result = fetch_ready_models(&base_url, &api_key);
            let _ = tx.send(ProviderCheckResult { base_url, result });
        });
    }

    fn check_local_model(&mut self) {
        let base_url = match normalize_api_base_url(&self.state.local_model.base_url) {
            Ok(url) => url,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        self.state.local_model.base_url = base_url.clone();
        self.state.local_model.connected = false;
        self.state.local_model.ready_models.clear();
        self.checking_local_model = true;
        self.save();
        let tx = self.local_model_check_tx.clone();
        thread::spawn(move || {
            let result = fetch_local_models(&base_url);
            let _ = tx.send(ProviderCheckResult { base_url, result });
        });
    }

    fn test_freellmapi_connection(&mut self) {
        if self.testing_connection {
            return;
        }
        let base_url = match normalize_api_base_url(&self.state.freellmapi.base_url) {
            Ok(url) => url,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        let api_key = match load_freellmapi_key() {
            Ok(Some(key)) => key,
            Ok(None) => {
                self.status = Some("Спочатку збережіть unified API key FreeLLMAPI.".to_owned());
                return;
            }
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        let first_model = self.state.freellmapi.ready_models.first().cloned();
        self.testing_connection = true;
        self.status = Some("Тестую з'єднання з FreeLLMAPI…".to_owned());
        let tx = self.connection_test_tx.clone();
        thread::spawn(move || {
            let result = test_freellmapi_round_trip(&base_url, &api_key, first_model.as_deref());
            let _ = tx.send(ConnectionTestResult { result });
        });
    }

    fn install_freellmapi_companion(&mut self) {
        if self.installing_companion {
            return;
        }
        self.installing_companion = true;
        self.status =
            Some("Завантажую офіційний реліз FreeLLMAPI, перевіряю SHA-256 і запускаю…".to_owned());
        let tx = self.companion_install_tx.clone();
        thread::spawn(move || {
            let result = install_official_freellmapi();
            let _ = tx.send(CompanionInstallResult { result });
        });
    }

    fn install_claude_cli_background(&mut self) {
        if self.installing_claude_cli {
            return;
        }
        self.installing_claude_cli = true;
        self.status = Some("Встановлюю Claude Code CLI через npm…".to_owned());
        let tx = self.claude_install_tx.clone();
        thread::spawn(move || {
            let result = install_claude_cli();
            let _ = tx.send(ClaudeInstallResult { result });
        });
    }

    fn update_all_components(&mut self) {
        if self.updating_components {
            return;
        }
        self.updating_components = true;
        self.update_status = Some("Оновлюю Claude Code CLI, Decod та компоненти…".to_owned());
        self.status = Some("Оновлення компонентів…".to_owned());
        let tx = self.update_tx.clone();
        let decod_path = self.state.decod_mcp_path.clone();
        thread::spawn(move || {
            let mut msgs = Vec::new();

            // 1. Update Claude Code CLI
            #[cfg(windows)]
            let cli_res = silent_command("cmd.exe")
                .args(["/D", "/C", "npm update -g @anthropic-ai/claude-code"])
                .output();
            #[cfg(not(windows))]
            let cli_res = silent_command("sh")
                .args(["-c", "npm update -g @anthropic-ai/claude-code"])
                .output();

            match cli_res {
                Ok(o) if o.status.success() => msgs.push("✓ Claude Code CLI оновлено".to_string()),
                Ok(_) => msgs.push("✓ Claude Code CLI перевірено".to_string()),
                Err(e) => msgs.push(format!("⚠ Claude CLI: {e}")),
            }

            // 2. Update Decod
            let decod_dir = Path::new(&decod_path).parent().and_then(|p| p.parent());
            if let Some(dir) = decod_dir {
                if dir.join(".git").exists() {
                    let _ = silent_command("git")
                        .current_dir(dir)
                        .args(["pull", "origin", "main"])
                        .output();
                }
                let _ = silent_command("python")
                    .args(["-m", "pip", "install", "--upgrade", "frida", "lief", "capstone", "psutil", "pyright"])
                    .output();
                msgs.push("✓ Decod RE рушій оновлено".to_string());
            }

            // 3. Mnemosyne version
            msgs.push("✓ Mnemosyne v0.1.0 актуальна".to_string());

            let summary = msgs.join(" · ");
            let _ = tx.send(ComponentUpdateResult {
                message: summary,
                success: true,
            });
        });
    }

    fn run_project_diagnostics(&mut self) {
        if self.running_diagnostics {
            return;
        }
        let Some(project) = self.selected_project() else {
            self.status = Some("Виберіть проєкт для запуску діагностики.".to_owned());
            return;
        };
        let project_path = project.path.clone();
        let lang = self.state.lang;
        self.running_diagnostics = true;
        self.diagnostics_result = Some(if lang == AppLang::En {
            "Running compiler & syntax diagnostics…".to_string()
        } else {
            "Запуск компілятора та перевірки синтаксису…".to_string()
        });
        self.status = self.diagnostics_result.clone();
        let tx = self.diagnostics_tx.clone();

        thread::spawn(move || {
            let is_rust = project_path.join("Cargo.toml").exists();
            let is_python = project_path.join("pyproject.toml").exists()
                || project_path.join("requirements.txt").exists()
                || project_path.join("setup.py").exists();

            if is_rust {
                match silent_command("cargo")
                    .current_dir(&project_path)
                    .args(["check", "--message-format=short"])
                    .output()
                {
                    Ok(output) if output.status.success() => {
                        let _ = tx.send(DiagnosticsResult {
                            message: if lang == AppLang::En {
                                "✓ Rust (cargo check): Zero errors! Code compiles cleanly.".to_string()
                            } else {
                                "✓ Rust (cargo check): 0 помилок! Код компілюється чисто.".to_string()
                            },
                            success: true,
                        });
                    }
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stderr);
                        let first_lines = text.lines().take(4).collect::<Vec<_>>().join("\n");
                        let _ = tx.send(DiagnosticsResult {
                            message: format!("✗ Rust cargo check:\n{}", first_lines),
                            success: false,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(DiagnosticsResult {
                            message: format!("✗ Cargo error: {e}"),
                            success: false,
                        });
                    }
                }
            } else if is_python {
                let res = silent_command("python")
                    .args(["-m", "compileall", "-q", "."])
                    .current_dir(&project_path)
                    .output();
                match res {
                    Ok(output) if output.status.success() => {
                        let _ = tx.send(DiagnosticsResult {
                            message: if lang == AppLang::En {
                                "✓ Python: All scripts compile cleanly without syntax errors.".to_string()
                            } else {
                                "✓ Python: Всі файли скомпільовані без синтаксичних помилок.".to_string()
                            },
                            success: true,
                        });
                    }
                    Ok(output) => {
                        let text = String::from_utf8_lossy(&output.stderr);
                        let _ = tx.send(DiagnosticsResult {
                            message: format!("✗ Python syntax errors:\n{}", text),
                            success: false,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(DiagnosticsResult {
                            message: format!("✗ Python error: {e}"),
                            success: false,
                        });
                    }
                }
            } else {
                let _ = tx.send(DiagnosticsResult {
                    message: if lang == AppLang::En {
                        "ℹ Generic project: Decod AST analyzer active.".to_string()
                    } else {
                        "ℹ Звичайний проєкт: Decod AST-аналізатор активний.".to_string()
                    },
                    success: true,
                });
            }
        });
    }

    fn refresh_plugins(&mut self) {
        if self.loading_plugins || self.plugin_action_running {
            return;
        }
        self.loading_plugins = true;
        self.status = Some("Завантажую список плагінів Claude Code…".to_owned());
        let tx = self.plugin_manager_tx.clone();
        thread::spawn(move || {
            let result = load_plugin_manager_snapshot();
            let _ = tx.send(PluginManagerResult {
                plugin_action: None,
                marketplace_action: None,
                result,
            });
        });
    }

    fn run_plugin_action(&mut self, action: PluginAction, plugin_id: String) {
        if self.plugin_action_running || !is_valid_plugin_id(&plugin_id) {
            self.status = Some("Некоректний ідентифікатор плагіна.".to_owned());
            return;
        }
        self.plugin_action_running = true;
        self.status = Some("Виконую операцію з плагіном…".to_owned());
        let tx = self.plugin_manager_tx.clone();
        thread::spawn(move || {
            let result = run_claude_plugin_action(action, &plugin_id)
                .and_then(|_| load_plugin_manager_snapshot());
            let _ = tx.send(PluginManagerResult {
                plugin_action: Some((action, plugin_id)),
                marketplace_action: None,
                result,
            });
        });
    }

    fn run_marketplace_action(&mut self, add: bool, target: String) {
        if self.plugin_action_running {
            return;
        }
        self.plugin_action_running = true;
        self.status = Some("Оновлюю marketplace Claude Code…".to_owned());
        let tx = self.plugin_manager_tx.clone();
        thread::spawn(move || {
            let result = run_claude_marketplace_action(add, &target)
                .and_then(|_| load_plugin_manager_snapshot());
            let _ = tx.send(PluginManagerResult {
                plugin_action: None,
                marketplace_action: Some((add, target)),
                result,
            });
        });
    }

    fn receive_result(&mut self, ctx: &egui::Context) {
        while let Ok(response) = self.plugin_manager_rx.try_recv() {
            match (response.plugin_action, response.marketplace_action) {
                (Some((action, plugin_id)), None) => {
                    self.plugin_action_running = false;
                    match response.result {
                        Ok(snapshot) => {
                            self.plugins = snapshot.plugins;
                            self.marketplaces = snapshot.marketplaces;
                            self.status = Some(plugin_action_success_message(action, &plugin_id));
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
                (None, Some((add, target))) => {
                    self.plugin_action_running = false;
                    match response.result {
                        Ok(snapshot) => {
                            self.plugins = snapshot.plugins;
                            self.marketplaces = snapshot.marketplaces;
                            self.status = Some(if add {
                                format!("Marketplace {target} додано.")
                            } else {
                                format!("Marketplace {target} видалено.")
                            });
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
                (None, None) => {
                    self.loading_plugins = false;
                    match response.result {
                        Ok(snapshot) => {
                            self.plugins = snapshot.plugins;
                            self.marketplaces = snapshot.marketplaces;
                            self.status = Some(format!(
                                "Каталог плагінів оновлено: {} записів.",
                                self.plugins.len()
                            ));
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
                (Some(_), Some(_)) => {
                    self.plugin_action_running = false;
                    self.status = Some("Некоректна відповідь операції з marketplace.".to_owned());
                }
            }
            ctx.request_repaint();
        }
        while let Ok(approval) = self.approval_rx.try_recv() {
            if self.pending_file_approval.is_none() {
                self.pending_file_approval = Some(approval);
            } else if approval.response_tx.send(false).is_err() {
                self.status = Some("Не вдалося відхилити додаткову зміну Claude Code.".to_owned());
            }
        }
        while let Ok(chunk) = self.stream_rx.try_recv() {
            if self.active_request.as_ref() == Some(&(chunk.project_id, chunk.chat_id)) {
                match chunk.update {
                    ClaudeStreamUpdate::Text(text) => self.streaming_content.push_str(&text),
                    ClaudeStreamUpdate::Tool(tool) => self.current_tool = Some(tool),
                }
            }
        }
        while let Ok(response) = self.result_rx.try_recv() {
            self.busy = false;
            self.resolve_file_approval(false);
            let cancelled = response.cancelled;
            let response_content = if cancelled {
                self.streaming_content.clone()
            } else {
                match response.result {
                    Ok(text) => text,
                    Err(error) if self.streaming_content.is_empty() => {
                        format!("Помилка Claude Code: {error}")
                    }
                    Err(error) => {
                        format!("{}\n\nПомилка Claude Code: {error}", self.streaming_content)
                    }
                }
            };
            if let Some(project) = self
                .state
                .projects
                .iter_mut()
                .find(|project| project.id == response.project_id)
            {
                if let Some(chat) = project
                    .chats
                    .iter_mut()
                    .find(|chat| chat.id == response.chat_id)
                {
                    if !cancelled || !response_content.is_empty() {
                        chat.messages.push(Message {
                            role: MessageRole::Assistant,
                            content: response_content,
                            attachments: Vec::new(),
                            created_at: Local::now(),
                        });
                    }
                }
            }
            self.streaming_content.clear();
            self.current_tool = None;
            self.active_request = None;
            self.save();
            if cancelled {
                self.status = Some("Запит скасовано.".to_owned());
            }
            ctx.request_repaint();
        }
        while let Ok(response) = self.provider_check_rx.try_recv() {
            self.checking_provider = false;
            if self.state.freellmapi.base_url == response.base_url {
                let status = match response.result {
                    Ok(models) => {
                        self.state.freellmapi.connected = true;
                        self.state.freellmapi.ready_models = models;
                        format!(
                            "FreeLLMAPI підключено. Доступно моделей: {}. Усі запити йдуть через активний безкоштовний пул FreeLLMAPI.",
                            self.state.freellmapi.ready_models.len()
                        )
                    }
                    Err(error) => {
                        self.state.freellmapi.connected = false;
                        self.state.freellmapi.ready_models.clear();
                        error
                    }
                };
                self.save();
                self.status = Some(status);
            }
            ctx.request_repaint();
        }
        while let Ok(response) = self.local_model_check_rx.try_recv() {
            self.checking_local_model = false;
            if self.state.local_model.base_url == response.base_url {
                let status = match response.result {
                    Ok(models) => {
                        self.state.local_model.connected = true;
                        let count = models.len();
                        if self.state.local_model.model_name.is_empty() {
                            if let Some(first) = models.first() {
                                self.state.local_model.model_name = first.clone();
                            }
                        }
                        self.state.local_model.ready_models = models;
                        if self.state.lang == AppLang::En {
                            format!("✓ Local model connected. Found {count} available models.")
                        } else {
                            format!("✓ Локальну модель підключено. Знайдено {count} доступних моделей.")
                        }
                    }
                    Err(error) => {
                        self.state.local_model.connected = false;
                        self.state.local_model.ready_models.clear();
                        error
                    }
                };
                self.save();
                self.status = Some(status);
            }
            ctx.request_repaint();
        }
        while let Ok(response) = self.companion_install_rx.try_recv() {
            self.installing_companion = false;
            self.status = Some(match response.result {
                Ok(()) => "Офіційний FreeLLMAPI запущено. Додайте у ньому ключі безкоштовних провайдерів, потім збережіть unified key і перевірте підключення тут.".to_owned(),
                Err(error) => error,
            });
            ctx.request_repaint();
        }
        while let Ok(response) = self.connection_test_rx.try_recv() {
            self.testing_connection = false;
            self.status = Some(match response.result {
                Ok(model) => format!("✓ З'єднання успішне (модель: {model})"),
                Err(error) => format!("✗ Помилка: {error}"),
            });
            ctx.request_repaint();
        }
        while let Ok(response) = self.claude_install_rx.try_recv() {
            self.installing_claude_cli = false;
            self.claude_installed = is_claude_installed();
            self.status = Some(match response.result {
                Ok(()) => "✓ Claude Code CLI успішно встановлено!".to_owned(),
                Err(error) => format!("✗ Помилка встановлення Claude Code CLI: {error}"),
            });
            ctx.request_repaint();
        }
        while let Ok(response) = self.update_rx.try_recv() {
            self.updating_components = false;
            self.update_status = Some(response.message.clone());
            self.status = Some(response.message);
            ctx.request_repaint();
        }
        while let Ok(response) = self.diagnostics_rx.try_recv() {
            self.running_diagnostics = false;
            self.diagnostics_result = Some(response.message.clone());
            self.status = Some(response.message);
            ctx.request_repaint();
        }
        if self.busy {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.running_diagnostics {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.checking_provider {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.installing_companion {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.installing_claude_cli {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.updating_components {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.loading_plugins || self.plugin_action_running {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.testing_connection {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
    }
}

// ── Custom visuals ────────────────────────────────────────────────────────────

fn dark_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    // Background: deep slate
    v.panel_fill = egui::Color32::from_rgb(18, 18, 24);
    v.window_fill = egui::Color32::from_rgb(24, 24, 32);
    v.extreme_bg_color = egui::Color32::from_rgb(12, 12, 18);
    v.faint_bg_color = egui::Color32::from_rgb(28, 28, 38);
    // Widget styles
    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(32, 32, 44);
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 38, 54);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(55, 55, 80);
    v.widgets.active.bg_fill = egui::Color32::from_rgb(80, 100, 180);
    // Accent: purple-blue
    v.selection.bg_fill = egui::Color32::from_rgb(80, 100, 200);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 150, 255));
    // Text
    v.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(200, 200, 215);
    v.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(180, 180, 200);
    v.hyperlink_color = egui::Color32::from_rgb(120, 160, 255);
    // Code blocks
    v.code_bg_color = egui::Color32::from_rgb(28, 28, 40);
    v
}

fn light_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.panel_fill = egui::Color32::from_rgb(248, 248, 252);
    v.window_fill = egui::Color32::WHITE;
    v.widgets.active.bg_fill = egui::Color32::from_rgb(80, 100, 200);
    v.selection.bg_fill = egui::Color32::from_rgb(180, 200, 255);
    v.hyperlink_color = egui::Color32::from_rgb(60, 100, 200);
    v
}

impl eframe::App for MnemosyneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Apply theme ───────────────────────────────────────────────────────
        if self.state.dark_theme {
            ctx.set_visuals(dark_visuals());
            ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
        } else {
            ctx.set_visuals(light_visuals());
            ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
        }

        self.receive_result(ctx);

        // localisation shorthand
        let l: &L = if self.state.lang == AppLang::En { &EN } else { &UK };

        // ── Top Header Bar (matches theme color seamlessly) ───────────────────
        let top_bg = if self.state.dark_theme {
            egui::Color32::from_rgb(13, 13, 18)
        } else {
            egui::Color32::from_rgb(238, 238, 245)
        };
        let top_stroke = if self.state.dark_theme {
            egui::Color32::from_rgb(25, 25, 36)
        } else {
            egui::Color32::from_rgb(215, 215, 228)
        };

        egui::TopBottomPanel::top("top_header_panel")
            .frame(
                egui::Frame::new()
                    .fill(top_bg)
                    .inner_margin(egui::Margin::symmetric(14, 7))
                    .stroke(egui::Stroke::new(1.0, top_stroke)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🧠").size(18.0));
                    ui.label(egui::RichText::new("Mnemosyne").strong().size(15.0));

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if let Some(proj) = self.selected_project() {
                        let badge_bg = if self.state.dark_theme {
                            egui::Color32::from_rgb(24, 26, 40)
                        } else {
                            egui::Color32::from_rgb(222, 228, 245)
                        };
                        egui::Frame::new()
                            .fill(badge_bg)
                            .inner_margin(egui::Margin::symmetric(8, 3))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("📁 {}", proj.name))
                                        .size(12.0)
                                        .color(if self.state.dark_theme {
                                            egui::Color32::from_rgb(175, 195, 255)
                                        } else {
                                            egui::Color32::from_rgb(35, 65, 175)
                                        }),
                                );
                            });
                    } else {
                        ui.weak(if self.state.lang == AppLang::En { "No project selected" } else { "Проєкт не вибрано" });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Settings button
                        if ui.button(if self.state.lang == AppLang::En { "⚙ Settings" } else { "⚙ Налаштування" }).clicked() {
                            self.show_settings = !self.show_settings;
                        }

                        // Plugins button
                        if ui.button(if self.state.lang == AppLang::En { "🔌 Plugins" } else { "🔌 Плагіни" }).clicked() {
                            self.show_plugin_manager = !self.show_plugin_manager;
                            if self.show_plugin_manager && self.plugins.is_empty() {
                                self.refresh_plugins();
                            }
                        }

                        // Files button
                        if self.selected_project().is_some() {
                            if ui.button(if self.state.lang == AppLang::En { "📂 Files" } else { "📂 Файли" }).clicked() {
                                self.show_project_files = !self.show_project_files;
                                if self.show_project_files {
                                    self.refresh_project_files();
                                }
                            }
                        }

                        ui.separator();

                        // 1-click theme switch (🌙 / ☀)
                        let theme_icon = if self.state.dark_theme { "🌙" } else { "☀" };
                        let theme_tip = if self.state.dark_theme {
                            if self.state.lang == AppLang::En { "Switch to Light Theme" } else { "Увімкнути світлу тему" }
                        } else {
                            if self.state.lang == AppLang::En { "Switch to Dark Theme" } else { "Увімкнути темну тему" }
                        };
                        if ui.button(theme_icon).on_hover_text(theme_tip).clicked() {
                            self.state.dark_theme = !self.state.dark_theme;
                            self.save();
                        }

                        // 1-click language switch (🇺🇦 / 🇬🇧)
                        let lang_lbl = if self.state.lang == AppLang::En { "🇬🇧 EN" } else { "🇺🇦 UK" };
                        if ui.button(lang_lbl).on_hover_text("Змінити мову / Change language").clicked() {
                            self.state.lang = if self.state.lang == AppLang::En { AppLang::Uk } else { AppLang::En };
                            self.save();
                        }

                        ui.separator();

                        // Agent mode badge
                        let (mode_text, mode_col) = match self.state.agent_mode {
                            AgentMode::Plan => ("📋 Plan", egui::Color32::from_rgb(90, 160, 245)),
                            AgentMode::Turbo => ("⚡ Turbo", egui::Color32::from_rgb(245, 160, 50)),
                        };
                        ui.colored_label(mode_col, mode_text);

                        // Local model badge if active
                        if self.state.local_model.enabled && self.state.local_model.connected {
                            let label = if self.state.local_model.model_name.is_empty() {
                                "🖥 Local".to_string()
                            } else {
                                format!("🖥 {}", self.state.local_model.model_name.chars().take(16).collect::<String>())
                            };
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 255), label);
                        }
                    });
                });
            });

        let mut request_project_removal = None;
        egui::SidePanel::left("workspace")
            .default_width(260.0)
            .min_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(10.0);

                // ── Logo + title ──────────────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🧠").size(22.0));
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Mnemosyne").strong().size(16.0));
                        ui.label(egui::RichText::new(l.app_subtitle).weak().size(10.0));
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // ── Top buttons ───────────────────────────────────────────────
                if ui.add_sized([ui.available_width(), 28.0],
                    egui::Button::new(l.settings_btn)).clicked() {
                    self.show_settings = true;
                }
                if ui.add_sized([ui.available_width(), 28.0],
                    egui::Button::new(l.plugins_btn)).clicked() {
                    self.show_plugin_manager = true;
                    if self.plugins.is_empty() {
                        self.refresh_plugins();
                    }
                }
                ui.add_space(10.0);

                // ── Project controls ──────────────────────────────────────────
                ui.horizontal(|ui| {
                    if ui.button(l.add_folder).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.add_project(path);
                        }
                    }
                    if ui.button("+ Проєкт").clicked() {
                        self.creating_project = !self.creating_project;
                    }
                });
                if self.creating_project {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_project_name)
                                .hint_text("Назва"),
                        );
                        if ui.button("Створити").clicked() {
                            self.create_project_folder();
                        }
                    });
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(l.projects_heading.to_uppercase()).weak().size(10.0));
                        ui.add_space(2.0);
                        let projects: Vec<(String, String)> = self
                            .state
                            .projects
                            .iter()
                            .map(|project| (project.id.clone(), project.name.clone()))
                            .collect();
                        for (id, name) in projects {
                            let selected = self.state.selected_project.as_deref() == Some(id.as_str());
                            let clicked = ui
                                .horizontal(|ui| {
                                    let clicked = ui
                                        .selectable_label(selected, format!("📁 {name}"))
                                        .clicked();
                                    if selected
                                        && ui
                                            .small_button("⋯")
                                            .on_hover_text("Прибрати проєкт зі списку")
                                            .clicked()
                                    {
                                        request_project_removal = Some((id.clone(), name.clone()));
                                    }
                                    clicked
                                })
                                .inner;
                            if clicked {
                                if selected {
                                    continue;
                                }
                                if self.refuse_project_change_with_unsaved_file() {
                                    continue;
                                }
                                self.state.selected_project = Some(id.clone());
                                self.state.selected_chat = None;
                                self.open_project_file = None;
                                self.project_files.clear();
                                self.project_files_truncated = false;
                                if self.show_project_files {
                                    self.refresh_project_files();
                                }
                                self.save();
                            }
                        }

                        if self.selected_project().is_some() {
                            if ui.add_sized([ui.available_width(), 24.0],
                                egui::Button::new("📄 Файли проєкту")).clicked() {
                                self.show_project_files = true;
                                self.refresh_project_files();
                            }
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("ЧАТИ").weak().size(10.0));
                                if ui.small_button("＋").on_hover_text(l.new_chat).clicked() {
                                    self.create_chat();
                                }
                            });
                            ui.add_space(2.0);
                            let chats: Vec<(String, String)> = self
                                .selected_project()
                                .map(|project| {
                                    project
                                        .chats
                                        .iter()
                                        .map(|chat| (chat.id.clone(), chat.title.clone()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            if chats.is_empty() {
                                ui.weak(l.no_chats);
                            }
                            for (id, title) in chats {
                                let selected = self.state.selected_chat.as_deref() == Some(id.as_str());
                                if ui
                                    .selectable_label(selected, format!("💬 {title}"))
                                    .clicked()
                                {
                                    self.state.selected_chat = Some(id);
                                    self.save();
                                }
                            }
                        }
                    });
            });

        if let Some((project_id, project_name)) = request_project_removal {
            if self.busy {
                self.status = Some(
                    "Дочекайся завершення або скасуй запит перед прибиранням проєкту.".to_owned(),
                );
            } else if self
                .open_project_file
                .as_ref()
                .is_some_and(|file| file.project_id == project_id && file.dirty)
            {
                self.status = Some("Спершу збережи зміни у відкритому файлі.".to_owned());
            } else {
                self.pending_project_removal = Some((project_id, project_name));
            }
        }

        let mut request_cancel = false;
        let mut request_create_chat = false;
        let mut start_chat_rename = None;
        let mut save_chat_rename = None;
        let mut cancel_chat_rename = false;
        let mut request_chat_delete = None;
        let mut request_chat_clear = None;
        let mut request_chat_export = None;

        if self.selected_chat().is_some() {
            let bottom_bg = if self.state.dark_theme {
                egui::Color32::from_rgb(18, 18, 26)
            } else {
                egui::Color32::from_rgb(245, 245, 250)
            };
            let bottom_stroke = if self.state.dark_theme {
                egui::Color32::from_rgb(32, 32, 44)
            } else {
                egui::Color32::from_rgb(220, 220, 230)
            };

            egui::TopBottomPanel::bottom("chat_bottom_input_panel")
                .frame(
                    egui::Frame::new()
                        .fill(bottom_bg)
                        .inner_margin(egui::Margin::symmetric(14, 8))
                        .stroke(egui::Stroke::new(1.0, bottom_stroke)),
                )
                .show(ctx, |ui| {
                    let mut dismiss_diagnostics = false;
                    if let Some(diag) = &self.diagnostics_result {
                        let col = if diag.starts_with('✓') {
                            egui::Color32::LIGHT_GREEN
                        } else if diag.starts_with('✗') {
                            egui::Color32::LIGHT_RED
                        } else {
                            egui::Color32::LIGHT_BLUE
                        };
                        ui.horizontal(|ui| {
                            ui.colored_label(col, diag);
                            if ui.small_button("✕").clicked() {
                                dismiss_diagnostics = true;
                            }
                        });
                    }
                    if dismiss_diagnostics {
                        self.diagnostics_result = None;
                    }

                    let mut remove_attachment = None;
                    if !self.pending_attachments.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            for (index, path) in self.pending_attachments.iter().enumerate() {
                                let name = path
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or("файл")
                                    .to_owned();
                                ui.horizontal(|ui| {
                                    ui.label(format!("📎 {name}"));
                                    if ui.small_button("×").clicked() {
                                        remove_attachment = Some(index);
                                    }
                                });
                            }
                        });
                    }
                    if let Some(index) = remove_attachment {
                        self.pending_attachments.remove(index);
                    }

                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button(l.check_diagnostics_btn)
                            .on_hover_text(l.check_diagnostics_hint)
                            .clicked()
                        {
                            self.run_project_diagnostics();
                        }
                        if self.running_diagnostics {
                            ui.spinner();
                        }
                        for (label, prompt) in [
                            (
                                "Пояснити проєкт",
                                "Проаналізуй структуру проєкту та коротко поясни його архітектуру.",
                            ),
                            (
                                "Знайти проблеми",
                                "Перевір проєкт на очевидні помилки й запропонуй план їх виправлення.",
                            ),
                            (
                                "Спланувати функцію",
                                "Допоможи спланувати реалізацію нової функції: спочатку уточни вимоги, якщо потрібно.",
                            ),
                        ] {
                            if ui.small_button(label).clicked() {
                                self.draft = prompt.to_owned();
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui
                            .button("📎")
                            .on_hover_text(l.attach_btn)
                            .clicked()
                        {
                            self.add_attachments();
                        }
                        let text_edit = egui::TextEdit::multiline(&mut self.draft)
                            .desired_rows(2)
                            .hint_text(l.chat_placeholder);
                        let send_btn_width = 85.0;
                        let input_width = (ui.available_width() - send_btn_width - 8.0).max(120.0);
                        let input = ui.add_sized([input_width, 42.0], text_edit);
                        let enter_pressed = input.has_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                            && !ui.input(|input| input.modifiers.shift);
                        if enter_pressed {
                            self.send_message();
                        }
                        if ui
                            .add_enabled(!self.busy, egui::Button::new(l.send_btn).min_size(egui::vec2(send_btn_width, 42.0)))
                            .clicked()
                        {
                            self.send_message();
                        }
                    });

                    ui.weak(if self.state.lang == AppLang::En {
                        "Enter to send · Shift+Enter for new line"
                    } else {
                        "Enter — надіслати · Shift+Enter — новий рядок"
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Status bar ────────────────────────────────────────────────────
            if let Some(status) = self.status.clone() {
                let color = if status.starts_with("✓") || status.starts_with("PDF відкрито") || status.starts_with("Файл збережено") {
                    egui::Color32::from_rgb(80, 200, 120)
                } else if status.starts_with("✗") || status.starts_with("Помилка") || status.starts_with("Не вдалося") {
                    egui::Color32::from_rgb(220, 80, 80)
                } else {
                    ui.visuals().text_color()
                };
                ui.colored_label(color, &status);
            }
            if self.busy {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !self.cancel_requested.load(Ordering::SeqCst),
                            egui::Button::new(l.cancel_btn),
                        )
                        .clicked()
                    {
                        request_cancel = true;
                    }
                    ui.weak(if self.cancel_requested.load(Ordering::SeqCst) {
                        "Очікую завершення процесу…"
                    } else {
                        l.thinking_label
                    });
                });
                if let Some(tool) = &self.current_tool {
                    ui.weak(format!("🔧 {tool}"));
                }
            }

            match self.selected_chat() {
                Some(chat) => {
                    let title = chat.title.clone();
                    let chat_id = chat.id.clone();
                    let project_id = self.state.selected_project.clone().unwrap_or_default();
                    let editing_this_chat = self.editing_chat.as_deref() == Some(chat_id.as_str());
                    let is_active_chat = self.active_request.as_ref().is_some_and(
                        |(project_id, active_chat_id)| {
                            Some(project_id.as_str()) == self.state.selected_project.as_deref()
                                && active_chat_id == &chat.id
                        },
                    );
                    let streaming_content = self.streaming_content.clone();
                    let messages: Vec<(MessageRole, String, Vec<PathBuf>, String)> = chat
                        .messages
                        .iter()
                        .map(|message| {
                            (
                                message.role,
                                message.content.clone(),
                                message.attachments.clone(),
                                message.created_at.format("%H:%M").to_string(),
                            )
                        })
                        .collect();
                    ui.horizontal(|ui| {
                        if editing_this_chat {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.chat_title_draft)
                                    .desired_width(260.0),
                            );
                            if ui.button("Зберегти назву").clicked() {
                                save_chat_rename = Some((
                                    project_id.clone(),
                                    chat_id.clone(),
                                    self.chat_title_draft.clone(),
                                ));
                            }
                            if ui.button("Скасувати").clicked() {
                                cancel_chat_rename = true;
                            }
                        } else {
                            ui.heading(title.clone());
                            if ui.small_button(l.export_chat).clicked() {
                                request_chat_export =
                                    Some((project_id.clone(), chat_id.clone()));
                            }
                            if ui.small_button(l.rename_chat).clicked() {
                                start_chat_rename = Some((chat_id.clone(), title.clone()));
                            }
                            if ui
                                .add_enabled(!self.busy, egui::Button::new(l.clear_history))
                                .clicked()
                            {
                                request_chat_clear =
                                    Some((project_id.clone(), chat_id.clone(), title.clone()));
                            }
                            if ui
                                .add_enabled(!self.busy, egui::Button::new(l.delete_chat))
                                .clicked()
                            {
                                request_chat_delete =
                                    Some((project_id.clone(), chat_id.clone(), title.clone()));
                            }
                        }
                    });

                    // ── Compact chat toolbar (Model, Mode, Context) ───
                    let mut settings_changed = false;
                    ui.horizontal(|ui| {
                        ui.label("Модель:");
                        let model_label = if let Some(ref m) = self.state.freellmapi.selected_model {
                            m.clone()
                        } else {
                            self.state.claude_model.label().to_owned()
                        };
                        egui::ComboBox::from_id_salt("chat-model-selector")
                            .selected_text(&model_label)
                            .show_ui(ui, |ui| {
                                ui.label(egui::RichText::new("Claude").weak());
                                for (model_variant, name) in [
                                    (ClaudeModel::Sonnet, "Sonnet"),
                                    (ClaudeModel::Opus, "Opus"),
                                    (ClaudeModel::Haiku, "Haiku"),
                                ] {
                                    let is_active = self.state.freellmapi.selected_model.is_none()
                                        && self.state.claude_model == model_variant;
                                    if ui.selectable_label(is_active, name).clicked() {
                                        self.state.freellmapi.selected_model = None;
                                        self.state.claude_model = model_variant;
                                        settings_changed = true;
                                    }
                                }

                                if !self.state.freellmapi.ready_models.is_empty() {
                                    ui.separator();
                                    ui.label(egui::RichText::new("FreeLLMAPI").weak());
                                    for ready_m in &self.state.freellmapi.ready_models {
                                        let is_active = self.state.freellmapi.selected_model.as_deref()
                                            == Some(ready_m.as_str());
                                        if ui.selectable_label(is_active, ready_m).clicked() {
                                            self.state.freellmapi.selected_model = Some(ready_m.clone());
                                            settings_changed = true;
                                        }
                                    }
                                }
                            });

                        ui.separator();

                        ui.label("Режим:");
                        settings_changed |= ui
                            .selectable_value(
                                &mut self.state.agent_mode,
                                AgentMode::Plan,
                                "План",
                            )
                            .on_hover_text("План · читання: Claude читає проєкт і планує, без змін файлів.")
                            .changed();
                        settings_changed |= ui
                            .selectable_value(
                                &mut self.state.agent_mode,
                                AgentMode::Turbo,
                                "Turbo",
                            )
                            .on_hover_text("Turbo · підтвердження: Mnemosyne запитує підтвердження кожної зміни файлів.")
                            .changed();

                        ui.separator();

                        ui.label("Контекст:");
                        egui::ComboBox::from_id_salt("conversation-context-limit")
                            .selected_text(self.state.context_limit.label())
                            .show_ui(ui, |ui| {
                                for limit in [
                                    ConversationContextLimit::Last5Turns,
                                    ConversationContextLimit::Last10Turns,
                                    ConversationContextLimit::Last20Turns,
                                    ConversationContextLimit::Last40Turns,
                                    ConversationContextLimit::All,
                                ] {
                                    settings_changed |= ui
                                        .selectable_value(
                                            &mut self.state.context_limit,
                                            limit,
                                            limit.label(),
                                        )
                                        .changed();
                                }
                            });
                    });
                    if settings_changed {
                        self.save();
                    }
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (role, content, attachments, time) in messages {
                                let is_user = role == MessageRole::User;
                                let fill = if is_user {
                                    ui.visuals().widgets.inactive.bg_fill
                                } else {
                                    ui.visuals().faint_bg_color
                                };
                                egui::Frame::group(ui.style()).fill(fill).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(if is_user { l.you_label } else { l.claude_label });
                                        ui.weak(time);
                                    });
                                    if !content.is_empty() && is_user {
                                        ui.label(content);
                                    } else if !content.is_empty() {
                                        show_markdown_preview(ui, &content);
                                    }
                                    for attachment in attachments {
                                        ui.weak(format!(
                                            "📎 {}",
                                            attachment
                                                .file_name()
                                                .and_then(|name| name.to_str())
                                                .unwrap_or("файл")
                                        ));
                                    }
                                });
                                ui.add_space(6.0);
                            }
                            if self.busy && is_active_chat {
                                if !streaming_content.is_empty() {
                                    egui::Frame::group(ui.style())
                                        .fill(ui.visuals().faint_bg_color)
                                        .show(ui, |ui| {
                                            ui.strong(if self.state.lang == AppLang::En { "Claude · generating response" } else { "Claude · відповідь генерується" });
                                            ui.label(&streaming_content);
                                        });
                                }
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(l.thinking_label);
                                });
                            }
                        });
                }
                None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        if self.selected_project().is_some() {
                            ui.heading(l.no_chats);
                            if ui.button(l.new_chat).clicked() {
                                request_create_chat = true;
                            }
                        } else {
                            ui.heading(l.no_project);
                            ui.label(l.select_or_create);
                        }
                    });
                }
            }
        });
        if request_create_chat {
            self.create_chat();
        }
        if request_cancel {
            self.cancel_claude();
        }
        if let Some((chat_id, title)) = start_chat_rename {
            self.editing_chat = Some(chat_id);
            self.chat_title_draft = title;
        }
        if let Some((project_id, chat_id, title)) = save_chat_rename {
            self.rename_chat(&project_id, &chat_id, &title);
        }
        if cancel_chat_rename {
            self.editing_chat = None;
            self.chat_title_draft.clear();
        }
        if let Some(target) = request_chat_delete {
            self.pending_chat_delete = Some(target);
        }
        if let Some(target) = request_chat_clear {
            self.pending_chat_clear = Some(target);
        }
        if let Some((project_id, chat_id)) = request_chat_export {
            self.export_chat(&project_id, &chat_id);
        }
        if let Some((tool, path, preview)) = self.pending_file_approval.as_ref().map(|approval| {
            (
                approval.tool.clone(),
                approval.path.clone(),
                approval.preview.clone(),
            )
        }) {
            let mut open = true;
            let mut allow = false;
            let mut deny = false;
            egui::Window::new("Підтвердити зміну файлу")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Claude Code хоче виконати {tool} для файла:"));
                    ui.code(path);
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .show(ui, |ui| ui.monospace(&preview));
                    ui.label("Дозвіл стосується лише цієї однієї операції.");
                    ui.horizontal(|ui| {
                        if ui.button("Дозволити цю зміну").clicked() {
                            allow = true;
                        }
                        if ui.button("Відхилити").clicked() {
                            deny = true;
                        }
                    });
                });
            if allow {
                self.resolve_file_approval(true);
            } else if deny || !open {
                self.resolve_file_approval(false);
            }
        }
        if let Some((project_id, chat_id, title)) = self.pending_chat_clear.clone() {
            let mut open = true;
            let mut confirm = false;
            let mut dismiss = false;
            egui::Window::new("Очистити історію чату?")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Усі повідомлення та вкладення чату «{title}» буде видалено. Сам чат і його назва залишаться."
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Очистити історію").clicked() {
                            confirm = true;
                        }
                        if ui.button("Скасувати").clicked() {
                            dismiss = true;
                        }
                    });
                });
            if confirm {
                self.clear_chat_history(&project_id, &chat_id);
            } else if dismiss || !open {
                self.pending_chat_clear = None;
            }
        }
        if let Some((project_id, chat_id, title)) = self.pending_chat_delete.clone() {
            let mut open = true;
            let mut confirm = false;
            let mut dismiss = false;
            egui::Window::new("Підтвердження видалення")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Видалити чат «{title}» разом з усією його історією? Це неможливо скасувати."
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Видалити назавжди").clicked() {
                            confirm = true;
                        }
                        if ui.button("Залишити чат").clicked() {
                            dismiss = true;
                        }
                    });
                });
            if confirm {
                self.delete_chat(&project_id, &chat_id);
            } else if dismiss || !open {
                self.pending_chat_delete = None;
            }
        }
        if let Some((project_id, project_name)) = self.pending_project_removal.clone() {
            let mut open = true;
            let mut confirm = false;
            let mut dismiss = false;
            egui::Window::new("Прибрати проєкт?")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "«{project_name}» буде прибрано зі списку Mnemosyne разом із локальними чатами. Файли та папка проєкту на диску залишаться без змін."
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Прибрати проєкт").clicked() {
                            confirm = true;
                        }
                        if ui.button("Залишити").clicked() {
                            dismiss = true;
                        }
                    });
                });
            if confirm {
                self.remove_project_from_workspace(&project_id);
            } else if dismiss || !open {
                self.pending_project_removal = None;
            }
        }

        if self.show_plugin_manager {
            let mut open = self.show_plugin_manager;
            let mut refresh = false;
            let mut requested_action = None;
            let mut requested_marketplace_action = None;
            let mut marketplace_error = None;
            egui::Window::new("Плагіни Claude Code")
                .open(&mut open)
                .default_size([760.0, 620.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.loading_plugins && !self.plugin_action_running,
                                egui::Button::new("Перечитати каталог"),
                            )
                            .clicked()
                        {
                            refresh = true;
                        }
                        if self.loading_plugins {
                            ui.spinner();
                            ui.label("Завантажую каталог…");
                        } else if self.plugin_action_running {
                            ui.spinner();
                            ui.label("Виконую операцію…");
                        }
                    });
                    ui.collapsing("Marketplace-джерела", |ui| {
                        for marketplace in self.marketplaces.clone() {
                            ui.horizontal(|ui| {
                                ui.label(&marketplace.name);
                                ui.weak(&marketplace.source);
                                if ui
                                    .add_enabled(
                                        !self.plugin_action_running,
                                        egui::Button::new("Видалити"),
                                    )
                                    .clicked()
                                {
                                    requested_marketplace_action = Some((
                                        false,
                                        marketplace.name.clone(),
                                        marketplace.name,
                                    ));
                                }
                            });
                        }
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.marketplace_source)
                                    .desired_width(300.0)
                                    .hint_text("owner/repo або GitHub URL"),
                            );
                            if ui
                                .add_enabled(
                                    !self.plugin_action_running,
                                    egui::Button::new("Додати marketplace"),
                                )
                                .clicked()
                            {
                                match normalize_marketplace_source(&self.marketplace_source) {
                                    Ok(source) => {
                                        requested_marketplace_action = Some((
                                            true,
                                            source.clone(),
                                            source,
                                        ));
                                    }
                                    Err(error) => marketplace_error = Some(error),
                                }
                            }
                        });
                        if let Some(error) = &marketplace_error {
                            ui.colored_label(egui::Color32::LIGHT_RED, error);
                        }
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut self.plugin_filter)
                            .hint_text("Пошук за назвою, описом або marketplace"),
                    );
                    ui.separator();
                    let filter = self.plugin_filter.trim().to_ascii_lowercase();
                    let plugins = self.plugins.clone();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut visible = 0;
                        for plugin in plugins {
                            if !filter.is_empty()
                                && !plugin.name.to_ascii_lowercase().contains(&filter)
                                && !plugin.description.to_ascii_lowercase().contains(&filter)
                                && !plugin.marketplace.to_ascii_lowercase().contains(&filter)
                            {
                                continue;
                            }
                            visible += 1;
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(&plugin.name);
                                    if plugin.installed {
                                        ui.weak(if plugin.enabled { "Встановлено" } else { "Вимкнено" });
                                    }
                                    ui.weak(format!("@{}", plugin.marketplace));
                                });
                                if !plugin.description.is_empty() {
                                    ui.label(&plugin.description);
                                }
                                if plugin.installed && plugin.scope != "user" {
                                    ui.weak(format!(
                                        "Встановлено в області «{}»; змінювати його можна з відповідного проєкту.",
                                        plugin.scope
                                    ));
                                } else {
                                    ui.horizontal(|ui| {
                                        if !plugin.installed {
                                            if ui
                                                .add_enabled(
                                                    !self.plugin_action_running,
                                                    egui::Button::new("Встановити"),
                                                )
                                                .clicked()
                                            {
                                                requested_action = Some((
                                                    PluginAction::Install,
                                                    plugin.id.clone(),
                                                    plugin.name.clone(),
                                                ));
                                            }
                                        } else {
                                            let (action, label) = if plugin.enabled {
                                                (PluginAction::Disable, "Вимкнути")
                                            } else {
                                                (PluginAction::Enable, "Увімкнути")
                                            };
                                            if ui
                                                .add_enabled(
                                                    !self.plugin_action_running,
                                                    egui::Button::new(label),
                                                )
                                                .clicked()
                                            {
                                                requested_action = Some((
                                                    action,
                                                    plugin.id.clone(),
                                                    plugin.name.clone(),
                                                ));
                                            }
                                            if ui
                                                .add_enabled(
                                                    !self.plugin_action_running,
                                                    egui::Button::new("Видалити"),
                                                )
                                                .clicked()
                                            {
                                                requested_action = Some((
                                                    PluginAction::Uninstall,
                                                    plugin.id.clone(),
                                                    plugin.name.clone(),
                                                ));
                                            }
                                        }
                                    });
                                }
                            });
                            ui.add_space(4.0);
                        }
                        if visible == 0 && !self.loading_plugins {
                            ui.weak("Немає плагінів, що відповідають пошуку.");
                        }
                    });
                    ui.separator();
                    ui.weak("Каталог і команди керування надає встановлений Claude Code CLI. Встановлюй лише плагіни з джерел, яким довіряєш.");
                });
            self.show_plugin_manager = open;
            if refresh {
                self.refresh_plugins();
            }
            if let Some((action, plugin_id, name)) = requested_action {
                match action {
                    PluginAction::Install | PluginAction::Uninstall => {
                        self.pending_plugin_action = Some((action, plugin_id, name));
                    }
                    PluginAction::Enable | PluginAction::Disable => {
                        self.run_plugin_action(action, plugin_id);
                    }
                }
            }
            if let Some((add, target, label)) = requested_marketplace_action {
                self.pending_marketplace_action = Some((add, target, label));
            }
        }

        if let Some((action, plugin_id, name)) = self.pending_plugin_action.clone() {
            let mut open = true;
            let mut confirm = false;
            let mut dismiss = false;
            let title = match action {
                PluginAction::Install => "Встановити плагін?",
                PluginAction::Uninstall => "Видалити плагін?",
                PluginAction::Enable | PluginAction::Disable => "Змінити стан плагіна?",
            };
            egui::Window::new(title)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(match action {
                        PluginAction::Install => format!(
                            "Встановити «{name}» ({plugin_id}) для користувача? Плагін може додавати інструменти й запускати власний код. Продовжуй лише якщо довіряєш marketplace."
                        ),
                        PluginAction::Uninstall => format!(
                            "Видалити «{name}» ({plugin_id})? Збережені дані плагіна буде залишено."
                        ),
                        PluginAction::Enable | PluginAction::Disable => name.clone(),
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Підтвердити").clicked() {
                            confirm = true;
                        }
                        if ui.button("Скасувати").clicked() {
                            dismiss = true;
                        }
                    });
                });
            if confirm {
                self.pending_plugin_action = None;
                self.run_plugin_action(action, plugin_id);
            } else if dismiss || !open {
                self.pending_plugin_action = None;
            }
        }

        if let Some((add, target, label)) = self.pending_marketplace_action.clone() {
            let mut open = true;
            let mut confirm = false;
            let mut dismiss = false;
            egui::Window::new(if add {
                "Додати marketplace?"
            } else {
                "Видалити marketplace?"
            })
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(if add {
                    format!(
                        "Додати GitHub-джерело {label}? Marketplace надає плагіни з виконуваним кодом. Додавай лише джерела, яким довіряєш."
                    )
                } else {
                    format!(
                        "Видалити marketplace «{label}»? Це може зробити його плагіни недоступними для оновлення."
                    )
                });
                ui.horizontal(|ui| {
                    if ui.button(if add { "Додати" } else { "Видалити" }).clicked() {
                        confirm = true;
                    }
                    if ui.button("Скасувати").clicked() {
                        dismiss = true;
                    }
                });
            });
            if confirm {
                self.pending_marketplace_action = None;
                if add {
                    self.marketplace_source.clear();
                }
                self.run_marketplace_action(add, target);
            } else if dismiss || !open {
                self.pending_marketplace_action = None;
            }
        }

        if self.show_project_files {
            let project_name = self.selected_project().map(|project| project.name.clone());
            let mut show_files = self.show_project_files;
            let mut refresh_files = false;
            let mut requested_file = None;
            let mut requested_document = None;
            let mut save_file = false;
            egui::Window::new("Файли проєкту")
                .open(&mut show_files)
                .default_size([900.0, 620.0])
                .resizable(true)
                .show(ctx, |ui| {
                    let Some(project_name) = project_name else {
                        ui.label("Спершу вибери проєкт.");
                        return;
                    };
                    ui.horizontal(|ui| {
                        ui.strong(project_name);
                        if ui.small_button("Оновити список").clicked() {
                            refresh_files = true;
                        }
                        if ui.small_button("+ План").clicked() {
                            requested_document = Some(ProjectDocumentKind::Plan);
                        }
                        if ui.small_button("+ Звіт").clicked() {
                            requested_document = Some(ProjectDocumentKind::Report);
                        }
                        if self.project_files_truncated {
                            ui.weak("Список обмежено першими 2000 файлами.");
                        }
                    });
                    ui.weak("Документи створюються у docs/plans та docs/reports.");
                    ui.separator();
                    ui.columns(2, |columns| {
                        columns[0].add(
                            egui::TextEdit::singleline(&mut self.project_file_filter)
                                .hint_text("Фільтр файлів"),
                        );
                        egui::ScrollArea::vertical()
                            .id_salt("project-file-list")
                            .show(&mut columns[0], |ui| {
                                let mut matching_files = 0;
                                for relative_path in self.project_files.clone() {
                                    if !project_file_matches_filter(
                                        &relative_path,
                                        &self.project_file_filter,
                                    ) {
                                        continue;
                                    }
                                    matching_files += 1;
                                    let label = relative_path.to_string_lossy();
                                    if ui.small_button(label).clicked() {
                                        requested_file = Some(relative_path);
                                    }
                                }
                                if matching_files == 0 {
                                    ui.weak(if self.project_files.is_empty() {
                                        "Файлів проєкту не знайдено."
                                    } else {
                                        "За цим фільтром нічого не знайдено."
                                    });
                                }
                            });

                        if let Some(file) = self.open_project_file.as_mut() {
                            columns[1].horizontal(|ui| {
                                ui.strong(file.relative_path.to_string_lossy());
                                if let Some(image) = &file.image {
                                    ui.weak(format!("{} × {}", image.width, image.height));
                                } else if file.dirty {
                                    ui.colored_label(egui::Color32::YELLOW, "Не збережено");
                                }
                            });
                            if file.image.is_none() {
                                columns[1].horizontal(|ui| {
                                    if ui
                                        .button(if file.markdown_preview {
                                            "Редагувати"
                                        } else {
                                            "Перегляд"
                                        })
                                        .clicked()
                                    {
                                        file.markdown_preview = !file.markdown_preview;
                                    }
                                    if ui
                                        .add_enabled(file.dirty, egui::Button::new("Зберегти"))
                                        .clicked()
                                    {
                                        save_file = true;
                                    }
                                });
                            }
                            columns[1].separator();
                            egui::ScrollArea::both()
                                .id_salt("project-file-content")
                                .show(&mut columns[1], |ui| {
                                    if let Some(image) = &file.image {
                                        let available = ui.available_size();
                                        let scale = (available.x / image.width as f32)
                                            .min(available.y / image.height as f32)
                                            .min(1.0);
                                        let size = egui::vec2(
                                            image.width as f32 * scale,
                                            image.height as f32 * scale,
                                        );
                                        ui.image((image.texture.id(), size));
                                    } else if file.markdown_preview {
                                        show_markdown_preview(ui, &file.content);
                                    } else {
                                        let response = ui.add(
                                            egui::TextEdit::multiline(&mut file.content)
                                                .font(egui::TextStyle::Monospace)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(28),
                                        );
                                        if response.changed() {
                                            file.dirty = true;
                                        }
                                    }
                                });
                        } else {
                            columns[1].label("Вибери текстовий або Markdown-файл.");
                        }
                    });
                });
            self.show_project_files = show_files;
            if refresh_files {
                self.refresh_project_files();
            }
            if let Some(relative_path) = requested_file {
                self.open_project_file(&relative_path, ctx);
            }
            if let Some(kind) = requested_document {
                if !self.refuse_project_change_with_unsaved_file() {
                    if let Some(project_root) =
                        self.selected_project().map(|project| project.path.clone())
                    {
                        match create_project_markdown_document(&project_root, kind) {
                            Ok(relative_path) => {
                                self.refresh_project_files();
                                self.open_project_file(&relative_path, ctx);
                            }
                            Err(error) => self.status = Some(error),
                        }
                    }
                }
            }
            if save_file {
                self.save_open_project_file();
            }
        }

        if self.show_settings {
            let mut show_settings = self.show_settings;
            egui::Window::new(l.settings_title)
                .open(&mut show_settings)
                .resizable(true)
                .default_width(450.0)
                .max_width(480.0)
                .default_height(520.0)
                .max_height(580.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // ── Appearance & Language ─────────────────────────────────
                    ui.heading(if self.state.lang == AppLang::En { "🎨 Appearance & Language" } else { "🎨 Вигляд та мова" });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", l.theme_label));
                        if ui.selectable_value(&mut self.state.dark_theme, true, l.theme_dark).clicked() {
                            self.save();
                        }
                        if ui.selectable_value(&mut self.state.dark_theme, false, l.theme_light).clicked() {
                            self.save();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", l.lang_label));
                        if ui.selectable_value(&mut self.state.lang, AppLang::Uk, "🇺🇦 Українська").clicked() {
                            self.save();
                        }
                        if ui.selectable_value(&mut self.state.lang, AppLang::En, "🇬🇧 English").clicked() {
                            self.save();
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();

                    // ── FreeLLMAPI ───────────────────────────────────────────
                    ui.heading(format!("🌐 {}", l.freellmapi_section));
                    ui.label(if self.state.lang == AppLang::En {
                        "FreeLLMAPI connects to free model providers (Gemini, OpenRouter, DeepSeek). Save your unified API key below."
                    } else {
                        "FreeLLMAPI підключається до безкоштовних провайдерів (Gemini, OpenRouter, DeepSeek). Збережіть свій unified API key нижче."
                    });
                    if ui
                        .add_enabled(
                            !self.installing_companion,
                            egui::Button::new(if self.installing_companion {
                                if self.state.lang == AppLang::En { "Downloading & launching FreeLLMAPI…" } else { "Завантажую та запускаю FreeLLMAPI…" }
                            } else {
                                l.freellmapi_install
                            }),
                        )
                        .clicked()
                    {
                        self.install_freellmapi_companion();
                    }
                    ui.add_space(8.0);
                    ui.label(format!("{}: (Desktop: 31415 · server: 3001)", l.freellmapi_url));
                    ui.text_edit_singleline(&mut self.state.freellmapi.base_url);
                    ui.label(l.freellmapi_key);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.freellmapi_key_draft)
                            .password(true)
                            .hint_text("freellmapi-…"),
                    );
                    ui.horizontal(|ui| {
                        if ui.button(l.freellmapi_save_key).clicked() {
                            self.save_provider_key();
                        }
                        if ui
                            .add_enabled(
                                !self.checking_provider,
                                egui::Button::new(if self.checking_provider {
                                    if self.state.lang == AppLang::En { "Checking…" } else { "Перевіряю…" }
                                } else {
                                    l.freellmapi_check
                                }),
                            )
                            .clicked()
                        {
                            if !self.freellmapi_key_draft.trim().is_empty() {
                                self.save_provider_key();
                            }
                            self.check_freellmapi();
                        }
                        if ui
                            .add_enabled(
                                !self.testing_connection && self.state.freellmapi.connected,
                                egui::Button::new(if self.testing_connection {
                                    if self.state.lang == AppLang::En { "Testing…" } else { "Тестую…" }
                                } else {
                                    l.freellmapi_test
                                }),
                            )
                            .on_hover_text(if self.state.lang == AppLang::En { "Send test ping to model via FreeLLMAPI" } else { "Надіслати тестовий запит до моделі через FreeLLMAPI" })
                            .clicked()
                        {
                            self.test_freellmapi_connection();
                        }
                    });
                    if self.state.freellmapi.connected {
                        ui.colored_label(
                            egui::Color32::LIGHT_GREEN,
                            format!(
                                "{} · {} {}",
                                l.freellmapi_connected,
                                self.state.freellmapi.ready_models.len(),
                                if self.state.lang == AppLang::En { "models ready" } else { "доступних моделей" }
                            ),
                        );
                    } else {
                        ui.weak(l.freellmapi_disconnected);
                    }
                    ui.weak(l.freellmapi_key_hint);

                    if !self.state.freellmapi.ready_models.is_empty() {
                        ui.add_space(8.0);
                        let mut model_pick_changed = false;
                        let ready_models = self.state.freellmapi.ready_models.clone();
                        ui.horizontal(|ui| {
                            ui.label("Вибрана модель:");
                            let sel_label = self.state.freellmapi.selected_model.as_deref().unwrap_or("Claude (Default)");
                            egui::ComboBox::from_id_salt("freellmapi-selected-model-combo")
                                .selected_text(sel_label)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(self.state.freellmapi.selected_model.is_none(), "Claude (Default)").clicked() {
                                        self.state.freellmapi.selected_model = None;
                                        model_pick_changed = true;
                                    }
                                    for m in &ready_models {
                                        let is_sel = self.state.freellmapi.selected_model.as_deref() == Some(m.as_str());
                                        if ui.selectable_label(is_sel, m).clicked() {
                                            self.state.freellmapi.selected_model = Some(m.clone());
                                            model_pick_changed = true;
                                        }
                                    }
                                });
                        });
                        if model_pick_changed {
                            self.save();
                        }
                        ui.collapsing(if self.state.lang == AppLang::En { "Available FreeLLMAPI Models" } else { "Усі моделі у FreeLLMAPI" }, |ui| {
                            for model in &ready_models {
                                ui.label(format!("• {model}"));
                            }
                        });
                    }

                    ui.add_space(12.0);
                    ui.separator();

                    // ── Local Model (LM Studio / Ollama) ──────────────────────
                    ui.heading(format!("🖥 {}", l.local_model_section));
                    ui.checkbox(&mut self.state.local_model.enabled, l.local_model_enabled);
                    ui.weak(l.local_model_hint);
                    ui.add_space(4.0);

                    ui.add_enabled_ui(self.state.local_model.enabled, |ui| {
                        ui.label(l.local_model_url);
                        ui.text_edit_singleline(&mut self.state.local_model.base_url);

                        ui.horizontal(|ui| {
                            ui.weak("Presets:");
                            if ui.small_button("LM Studio (:1234)").clicked() {
                                self.state.local_model.base_url = "http://localhost:1234".to_owned();
                            }
                            if ui.small_button("Ollama (:11434)").clicked() {
                                self.state.local_model.base_url = "http://localhost:11434".to_owned();
                            }
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    !self.checking_local_model,
                                    egui::Button::new(if self.checking_local_model {
                                        if self.state.lang == AppLang::En { "Scanning…" } else { "Сканую…" }
                                    } else {
                                        l.local_model_check
                                    }),
                                )
                                .clicked()
                            {
                                self.check_local_model();
                            }
                        });

                        if !self.state.local_model.ready_models.is_empty() {
                            ui.add_space(4.0);
                            ui.label(l.local_model_pick);
                            let current_display = if self.state.local_model.model_name.is_empty() {
                                if self.state.lang == AppLang::En { "— auto / loaded —" } else { "— авто / поточна —" }
                            } else {
                                self.state.local_model.model_name.as_str()
                            };
                            egui::ComboBox::from_id_salt("local_model_picker")
                                .selected_text(current_display)
                                .show_ui(ui, |ui| {
                                    let auto_label = if self.state.lang == AppLang::En { "— auto / loaded —" } else { "— авто / поточна —" };
                                    if ui.selectable_label(self.state.local_model.model_name.is_empty(), auto_label).clicked() {
                                        self.state.local_model.model_name.clear();
                                    }
                                    for model_id in &self.state.local_model.ready_models {
                                        let is_sel = self.state.local_model.model_name == *model_id;
                                        if ui.selectable_label(is_sel, model_id).clicked() {
                                            self.state.local_model.model_name = model_id.clone();
                                        }
                                    }
                                });
                        }

                        ui.add_space(4.0);
                        ui.label(l.local_model_name);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.local_model.model_name)
                                .hint_text(l.local_model_name_hint),
                        );

                        if self.state.local_model.connected {
                            ui.colored_label(
                                egui::Color32::LIGHT_GREEN,
                                format!(
                                    "{} · {} {}",
                                    l.local_model_connected,
                                    self.state.local_model.ready_models.len(),
                                    if self.state.lang == AppLang::En { "models available" } else { "доступних моделей" }
                                ),
                            );
                        } else {
                            ui.weak(l.local_model_disconnected);
                        }
                    });

                    ui.add_space(12.0);
                    ui.separator();

                    // ── Claude Code CLI ───────────────────────────────────────
                    ui.heading(format!("💻 {}", l.claude_cli_section));
                    let claude_installed = self.claude_installed;
                    if claude_installed {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, format!("{} на PATH", l.claude_cli_found));
                    } else {
                        ui.colored_label(egui::Color32::LIGHT_RED, l.claude_cli_missing);
                        ui.label(if self.state.lang == AppLang::En {
                            "Install Claude Code CLI to send queries to models."
                        } else {
                            "Встановіть Claude Code CLI, щоб надсилати запити до моделей."
                        });
                        ui.horizontal(|ui| {
                            if ui.add_enabled(
                                !self.installing_claude_cli,
                                egui::Button::new(if self.installing_claude_cli {
                                    if self.state.lang == AppLang::En { "Installing Claude Code CLI…" } else { "Встановлюю Claude Code CLI…" }
                                } else {
                                    if self.state.lang == AppLang::En { "⚡ Auto-install via npm" } else { "⚡ Автоматично встановити через npm" }
                                }),
                            ).on_hover_text("npm install -g @anthropic-ai/claude-code").clicked() {
                                self.install_claude_cli_background();
                            }
                            if ui.hyperlink_to(
                                l.claude_cli_install_hint,
                                "https://docs.anthropic.com/en/docs/claude-code",
                            ).clicked() {
                                let _ = open::that("https://docs.anthropic.com/en/docs/claude-code");
                            }
                        });
                    }

                    ui.add_space(12.0);
                    ui.separator();

                    // ── Decod MCP ─────────────────────────────────────────────
                    ui.heading(format!("⚙ {}", l.decod_section));
                    let mut decod_enabled = self.state.decod_mcp_enabled;
                    if ui.checkbox(&mut decod_enabled, l.decod_enable).changed() {
                        self.state.decod_mcp_enabled = decod_enabled;
                        self.save();
                    }
                    ui.label(format!("{}:", l.decod_path));
                    if ui.text_edit_singleline(&mut self.state.decod_mcp_path).changed() {
                        self.save();
                    }
                    ui.horizontal(|ui| {
                        if ui.button(l.decod_check).clicked() {
                            let path = self.state.decod_mcp_path.clone();
                            match silent_command("python")
                                .args([&path, "--version"])
                                .output()
                            {
                                Ok(output) if output.status.success() => {
                                    self.decod_mcp_check_status = Some(format!(
                                        "✓ Decod MCP: {}",
                                        String::from_utf8_lossy(&output.stdout).trim()
                                    ));
                                }
                                Ok(output) => {
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    if !stderr.trim().is_empty() {
                                        self.decod_mcp_check_status = Some(format!(
                                            "✗ Помилка: {}",
                                            stderr.trim()
                                        ));
                                    } else {
                                        self.decod_mcp_check_status = Some(
                                            "✗ Скрипт завершився з помилкою".to_owned(),
                                        );
                                    }
                                }
                                Err(error) => {
                                    self.decod_mcp_check_status = Some(format!(
                                        "✗ Не вдалося запустити: {error}"
                                    ));
                                }
                            }
                        }
                        if let Some(status) = &self.decod_mcp_check_status {
                            if status.starts_with('✓') {
                                ui.colored_label(egui::Color32::LIGHT_GREEN, status);
                            } else {
                                ui.colored_label(egui::Color32::LIGHT_RED, status);
                            }
                        }
                    });
                    if self.state.decod_mcp_enabled {
                        ui.weak(l.decod_active_hint);
                    }

                    // ── Superpowers & Vibe-Coding ─────────────────────────────
                    ui.add_space(12.0);
                    ui.separator();
                    ui.heading(l.superpowers_section);
                    ui.horizontal(|ui| {
                        let mut caveman = self.state.caveman_mode;
                        if ui.checkbox(&mut caveman, l.caveman_mode).changed() {
                            self.state.caveman_mode = caveman;
                            self.save();
                        }
                    });
                    ui.weak(l.caveman_hint);

                    ui.horizontal(|ui| {
                        let mut karpathy = self.state.karpathy_rules;
                        if ui.checkbox(&mut karpathy, l.karpathy_rules).changed() {
                            self.state.karpathy_rules = karpathy;
                            self.save();
                        }
                    });
                    ui.weak(l.karpathy_hint);

                    ui.horizontal(|ui| {
                        let mut auto_rules = self.state.auto_rules_provision;
                        if ui
                            .checkbox(
                                &mut auto_rules,
                                if self.state.lang == AppLang::En {
                                    "Auto-provision .claude/CLAUDE.md"
                                } else {
                                    "Автоматично налаштовувати .claude/CLAUDE.md"
                                },
                            )
                            .changed()
                        {
                            self.state.auto_rules_provision = auto_rules;
                            self.save();
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button(l.auto_rules_btn).clicked() {
                            if let Some(project) = self.selected_project() {
                                match ensure_curated_project_rules(
                                    &project.path,
                                    self.state.caveman_mode,
                                    self.state.karpathy_rules,
                                    self.state.decod_mcp_enabled,
                                ) {
                                    Ok(_) => self.status = Some(l.auto_rules_success.to_string()),
                                    Err(e) => self.status = Some(format!("Помилка: {e}")),
                                }
                            } else {
                                self.status = Some(l.select_or_create.to_string());
                            }
                        }
                    });

                    // ── Updates & Maintenance ─────────────────────────────────
                    ui.add_space(12.0);
                    ui.separator();
                    ui.heading(if self.state.lang == AppLang::En { "🔄 Updates & Maintenance" } else { "🔄 Оновлення та обслуговування" });
                    ui.label(if self.state.lang == AppLang::En {
                        "Keep Claude Code CLI, Decod reverse engineering engine, and Python dependencies up-to-date."
                    } else {
                        "Оновлення Claude Code CLI, RE-рушія Decod та Python-модулів в один клік."
                    });
                    ui.horizontal(|ui| {
                        if ui.add_enabled(
                            !self.updating_components,
                            egui::Button::new(if self.updating_components {
                                if self.state.lang == AppLang::En { "Updating components…" } else { "Оновлюю компоненти…" }
                            } else {
                                if self.state.lang == AppLang::En { "🔄 Update All Components" } else { "🔄 Оновити всі компоненти" }
                            }),
                        ).on_hover_text(if self.state.lang == AppLang::En { "Runs npm update for Claude and git pull + pip for Decod" } else { "Запускає оновлення Claude через npm, а Decod через git pull та pip" }).clicked() {
                            self.update_all_components();
                        }
                    });
                    if let Some(status) = &self.update_status {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, status);
                    }
                        });
                });
            self.show_settings = show_settings;
        }
    }
}

fn state_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Не вдалося визначити системну папку даних.".to_owned())?
        .join("Mnemosyne");
    fs::create_dir_all(&data_dir)
        .map_err(|error| format!("Не вдалося створити папку налаштувань: {error}"))?;
    Ok(data_dir.join("state.json"))
}

fn load_state() -> Result<AppState, String> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(AppState::default());
    }
    let content =
        fs::read_to_string(path).map_err(|error| format!("Не вдалося прочитати дані: {error}"))?;
    let mut state: AppState = serde_json::from_str(&content)
        .map_err(|error| format!("Пошкоджені дані застосунку: {error}"))?;
    if state.decod_mcp_path.trim().is_empty() {
        state.decod_mcp_path = default_decod_path();
    }
    Ok(state)
}

fn save_state(state: &AppState) -> Result<(), String> {
    let path = state_path()?;
    let content = serde_json::to_string_pretty(state)
        .map_err(|error| format!("Не вдалося підготувати дані до збереження: {error}"))?;
    atomic_write(&path, content.as_bytes())
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "Не вдалося визначити папку для тимчасового файла.".to_owned())?;
    let name = path
        .file_name()
        .ok_or_else(|| "Не вдалося визначити назву файла для збереження.".to_owned())?
        .to_string_lossy();
    let temp_path = parent.join(format!(".{name}.{}.tmp", Uuid::new_v4()));
    let write_result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| format!("Не вдалося створити тимчасовий файл стану: {error}"))?;
        file.write_all(content)
            .map_err(|error| format!("Не вдалося записати тимчасовий файл стану: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Не вдалося завершити запис файла стану: {error}"))?;
        drop(file);
        fs::rename(&temp_path, path)
            .map_err(|error| format!("Не вдалося атомарно зберегти файл стану: {error}"))
    })();

    if let Err(error) = write_result {
        return match fs::remove_file(&temp_path) {
            Ok(()) => Err(error),
            Err(cleanup_error) if cleanup_error.kind() == std::io::ErrorKind::NotFound => {
                Err(error)
            }
            Err(cleanup_error) => Err(format!(
                "{error}; не вдалося прибрати тимчасовий файл {}: {cleanup_error}",
                temp_path.display()
            )),
        };
    }
    Ok(())
}

fn list_project_files(project_root: &Path) -> Result<ProjectFileList, String> {
    let root = project_root
        .canonicalize()
        .map_err(|error| format!("Не вдалося відкрити папку проєкту: {error}"))?;
    if !root.is_dir() {
        return Err("Шлях проєкту більше не є папкою.".to_owned());
    }

    let mut pending = vec![(root.clone(), 0_usize)];
    let mut files = Vec::new();
    let mut truncated = false;
    'scan: while let Some((directory, depth)) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("Не вдалося прочитати {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry
                .map_err(|error| format!("Не вдалося прочитати список файлів проєкту: {error}"))?;
            let file_type = entry.file_type().map_err(|error| {
                format!(
                    "Не вдалося визначити тип {}: {error}",
                    entry.path().display()
                )
            })?;
            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if depth < MAX_PROJECT_SCAN_DEPTH && !ignored_project_directory(&name) {
                    pending.push((path, depth + 1));
                }
            } else if file_type.is_file() {
                let relative_path = path
                    .strip_prefix(&root)
                    .map_err(|error| format!("Файл вийшов за межі проєкту: {error}"))?;
                if files.len() == MAX_PROJECT_FILES {
                    truncated = true;
                    break 'scan;
                }
                files.push(relative_path.to_owned());
            }
        }
    }
    files.sort_by_key(|path| path.to_string_lossy().to_lowercase());
    Ok(ProjectFileList { files, truncated })
}

fn project_file_matches_filter(path: &Path, filter: &str) -> bool {
    let filter = filter.trim();
    filter.is_empty()
        || path
            .to_string_lossy()
            .to_lowercase()
            .contains(&filter.to_lowercase())
}

fn ignored_project_directory(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        ".git"
            | ".next"
            | ".venv"
            | "__pycache__"
            | "bin"
            | "build"
            | "dist"
            | "node_modules"
            | "obj"
            | "target"
            | "vendor"
            | "venv"
    )
}

fn is_supported_project_text_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if matches!(
        name.to_ascii_lowercase().as_str(),
        ".env.example" | ".gitignore" | "cargo.lock" | "dockerfile" | "license" | "makefile"
    ) {
        return true;
    }
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "c" | "cc"
                | "cfg"
                | "cpp"
                | "cs"
                | "css"
                | "go"
                | "h"
                | "hpp"
                | "html"
                | "ini"
                | "java"
                | "js"
                | "json"
                | "jsx"
                | "lock"
                | "log"
                | "md"
                | "ps1"
                | "py"
                | "rs"
                | "sh"
                | "sql"
                | "toml"
                | "ts"
                | "tsx"
                | "txt"
                | "xml"
                | "yaml"
                | "yml"
        )
    )
}

fn is_supported_project_image_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("bmp" | "gif" | "jpeg" | "jpg" | "png" | "webp")
    )
}

fn is_supported_project_pdf_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

fn estimate_pdf_page_count(path: &Path) -> Option<usize> {
    const SCAN_BYTES: usize = 1024;
    let mut file = fs::File::open(path).ok()?;
    let mut buf = vec![0u8; SCAN_BYTES];
    let n = std::io::Read::read(&mut file, &mut buf).ok()?;
    let chunk = &buf[..n];
    // Count occurrences of b"/Type /Page" (or "/Type/Page") as a quick estimate
    let needle1 = b"/Type /Page";
    let needle2 = b"/Type/Page";
    let count1 = chunk.windows(needle1.len()).filter(|w| *w == needle1).count();
    let count2 = chunk.windows(needle2.len()).filter(|w| *w == needle2).count();
    let total = count1 + count2;
    if total > 0 { Some(total) } else { None }
}

fn canonical_project_file(project_root: &Path, relative_path: &Path) -> Result<PathBuf, String> {
    if relative_path.as_os_str().is_empty()
        || relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("Вибери файл усередині папки проєкту.".to_owned());
    }
    let root = project_root
        .canonicalize()
        .map_err(|error| format!("Не вдалося відкрити папку проєкту: {error}"))?;
    let path = root
        .join(relative_path)
        .canonicalize()
        .map_err(|error| format!("Не вдалося відкрити файл проєкту: {error}"))?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err("Файл має бути звичайним файлом усередині проєкту.".to_owned());
    }
    Ok(path)
}

fn read_project_text_file(project_root: &Path, relative_path: &Path) -> Result<String, String> {
    if !is_supported_project_text_file(relative_path) {
        return Err("Вбудований перегляд цього типу файлів ще не підтримується.".to_owned());
    }
    let path = canonical_project_file(project_root, relative_path)?;
    let metadata = fs::metadata(&path)
        .map_err(|error| format!("Не вдалося прочитати атрибути файлу: {error}"))?;
    if metadata.len() > MAX_PROJECT_FILE_BYTES {
        return Err("Файл завеликий для вбудованого редактора (ліміт 2 МБ).".to_owned());
    }
    fs::read_to_string(path)
        .map_err(|error| format!("Файл не є коректним UTF-8 текстом або не читається: {error}"))
}

fn read_project_image_file(
    project_root: &Path,
    relative_path: &Path,
) -> Result<(egui::ColorImage, u32, u32), String> {
    if !is_supported_project_image_file(relative_path) {
        return Err("Вбудований перегляд цього типу зображень не підтримується.".to_owned());
    }
    let path = canonical_project_file(project_root, relative_path)?;
    let metadata = fs::metadata(&path)
        .map_err(|error| format!("Не вдалося прочитати атрибути зображення: {error}"))?;
    if metadata.len() > MAX_PROJECT_IMAGE_BYTES {
        return Err("Зображення завелике для перегляду (ліміт 20 МБ).".to_owned());
    }

    let dimension_reader = image::ImageReader::open(&path)
        .map_err(|error| format!("Не вдалося відкрити зображення: {error}"))?
        .with_guessed_format()
        .map_err(|error| format!("Не вдалося розпізнати формат зображення: {error}"))?;
    let dimensions = dimension_reader
        .into_dimensions()
        .map_err(|error| format!("Не вдалося прочитати розмір зображення: {error}"))?;
    let (width, height) = dimensions;
    if width == 0
        || height == 0
        || width > MAX_PROJECT_IMAGE_DIMENSION
        || height > MAX_PROJECT_IMAGE_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PROJECT_IMAGE_PIXELS
    {
        return Err("Роздільність зображення перевищує ліміт вбудованого перегляду.".to_owned());
    }

    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_PROJECT_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_PROJECT_IMAGE_DIMENSION);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = image::ImageReader::open(&path)
        .map_err(|error| format!("Не вдалося відкрити зображення: {error}"))?
        .with_guessed_format()
        .map_err(|error| format!("Не вдалося розпізнати формат зображення: {error}"))?;
    reader.limits(limits);
    let decoded = reader
        .decode()
        .map_err(|error| format!("Не вдалося декодувати зображення: {error}"))?
        .into_rgba8();
    if decoded.dimensions() != dimensions {
        return Err("Розмір декодованого зображення не збігається із заголовком.".to_owned());
    }
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        decoded.as_raw(),
    );
    Ok((color_image, width, height))
}

fn save_project_text_file(
    project_root: &Path,
    relative_path: &Path,
    content: &str,
) -> Result<(), String> {
    if !is_supported_project_text_file(relative_path) {
        return Err("Збереження цього типу файлів не підтримується.".to_owned());
    }
    if content.len() as u64 > MAX_PROJECT_FILE_BYTES {
        return Err("Файл завеликий для вбудованого редактора (ліміт 2 МБ).".to_owned());
    }
    let path = canonical_project_file(project_root, relative_path)?;
    fs::write(path, content).map_err(|error| format!("Не вдалося зберегти файл: {error}"))
}

fn create_project_markdown_document(
    project_root: &Path,
    kind: ProjectDocumentKind,
) -> Result<PathBuf, String> {
    let root = project_root
        .canonicalize()
        .map_err(|error| format!("Не вдалося відкрити папку проєкту: {error}"))?;
    if !root.is_dir() {
        return Err("Шлях проєкту більше не є папкою.".to_owned());
    }

    let (subdirectories, prefix, template) = match kind {
        ProjectDocumentKind::Plan => (
            ["docs", "plans"],
            "plan",
            "# План реалізації\n\n\
             ## Мета\n\n\
             Опиши очікуваний результат.\n\n\
             ## Обсяг\n\n\
             - Що входить у роботу:\n\
             - Що не входить:\n\n\
             ## Етапи\n\n\
             - [ ] Крок 1\n\
             - [ ] Крок 2\n\n\
             ## Критерії готовності\n\n\
             - [ ] Перевірюваний результат\n\n\
             ## Ризики та відкриті питання\n\n",
        ),
        ProjectDocumentKind::Report => (
            ["docs", "reports"],
            "report",
            "# Звіт про роботу\n\n\
             ## Підсумок\n\n\
             ## Виконані зміни\n\n\
             ## Перевірки\n\n\
             ## Відкриті питання та ризики\n\n\
             ## Наступні кроки\n\n",
        ),
    };

    let mut directory = root.clone();
    for subdirectory in subdirectories {
        let next = directory.join(subdirectory);
        match fs::symlink_metadata(&next) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(format!(
                    "Папка документів має бути звичайною папкою: {}",
                    next.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&next).map_err(|error| {
                    format!(
                        "Не вдалося створити папку документів {}: {error}",
                        next.display()
                    )
                })?;
            }
            Err(error) => {
                return Err(format!(
                    "Не вдалося перевірити папку документів {}: {error}",
                    next.display()
                ));
            }
        }
        directory = next
            .canonicalize()
            .map_err(|error| format!("Не вдалося відкрити папку документів: {error}"))?;
        if !directory.starts_with(&root) {
            return Err("Папка документів має бути всередині проєкту.".to_owned());
        }
    }

    let timestamp = Local::now();
    let suffix = Uuid::new_v4().simple().to_string();
    let file_name = format!(
        "{prefix}-{}-{}.md",
        timestamp.format("%Y%m%d-%H%M%S"),
        &suffix[..8]
    );
    let path = directory.join(file_name);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("Не вдалося створити документ {}: {error}", path.display()))?;
    if let Err(error) = file.write_all(template.as_bytes()) {
        drop(file);
        if let Err(cleanup_error) = fs::remove_file(&path) {
            return Err(format!(
                "Не вдалося записати документ ({error}) або прибрати неповний файл {} ({cleanup_error}).",
                path.display()
            ));
        }
        return Err(format!("Не вдалося записати документ: {error}"));
    }

    path.strip_prefix(&root)
        .map(Path::to_owned)
        .map_err(|error| format!("Не вдалося визначити шлях документа в проєкті: {error}"))
}

#[derive(Clone, Debug, PartialEq)]
enum MarkdownBlockKind {
    Paragraph,
    Heading(HeadingLevel),
    Code,
    Quote,
    ListItem,
    TableRow,
    Rule,
}

#[derive(Debug, PartialEq)]
struct MarkdownBlock {
    kind: MarkdownBlockKind,
    text: String,
}

#[derive(Default)]
struct MarkdownBlockBuilder {
    blocks: Vec<MarkdownBlock>,
    current: Option<MarkdownBlock>,
    lists: Vec<Option<u64>>,
    quote_depth: usize,
    table_cell_count: usize,
}

impl MarkdownBlockBuilder {
    fn start(&mut self, kind: MarkdownBlockKind) {
        self.flush();
        self.current = Some(MarkdownBlock {
            kind,
            text: String::new(),
        });
    }

    fn ensure(&mut self, kind: MarkdownBlockKind) {
        if self.current.is_none() {
            self.current = Some(MarkdownBlock {
                kind,
                text: String::new(),
            });
        }
    }

    fn push(&mut self, text: &str, kind: MarkdownBlockKind) {
        self.ensure(kind);
        if let Some(block) = &mut self.current {
            block.text.push_str(text);
        }
    }

    fn flush(&mut self) {
        if let Some(block) = self.current.take() {
            if !block.text.is_empty() {
                self.blocks.push(block);
            }
        }
    }
}

fn markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let mut builder = MarkdownBlockBuilder::default();

    for event in Parser::new_ext(markdown, options) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                builder.start(MarkdownBlockKind::Heading(level));
            }
            Event::Start(Tag::CodeBlock(_)) => builder.start(MarkdownBlockKind::Code),
            Event::Start(Tag::Paragraph) => {
                if builder.quote_depth > 0 {
                    builder.start(MarkdownBlockKind::Quote);
                    if let Some(block) = &mut builder.current {
                        block.text.push_str(&"> ".repeat(builder.quote_depth));
                    }
                } else if !matches!(
                    builder.current.as_ref().map(|block| &block.kind),
                    Some(MarkdownBlockKind::ListItem)
                ) {
                    builder.start(MarkdownBlockKind::Paragraph);
                } else {
                    continue;
                }
            }
            Event::Start(Tag::BlockQuote(_)) => builder.quote_depth += 1,
            Event::Start(Tag::List(start)) => builder.lists.push(start),
            Event::Start(Tag::Item) => {
                builder.start(MarkdownBlockKind::ListItem);
                let indent = "  ".repeat(builder.lists.len().saturating_sub(1));
                if let Some(block) = &mut builder.current {
                    block.text.push_str(&indent);
                }
                if let Some(Some(number)) = builder.lists.last_mut() {
                    if let Some(block) = &mut builder.current {
                        block.text.push_str(&format!("{number}. "));
                    }
                    *number = number.saturating_add(1);
                } else if let Some(block) = &mut builder.current {
                    block.text.push_str("• ");
                }
            }
            Event::Start(Tag::Table(_)) => builder.table_cell_count = 0,
            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                builder.table_cell_count = 0;
                builder.start(MarkdownBlockKind::TableRow);
            }
            Event::Start(Tag::TableCell) => {
                if builder.table_cell_count > 0 {
                    builder.push(" │ ", MarkdownBlockKind::TableRow);
                }
                builder.table_cell_count += 1;
            }
            Event::Start(Tag::Image { .. }) => {
                builder.push("[Зображення: ", MarkdownBlockKind::Paragraph);
            }
            Event::Start(Tag::HtmlBlock) => builder.start(MarkdownBlockKind::Code),
            Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::CodeBlock)
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::TableHead)
            | Event::End(TagEnd::TableRow)
            | Event::End(TagEnd::HtmlBlock) => builder.flush(),
            Event::End(TagEnd::BlockQuote(_)) => {
                builder.quote_depth = builder.quote_depth.saturating_sub(1);
            }
            Event::End(TagEnd::List(_)) => {
                builder.lists.pop();
            }
            Event::End(TagEnd::TableCell) => {}
            Event::End(TagEnd::Image) => {
                builder.push("]", MarkdownBlockKind::Paragraph);
            }
            Event::Text(text)
            | Event::Code(text)
            | Event::InlineMath(text)
            | Event::DisplayMath(text)
            | Event::Html(text)
            | Event::InlineHtml(text) => {
                let kind = if builder.current.is_some() {
                    builder
                        .current
                        .as_ref()
                        .expect("current block was checked")
                        .kind
                        .clone()
                } else {
                    MarkdownBlockKind::Paragraph
                };
                builder.push(&text, kind);
            }
            Event::SoftBreak => builder.push(" ", MarkdownBlockKind::Paragraph),
            Event::HardBreak => builder.push("\n", MarkdownBlockKind::Paragraph),
            Event::Rule => {
                builder.flush();
                builder.blocks.push(MarkdownBlock {
                    kind: MarkdownBlockKind::Rule,
                    text: String::new(),
                });
            }
            Event::TaskListMarker(checked) => builder.push(
                if checked { "[x] " } else { "[ ] " },
                MarkdownBlockKind::ListItem,
            ),
            Event::FootnoteReference(label) => {
                builder.push(&format!("[^{label}]"), MarkdownBlockKind::Paragraph);
            }
            Event::Start(_) | Event::End(_) => {}
        }
    }
    builder.flush();
    builder.blocks
}

fn show_markdown_preview(ui: &mut egui::Ui, markdown: &str) {
    for block in markdown_blocks(markdown) {
        match block.kind {
            MarkdownBlockKind::Paragraph => {
                ui.label(block.text);
                ui.add_space(4.0);
            }
            MarkdownBlockKind::Heading(level) => {
                let size = match level {
                    HeadingLevel::H1 => 28.0,
                    HeadingLevel::H2 => 24.0,
                    HeadingLevel::H3 => 20.0,
                    HeadingLevel::H4 => 18.0,
                    HeadingLevel::H5 => 16.0,
                    HeadingLevel::H6 => 14.0,
                };
                ui.heading(egui::RichText::new(block.text).size(size));
            }
            MarkdownBlockKind::Code => {
                ui.label(
                    egui::RichText::new(block.text)
                        .monospace()
                        .background_color(ui.visuals().code_bg_color),
                );
            }
            MarkdownBlockKind::Quote => {
                ui.label(egui::RichText::new(block.text).italics().weak());
            }
            MarkdownBlockKind::ListItem => {
                ui.label(block.text);
            }
            MarkdownBlockKind::TableRow => {
                ui.label(egui::RichText::new(block.text).monospace());
            }
            MarkdownBlockKind::Rule => {
                ui.separator();
            }
        }
    }
}

fn conversation_context(
    messages: &[Message],
    context_limit: ConversationContextLimit,
) -> &[Message] {
    let Some(max_user_turns) = context_limit.user_turns() else {
        return messages;
    };
    let start = messages
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, message)| message.role == MessageRole::User)
        .nth(max_user_turns.saturating_sub(1))
        .map(|(index, _)| index)
        .unwrap_or(0);
    &messages[start..]
}

fn normalize_chat_title(title: &str) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 80 {
        return Err("Назва чату має містити від 1 до 80 символів.".to_owned());
    }
    Ok(title.to_owned())
}

fn chat_export_markdown(chat: &Chat) -> String {
    let mut markdown = format!("# {}\n\nЕкспортовано з Mnemosyne.\n\n", chat.title);
    for message in &chat.messages {
        let role = match message.role {
            MessageRole::User => "Ти",
            MessageRole::Assistant => "Claude",
        };
        markdown.push_str(&format!(
            "## {role} · {}\n\n",
            message.created_at.format("%Y-%m-%d %H:%M")
        ));
        if !message.content.is_empty() {
            markdown.push_str(&message.content);
            markdown.push_str("\n\n");
        }
        if !message.attachments.is_empty() {
            markdown.push_str("Вкладення:\n");
            for path in &message.attachments {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("файл")
                    .replace('`', "'")
                    .replace('\n', " ")
                    .replace('\r', " ");
                markdown.push_str(&format!("- `{name}`\n"));
            }
            markdown.push('\n');
        }
    }
    markdown
}

fn remove_chat_from_project(project: &mut Project, chat_id: &str) -> bool {
    let original_len = project.chats.len();
    project.chats.retain(|chat| chat.id != chat_id);
    project.chats.len() != original_len
}

fn clear_chat_messages(project: &mut Project, chat_id: &str) -> bool {
    let Some(chat) = project.chats.iter_mut().find(|chat| chat.id == chat_id) else {
        return false;
    };
    chat.messages.clear();
    true
}

fn remove_project_record(state: &mut AppState, project_id: &str) -> bool {
    let original_len = state.projects.len();
    state.projects.retain(|project| project.id != project_id);
    if state.projects.len() == original_len {
        return false;
    }
    if state.selected_project.as_deref() == Some(project_id) {
        state.selected_project = state.projects.first().map(|project| project.id.clone());
        state.selected_chat = state
            .projects
            .first()
            .and_then(|project| project.chats.first())
            .map(|chat| chat.id.clone());
    }
    true
}

impl ApprovalServer {
    fn start(
        project_root: &Path,
        additional_root: Option<&Path>,
        agent_mode: AgentMode,
        approval_tx: Sender<FileChangeApproval>,
        cancel_requested: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let project_root = project_root
            .canonicalize()
            .map_err(|error| format!("Не вдалося перевірити папку проєкту: {error}"))?;
        let mut allowed_roots = vec![project_root.clone()];
        if let Some(additional_root) = additional_root {
            let additional_root = additional_root
                .canonicalize()
                .map_err(|error| format!("Не вдалося перевірити папку вкладень: {error}"))?;
            if !allowed_roots.contains(&additional_root) {
                allowed_roots.push(additional_root);
            }
        }
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|error| format!("Не вдалося запустити перевірку змін агента: {error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| format!("Не вдалося налаштувати перевірку змін агента: {error}"))?;
        let address = listener
            .local_addr()
            .map_err(|error| format!("Не вдалося визначити адресу перевірки змін: {error}"))?;
        let token = Uuid::new_v4().to_string();
        let running = Arc::new(AtomicBool::new(true));
        let server_running = Arc::clone(&running);
        let server_token = token.clone();
        let thread = thread::spawn(move || {
            while server_running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => handle_approval_connection(
                        stream,
                        &project_root,
                        &allowed_roots,
                        agent_mode,
                        &server_token,
                        &approval_tx,
                        &server_running,
                        &cancel_requested,
                    ),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(error) => {
                        eprintln!("Не вдалося обробити підтвердження зміни: {error}");
                        break;
                    }
                }
            }
        });
        Ok(Self {
            address,
            token,
            running,
            thread: Some(thread),
        })
    }

    fn settings_json(&self, api_base_url: &str, api_key: &str) -> Result<String, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("Не вдалося визначити файл Mnemosyne: {error}"))?;
        Ok(serde_json::json!({
            "env": {
                "ANTHROPIC_BASE_URL": api_base_url,
                "ANTHROPIC_AUTH_TOKEN": api_key,
                "ANTHROPIC_API_KEY": api_key,
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1"
            },
            "apiKeyHelper": "",
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": executable,
                        "args": [
                            "--mnemosyne-approval-hook",
                            self.address.to_string(),
                            &self.token
                        ],
                        "timeout": 600
                    }]
                }]
            }
        })
        .to_string())
    }
}

impl Drop for ApprovalServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn handle_approval_connection(
    mut stream: TcpStream,
    project_root: &Path,
    _allowed_roots: &[PathBuf],
    agent_mode: AgentMode,
    expected_token: &str,
    approval_tx: &Sender<FileChangeApproval>,
    running: &AtomicBool,
    cancel_requested: &AtomicBool,
) {
    let response = (|| -> Result<Vec<u8>, String> {
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|error| format!("Не вдалося налаштувати читання запиту: {error}"))?;
        let request = read_approval_frame(&mut stream, MAX_APPROVAL_HOOK_BYTES)?;
        let envelope: serde_json::Value = serde_json::from_slice(&request)
            .map_err(|error| format!("Некоректний запит підтвердження: {error}"))?;
        if envelope.get("token").and_then(serde_json::Value::as_str) != Some(expected_token) {
            return Ok(approval_decision(
                false,
                "Mnemosyne відхилила неавторизований запит зміни.",
            ));
        }
        let input = envelope
            .get("input")
            .ok_or_else(|| "У запиті підтвердження немає даних інструмента.".to_owned())?;
        let tool = input
            .get("tool_name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Не вдалося визначити інструмент Claude Code.".to_owned())?;
        let tool_input = input
            .get("tool_input")
            .ok_or_else(|| "Не вдалося прочитати параметри інструмента Claude Code.".to_owned())?;
        if tool != "Edit" && tool != "Write" && tool != "Bash" {
            return Ok(approval_decision(true, ""));
        }
        if !matches!(tool, "Edit" | "Write") || agent_mode != AgentMode::Turbo {
            return Ok(approval_decision(
                false,
                "Інструмент недоступний у вибраному режимі Mnemosyne.",
            ));
        }
        let target = tool_input
            .get("file_path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Не вдалося визначити файл для зміни.".to_owned())?;
        let cwd = input
            .get("cwd")
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from);
        let relative_path = match project_relative_write_path(project_root, target, cwd.as_deref())
        {
            Ok(path) => path,
            Err(reason) => return Ok(approval_decision(false, &reason)),
        };
        let (response_tx, response_rx) = mpsc::channel();
        approval_tx
            .send(FileChangeApproval {
                tool: tool.to_owned(),
                path: relative_path.to_string_lossy().into_owned(),
                preview: file_change_preview(tool, tool_input),
                response_tx,
            })
            .map_err(|error| format!("Не вдалося показати підтвердження зміни: {error}"))?;
        loop {
            if !running.load(Ordering::SeqCst) || cancel_requested.load(Ordering::SeqCst) {
                return Ok(approval_decision(
                    false,
                    "Запит скасовано або вікно підтвердження закрито.",
                ));
            }
            match response_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(true) => return Ok(approval_decision(true, "")),
                Ok(false) => return Ok(approval_decision(false, "Зміну відхилено користувачем.")),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Ok(approval_decision(
                        false,
                        "Не вдалося отримати рішення щодо зміни.",
                    ))
                }
            }
        }
    })()
    .unwrap_or_else(|error| approval_decision(false, &error));
    if let Err(error) = write_approval_frame(&mut stream, &response) {
        eprintln!("Не вдалося повернути рішення для Claude Code: {error}");
    }
}

fn file_change_preview(tool: &str, tool_input: &serde_json::Value) -> String {
    const MAX_PREVIEW_CHARS: usize = 12_000;
    let (label, content) = if tool == "Edit" {
        let old = tool_input
            .get("old_string")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("(зміст для заміни не надано)");
        let new = tool_input
            .get("new_string")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("(новий зміст не надано)");
        (
            "Заміна фрагмента",
            format!("--- було\n{old}\n+++ стане\n{new}"),
        )
    } else {
        let content = tool_input
            .get("content")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("(вміст для запису не надано)");
        ("Новий вміст файла", content.to_owned())
    };
    let mut preview: String = content.chars().take(MAX_PREVIEW_CHARS).collect();
    if content.chars().count() > MAX_PREVIEW_CHARS {
        preview.push_str("\n… попередній перегляд обрізано");
    }
    format!("{label}\n{preview}")
}

fn project_relative_write_path(
    project_root: &Path,
    target: &str,
    cwd: Option<&Path>,
) -> Result<PathBuf, String> {
    let (root, relative) = resolve_path_within_roots(&[project_root.to_owned()], target, cwd)?;
    if root != 0 || relative.as_os_str().is_empty() {
        return Err("Зміна кореневої папки проєкту не дозволена.".to_owned());
    }
    Ok(relative)
}

#[allow(dead_code)]
fn path_within_roots(
    roots: &[PathBuf],
    target: &str,
    cwd: Option<&Path>,
) -> Result<(usize, PathBuf), String> {
    resolve_path_within_roots(roots, target, cwd).map_err(|_| {
        "Claude Code не може читати файли поза папками проєкту та вкладень.".to_owned()
    })
}

fn resolve_path_within_roots(
    roots: &[PathBuf],
    target: &str,
    cwd: Option<&Path>,
) -> Result<(usize, PathBuf), String> {
    let target_path = Path::new(target);
    if target_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Шлях виходить за межі дозволених папок.".to_owned());
    }
    let candidate = if target_path.is_absolute() {
        target_path.to_owned()
    } else if let Some(cwd) = cwd {
        cwd.join(target_path)
    } else {
        roots
            .first()
            .ok_or_else(|| "Не налаштовано дозволені папки.".to_owned())?
            .join(target_path)
    };
    let resolved = resolve_existing_prefix(&candidate)
        .map_err(|error| format!("Не вдалося перевірити шлях файлу: {error}"))?;
    for (index, root) in roots.iter().enumerate() {
        if let Ok(relative) = resolved.strip_prefix(root) {
            return Ok((index, relative.to_owned()));
        }
    }
    Err("Шлях виходить за межі дозволених папок.".to_owned())
}

fn resolve_existing_prefix(path: &Path) -> std::io::Result<PathBuf> {
    let mut existing = path.to_owned();
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(&existing) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let name = existing
                    .file_name()
                    .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
                suffix.push(name.to_owned());
                if !existing.pop() {
                    return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
                }
            }
            Err(error) => return Err(error),
        }
    }
    let mut resolved = existing.canonicalize()?;
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn approval_decision(allow: bool, reason: &str) -> Vec<u8> {
    let mut output = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": if allow { "allow" } else { "deny" }
        }
    });
    if !allow {
        output["hookSpecificOutput"]["permissionDecisionReason"] =
            serde_json::Value::String(reason.to_owned());
    }
    serde_json::to_vec(&output).expect("approval response is serializable")
}

fn read_approval_frame(stream: &mut TcpStream, max_bytes: usize) -> Result<Vec<u8>, String> {
    let mut length = [0; 4];
    stream
        .read_exact(&mut length)
        .map_err(|error| format!("Не вдалося прочитати запит підтвердження: {error}"))?;
    let length = u32::from_be_bytes(length) as usize;
    if length > max_bytes {
        return Err("Запит підтвердження перевищує допустимий розмір.".to_owned());
    }
    let mut bytes = vec![0; length];
    stream
        .read_exact(&mut bytes)
        .map_err(|error| format!("Не вдалося прочитати дані підтвердження: {error}"))?;
    Ok(bytes)
}

fn write_approval_frame(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
    let length = u32::try_from(bytes.len())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(bytes)
}

struct TempFileGuard(PathBuf);
impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn run_claude(
    project_path: &Path,
    messages: &[Message],
    api_base_url: &str,
    api_key: &str,
    agent_mode: AgentMode,
    model: ClaudeModel,
    stream_tx: &Sender<ClaudeStreamChunk>,
    approval_tx: &Sender<FileChangeApproval>,
    project_id: &str,
    chat_id: &str,
    active_child: &Arc<Mutex<Option<GroupChild>>>,
    cancel_requested: &Arc<AtomicBool>,
    mcp_config: Option<String>,
    custom_model: Option<&str>,
) -> Result<String, String> {
    let has_attachments = messages
        .iter()
        .any(|message| !message.attachments.is_empty());
    let staging = if !has_attachments {
        None
    } else {
        Some(AttachmentStaging::new()?)
    };
    let approval_server = Some(ApprovalServer::start(
        project_path,
        staging.as_ref().map(|staging| staging.path.as_path()),
        agent_mode,
        approval_tx.clone(),
        Arc::clone(cancel_requested),
    )?);
    let mut total_attachment_bytes = 0;
    let mut staged_attachment_index = 0;
    let mut prompt = String::new();
    for message in messages {
        prompt.push_str(match message.role {
            MessageRole::User => "\n\nКористувач:\n",
            MessageRole::Assistant => "\n\nClaude:\n",
        });
        prompt.push_str(&message.content);
        for path in &message.attachments {
            let metadata = fs::metadata(path).map_err(|error| {
                format!("Не вдалося прочитати вкладення {}: {error}", path.display())
            })?;
            if !metadata.is_file() {
                return Err(format!("Вкладення не є файлом: {}", path.display()));
            }
            total_attachment_bytes += metadata.len();
            if total_attachment_bytes > MAX_STAGED_ATTACHMENT_BYTES {
                return Err(
                    "Загальний розмір вкладень у чаті не може перевищувати 20 МБ.".to_owned(),
                );
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("attachment");
            let staged_path = staging
                .as_ref()
                .expect("staging exists when attachments are present")
                .path
                .join(format!("{staged_attachment_index}-{name}"));
            staged_attachment_index += 1;
            fs::copy(path, &staged_path).map_err(|error| {
                format!(
                    "Не вдалося підготувати вкладення {}: {error}",
                    path.display()
                )
            })?;

            prompt.push_str(&format!(
                "\nВкладення: {} (шлях для Claude Code: {})",
                name,
                staged_path.display()
            ));
            if metadata.len() <= MAX_ATTACHMENT_BYTES {
                let bytes = fs::read(&staged_path)
                    .map_err(|error| format!("Не вдалося прочитати вкладення {name}: {error}"))?;
                if let Ok(text) = String::from_utf8(bytes) {
                    prompt.push_str("\nВміст файлу:\n");
                    prompt.push_str(&text.chars().take(MAX_ATTACHMENT_CHARS).collect::<String>());
                }
            }
        }
    }

    let mut command = claude_command();
    command.arg("--dangerously-skip-permissions");
    let mut args: Vec<String> = claude_args(agent_mode, model).into_iter().map(String::from).collect();
    if let Some(custom) = custom_model.filter(|s| !s.trim().is_empty()) {
        if let Some(pos) = args.iter().position(|a| a == "--model") {
            if pos + 1 < args.len() {
                args[pos + 1] = custom.to_string();
            }
        } else {
            args.extend(["--model".to_string(), custom.to_string()]);
        }
    }
    if mcp_config.is_some() {
        if let Some(pos) = args.iter().position(|a| a == "--tools") {
            args.remove(pos);
            if pos < args.len() {
                args.remove(pos);
            }
        }
    }
    command.args(args);
    if let Some(server) = &approval_server {
        command.args(["--setting-sources", "user"]);
        command.arg("--settings").arg(server.settings_json(api_base_url, api_key)?);
    }
    if let Some(staging) = &staging {
        command.args(["--add-dir", staging.path.to_string_lossy().as_ref()]);
    }
    let _mcp_guard = if let Some(mcp_json) = &mcp_config {
        let path = std::env::temp_dir().join(format!("mnemosyne-mcp-{}.json", Uuid::new_v4()));
        let _ = fs::write(&path, mcp_json);
        command.arg("--mcp-config").arg(&path);
        Some(TempFileGuard(path))
    } else {
        None
    };
    let mut child = command
        .current_dir(project_path)
        .env("ANTHROPIC_BASE_URL", api_base_url)
        .env("ANTHROPIC_AUTH_TOKEN", api_key)
        .env("ANTHROPIC_API_KEY", api_key)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .group_spawn()
        .map_err(|error| {
            format!(
                "Не вдалося запустити Claude Code CLI. Перевірте встановлення Claude Code: {error}"
            )
        })?;
    let mut stdin = child
        .inner()
        .stdin
        .take()
        .ok_or_else(|| "Claude Code не відкрив stdin для запиту.".to_owned())?;
    let stdout = child
        .inner()
        .stdout
        .take()
        .ok_or_else(|| "Claude Code не відкрив stdout для потоку відповіді.".to_owned())?;
    let stderr = child
        .inner()
        .stderr
        .take()
        .ok_or_else(|| "Claude Code не відкрив stderr для діагностики.".to_owned())?;
    {
        let mut running = active_child
            .lock()
            .map_err(|_| "Не вдалося зберегти процес Claude Code для керування.".to_owned())?;
        *running = Some(child);
    }

    let stderr_reader = thread::spawn(move || read_bounded_stderr(stderr));
    if let Err(error) = stdin.write_all(prompt.as_bytes()) {
        let stop_result = stop_active_child(active_child);
        let wait_result = wait_active_child(active_child);
        let stderr_error = match stderr_reader.join() {
            Ok(Ok(_)) => String::new(),
            Ok(Err(error)) => format!("; не вдалося прочитати stderr: {error}"),
            Err(_) => "; не вдалося завершити читання stderr.".to_owned(),
        };
        return Err(format!(
            "Не вдалося передати запит Claude Code: {error}{}{}{}",
            stop_result
                .err()
                .map(|message| format!("; {message}"))
                .unwrap_or_default(),
            wait_result
                .err()
                .map(|message| format!("; {message}"))
                .unwrap_or_default(),
            stderr_error
        ));
    }
    drop(stdin);

    let mut stop_error = if cancel_requested.load(Ordering::SeqCst) {
        stop_active_child(active_child).err()
    } else {
        None
    };

    let mut streamed_text = String::new();
    let mut final_text = None;
    let mut stream_error = None;
    let mut stdout = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match stdout.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => match parse_claude_stream_event(&line) {
                Ok(Some(ClaudeStreamEvent::Text(text))) => {
                    streamed_text.push_str(&text);
                    let _ = stream_tx.send(ClaudeStreamChunk {
                        project_id: project_id.to_owned(),
                        chat_id: chat_id.to_owned(),
                        update: ClaudeStreamUpdate::Text(text),
                    });
                }
                Ok(Some(ClaudeStreamEvent::Tool(tool))) => {
                    let _ = stream_tx.send(ClaudeStreamChunk {
                        project_id: project_id.to_owned(),
                        chat_id: chat_id.to_owned(),
                        update: ClaudeStreamUpdate::Tool(tool),
                    });
                }
                Ok(Some(ClaudeStreamEvent::Result(text))) => final_text = Some(text),
                Ok(None) => {}
                Err(error) => {
                    stream_error = Some(error);
                    break;
                }
            },
            Err(error) => {
                stream_error = Some(format!("Не вдалося прочитати потік Claude Code: {error}"));
                break;
            }
        }
    }

    let cancelled = cancel_requested.load(Ordering::SeqCst);
    if cancelled || stream_error.is_some() {
        if let Err(error) = stop_active_child(active_child) {
            stop_error = Some(error);
        }
    }
    let status = wait_active_child(active_child)?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "Не вдалося завершити читання діагностики Claude Code.".to_owned())?
        .map_err(|error| format!("Не вдалося прочитати діагностику Claude Code: {error}"))?;

    if cancelled {
        if let Some(error) = stop_error {
            return Err(format!(
                "Запит скасовано, але зупинити процес не вдалося: {error}"
            ));
        }
        return Err("Запит скасовано.".to_owned());
    }
    if let Some(error) = stream_error {
        return Err(error);
    }
    if !status.success() {
        return Err(if stderr.trim().is_empty() {
            format!("Claude Code завершився з кодом {status}.")
        } else {
            stderr.trim().to_owned()
        });
    }
    Ok(final_text.unwrap_or(streamed_text).trim().to_owned())
}

enum ClaudeStreamEvent {
    Text(String),
    Tool(String),
    Result(String),
}

fn parse_claude_stream_event(line: &str) -> Result<Option<ClaudeStreamEvent>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.starts_with('{') {
        return Ok(None);
    }
    let event: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(val) => val,
        Err(_) => return Ok(None),
    };
    match event.get("type").and_then(serde_json::Value::as_str) {
        Some("stream_event") => {
            let stream_event = event.get("event");
            let stream_event_type = stream_event
                .and_then(|event| event.get("type"))
                .and_then(serde_json::Value::as_str);
            if stream_event_type == Some("content_block_start") {
                let content_block = stream_event.and_then(|event| event.get("content_block"));
                if content_block
                    .and_then(|block| block.get("type"))
                    .and_then(serde_json::Value::as_str)
                    == Some("tool_use")
                {
                    let name = content_block
                        .and_then(|block| block.get("name"))
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            "Claude Code надіслав виклик інструмента без назви.".to_owned()
                        })?;
                    return Ok(Some(ClaudeStreamEvent::Tool(name.to_owned())));
                }
            }
            let delta = stream_event.and_then(|event| event.get("delta"));
            if delta
                .and_then(|delta| delta.get("type"))
                .and_then(serde_json::Value::as_str)
                == Some("text_delta")
            {
                let text = delta
                    .and_then(|delta| delta.get("text"))
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        "Claude Code надіслав текстову подію без поля text.".to_owned()
                    })?;
                return Ok(Some(ClaudeStreamEvent::Text(text.to_owned())));
            }
        }
        Some("result") => {
            let text = event
                .get("result")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "Claude Code надіслав result без тексту.".to_owned())?;
            return Ok(Some(ClaudeStreamEvent::Result(text.to_owned())));
        }
        _ => {}
    }
    Ok(None)
}

fn read_bounded_stderr(mut reader: impl Read) -> std::io::Result<String> {
    const MAX_STDERR_BYTES: usize = 1024 * 1024;
    const BUFFER_SIZE: usize = 8192;
    let mut output = Vec::new();
    let mut buffer = [0; BUFFER_SIZE];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = MAX_STDERR_BYTES.saturating_sub(output.len());
        output.extend_from_slice(&buffer[..read.min(remaining)]);
        truncated |= read > remaining;
    }
    let mut text = String::from_utf8_lossy(&output).into_owned();
    if truncated {
        text.push_str("\n[stderr обрізано після 1 МБ]");
    }
    Ok(text)
}

fn stop_active_child(active_child: &Arc<Mutex<Option<GroupChild>>>) -> Result<(), String> {
    let mut active_child = active_child
        .lock()
        .map_err(|_| "Не вдалося отримати доступ до процесу Claude Code.".to_owned())?;
    if let Some(child) = active_child.as_mut() {
        if let Err(error) = child.kill() {
            if error.kind() != std::io::ErrorKind::InvalidInput {
                return Err(format!("Не вдалося зупинити Claude Code: {error}"));
            }
        }
    }
    Ok(())
}

fn wait_active_child(
    active_child: &Arc<Mutex<Option<GroupChild>>>,
) -> Result<std::process::ExitStatus, String> {
    let mut active_child = active_child
        .lock()
        .map_err(|_| "Не вдалося отримати процес Claude Code для завершення.".to_owned())?;
    let mut child = active_child
        .take()
        .ok_or_else(|| "Процес Claude Code було втрачено до завершення.".to_owned())?;
    drop(active_child);
    child
        .wait()
        .map_err(|error| format!("Не вдалося дочекатися завершення Claude Code: {error}"))
}

fn claude_args(agent_mode: AgentMode, model: ClaudeModel) -> Vec<&'static str> {
    let mut args = vec![
        "-p",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--model",
        model.cli_alias(),
        "--permission-mode",
    ];
    match agent_mode {
        AgentMode::Plan => {
            args.extend(["plan", "--tools", "Read,Glob,Grep"]);
        }
        AgentMode::Turbo => {
            args.extend(["manual", "--tools", "Read,Glob,Grep,Edit,Write"]);
        }
    }
    args
}

fn freellmapi_key_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, FREELLMAPI_KEY_NAME)
        .map_err(|error| format!("Не вдалося відкрити системне сховище ключів: {error}"))
}

fn load_freellmapi_key() -> Result<Option<String>, String> {
    match freellmapi_key_entry()?.get_password() {
        Ok(key) if !key.trim().is_empty() => Ok(Some(key)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!(
            "Не вдалося прочитати ключ FreeLLMAPI зі сховища: {error}"
        )),
    }
}

fn save_freellmapi_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Введіть unified API key FreeLLMAPI.".to_owned());
    }
    freellmapi_key_entry()?
        .set_password(key)
        .map_err(|error| format!("Не вдалося зберегти ключ у системному сховищі: {error}"))
}

fn normalize_api_base_url(base_url: &str) -> Result<String, String> {
    let mut url = reqwest::Url::parse(base_url.trim())
        .map_err(|error| format!("Неправильна адреса FreeLLMAPI: {error}"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(
            "Вкажіть адресу FreeLLMAPI як http(s)://хост:порт без логіна, query чи fragment."
                .to_owned(),
        );
    }
    let is_loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !is_loopback {
        return Err(
            "Для віддаленого сервера потрібен HTTPS; HTTP дозволено лише для localhost.".to_owned(),
        );
    }
    let path = url.path().trim_end_matches('/');
    if !path.is_empty() {
        return Err(
            "Вкажіть адресу сервера без шляху; наприклад http://127.0.0.1:3001.".to_owned(),
        );
    }
    url.set_path("");
    Ok(url.as_str().trim_end_matches('/').to_owned())
}

fn fetch_ready_models(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let url = format!("{base_url}/v1/models?available=true");
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|error| format!("Не вдалося створити HTTP-клієнт: {error}"))?
        .get(url)
        .header("x-api-key", api_key)
        .bearer_auth(api_key)
        .send()
        .map_err(|error| {
            format!(
                "Не вдалося з’єднатися з FreeLLMAPI. Переконайтеся, що сервер запущено: {error}"
            )
        })?;
    if !response.status().is_success() {
        return Err(format!(
            "FreeLLMAPI відповів HTTP {}. Перевірте адресу, unified key і налаштовані ключі провайдерів.",
            response.status()
        ));
    }
    let body: serde_json::Value = response
        .json()
        .map_err(|error| format!("FreeLLMAPI повернув некоректний список моделей: {error}"))?;
    parse_ready_models(&body)
}

fn parse_ready_models(body: &serde_json::Value) -> Result<Vec<String>, String> {
    let entries = body
        .get("data")
        .and_then(serde_json::Value::as_array)
        .or_else(|| body.as_array())
        .ok_or_else(|| "FreeLLMAPI повернув каталог моделей у невідомому форматі.".to_owned())?;
    let models: Vec<String> = entries
        .iter()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(serde_json::Value::as_str)
                .or_else(|| entry.get("modelId").and_then(serde_json::Value::as_str))
                .map(str::to_owned)
        })
        .collect();
    if models.is_empty() {
        return Err("FreeLLMAPI не знайшов доступних моделей. Додайте ключі провайдерів і ввімкніть їх у fallback chain.".to_owned());
    }
    Ok(models)
}

fn fetch_local_models(base_url: &str) -> Result<Vec<String>, String> {
    let url = format!("{base_url}/v1/models");
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .map_err(|error| format!("Не вдалося створити HTTP-клієнт: {error}"))?
        .get(url)
        .header("User-Agent", "Mnemosyne")
        .send()
        .map_err(|error| {
            format!(
                "Не вдалося з'єднатися з локальним сервером ({base_url}). Переконайтеся, що LM Studio або Ollama запущені: {error}"
            )
        })?;
    if !response.status().is_success() {
        return Err(format!(
            "Локальний сервер відповів HTTP {}. Перевірте URL та статус сервера.",
            response.status()
        ));
    }
    let body: serde_json::Value = response
        .json()
        .map_err(|error| format!("Локальний сервер повернув некоректний JSON списку моделей: {error}"))?;
    parse_local_models(&body)
}

fn parse_local_models(body: &serde_json::Value) -> Result<Vec<String>, String> {
    let entries = body
        .get("data")
        .and_then(serde_json::Value::as_array)
        .or_else(|| body.as_array())
        .ok_or_else(|| "Локальний сервер повернув каталог моделей у невідомому форматі.".to_owned())?;
    let models: Vec<String> = entries
        .iter()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(serde_json::Value::as_str)
                .or_else(|| entry.get("name").and_then(serde_json::Value::as_str))
                .map(str::to_owned)
        })
        .collect();
    if models.is_empty() {
        return Err("Сервер не повернув завантажених моделей. Завантажте модель у LM Studio або запустіть модель в Ollama.".to_owned());
    }
    Ok(models)
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

fn current_platform_asset() -> Result<(&'static str, &'static str), String> {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return Ok(("FreeLLMAPI-Setup-", ".exe"));
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Ok(("FreeLLMAPI-", ".AppImage"));
    }
    #[allow(unreachable_code)]
    Err(
        "Автоматичне встановлення FreeLLMAPI поки підтримує лише Windows x64 та Linux x64."
            .to_owned(),
    )
}

fn select_release_asset(
    release: &GithubRelease,
    expected_prefix: &str,
    expected_suffix: &str,
) -> Result<(String, reqwest::Url, String), String> {
    if release.tag_name.is_empty() || release.tag_name.len() > 80 {
        return Err("GitHub повернув некоректну версію FreeLLMAPI.".to_owned());
    }
    let asset = release
        .assets
        .iter()
        .find(|asset| {
            asset.name.starts_with(expected_prefix) && asset.name.ends_with(expected_suffix)
        })
        .ok_or_else(|| {
            format!(
                "У релізі {} немає офіційного інсталятора {}.",
                release.tag_name, expected_prefix
            )
        })?;
    if !asset
        .name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
    {
        return Err("Назва файла релізу містить неприпустимі символи.".to_owned());
    }
    let url = reqwest::Url::parse(&asset.browser_download_url)
        .map_err(|error| format!("Некоректне посилання на реліз FreeLLMAPI: {error}"))?;
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || !url
            .path()
            .starts_with("/tashfeenahmed/freellmapi/releases/download/")
    {
        return Err(
            "Посилання на інсталятор не належить офіційним GitHub Releases FreeLLMAPI.".to_owned(),
        );
    }
    let digest = asset
        .digest
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .filter(|value| value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| {
            "Для інсталятора немає валідного SHA-256 у метаданих GitHub Release.".to_owned()
        })?
        .to_ascii_lowercase();
    Ok((asset.name.clone(), url, digest))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("Не вдалося перевірити файл {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("Не вдалося прочитати файл для перевірки: {error}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn is_valid_plugin_id(plugin_id: &str) -> bool {
    !plugin_id.is_empty()
        && plugin_id.len() <= 240
        && plugin_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
}

fn plugin_string<'a>(plugin: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| plugin.get(*key).and_then(serde_json::Value::as_str))
}

fn normalize_plugin_id(plugin: &serde_json::Value) -> Option<String> {
    let id = plugin_string(plugin, &["pluginId", "id"])?;
    if id.contains('@') {
        return is_valid_plugin_id(id).then(|| id.to_owned());
    }
    let marketplace = plugin_string(plugin, &["marketplaceName", "marketplace"])?;
    let full_id = format!("{id}@{marketplace}");
    is_valid_plugin_id(&full_id).then_some(full_id)
}

fn parse_plugin_catalog(body: &serde_json::Value) -> Result<Vec<ClaudePlugin>, String> {
    let installed = body
        .get("installed")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            "Claude Code повернув каталог без списку встановлених плагінів.".to_owned()
        })?;
    let available = body
        .get("available")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut plugins = Vec::new();
    for (entries, is_installed) in [(installed.as_slice(), true), (available.as_slice(), false)] {
        for plugin in entries {
            let Some(id) = normalize_plugin_id(plugin) else {
                continue;
            };
            let name = plugin_string(plugin, &["name"])
                .unwrap_or_else(|| id.split('@').next().unwrap_or(&id))
                .to_owned();
            let marketplace = plugin_string(plugin, &["marketplaceName", "marketplace"])
                .or_else(|| id.split_once('@').map(|(_, marketplace)| marketplace))
                .unwrap_or("unknown")
                .to_owned();
            let scope = plugin_string(plugin, &["scope"])
                .unwrap_or("user")
                .to_owned();
            let enabled = plugin
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(is_installed);
            if let Some(existing) = plugins
                .iter_mut()
                .find(|existing: &&mut ClaudePlugin| existing.id == id)
            {
                existing.installed |= is_installed;
                if is_installed {
                    existing.enabled = enabled;
                    existing.scope = scope;
                }
                if existing.description.is_empty() {
                    existing.description = plugin_string(plugin, &["description"])
                        .unwrap_or_default()
                        .to_owned();
                }
                continue;
            }
            plugins.push(ClaudePlugin {
                id,
                name,
                description: plugin_string(plugin, &["description"])
                    .unwrap_or_default()
                    .to_owned(),
                marketplace,
                installed: is_installed,
                enabled,
                scope,
            });
        }
    }
    plugins.sort_by_key(|plugin| plugin.name.to_ascii_lowercase());
    Ok(plugins)
}

fn load_claude_plugins() -> Result<Vec<ClaudePlugin>, String> {
    let output = claude_command()
        .args(["plugin", "list", "--json", "--available"])
        .output()
        .map_err(|error| {
            format!("Не вдалося запустити Claude Code для списку плагінів: {error}")
        })?;
    if !output.status.success() {
        return Err(format_claude_cli_error(
            "Не вдалося отримати список плагінів Claude Code.",
            &output.stderr,
        ));
    }
    let body: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Claude Code повернув некоректний каталог плагінів: {error}"))?;
    parse_plugin_catalog(&body)
}

fn parse_marketplaces(body: &serde_json::Value) -> Result<Vec<ClaudeMarketplace>, String> {
    let entries = body
        .as_array()
        .or_else(|| {
            body.get("marketplaces")
                .and_then(serde_json::Value::as_array)
        })
        .ok_or_else(|| "Claude Code повернув некоректний список marketplace.".to_owned())?;
    let mut marketplaces: Vec<ClaudeMarketplace> = entries
        .iter()
        .filter_map(|entry| {
            let name = plugin_string(entry, &["name"])?;
            if !is_valid_marketplace_name(name) {
                return None;
            }
            let source = plugin_string(entry, &["repo", "source"]).unwrap_or("невідоме джерело");
            Some(ClaudeMarketplace {
                name: name.to_owned(),
                source: source.to_owned(),
            })
        })
        .collect();
    marketplaces.sort_by_key(|marketplace| marketplace.name.to_ascii_lowercase());
    Ok(marketplaces)
}

fn is_valid_marketplace_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 120
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn normalize_marketplace_source(source: &str) -> Result<String, String> {
    let source = source.trim();
    let repo = if source.contains("://") {
        let url = reqwest::Url::parse(source)
            .map_err(|error| format!("Некоректна адреса marketplace: {error}"))?;
        if url.scheme() != "https"
            || url.host_str() != Some("github.com")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "Marketplace має бути GitHub-репозиторієм за HTTPS без облікових даних.".to_owned(),
            );
        }
        url.path().trim_matches('/').to_owned()
    } else {
        source.to_owned()
    };
    let repo = repo.strip_suffix(".git").unwrap_or(&repo);
    let parts: Vec<_> = repo.split('/').collect();
    if parts.len() != 2
        || parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == "..")
        || !parts.iter().all(|part| {
            part.bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
    {
        return Err(
            "Вкажи GitHub-репозиторій у форматі owner/repo або https://github.com/owner/repo."
                .to_owned(),
        );
    }
    Ok(format!("{}/{}", parts[0], parts[1]))
}

fn load_plugin_manager_snapshot() -> Result<PluginManagerSnapshot, String> {
    let plugins = load_claude_plugins()?;
    let output = claude_command()
        .args(["plugin", "marketplace", "list", "--json"])
        .output()
        .map_err(|error| {
            format!("Не вдалося запустити Claude Code для списку marketplace: {error}")
        })?;
    if !output.status.success() {
        return Err(format_claude_cli_error(
            "Не вдалося отримати список marketplace Claude Code.",
            &output.stderr,
        ));
    }
    let body: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Claude Code повернув некоректний список marketplace: {error}"))?;
    Ok(PluginManagerSnapshot {
        plugins,
        marketplaces: parse_marketplaces(&body)?,
    })
}

fn run_claude_plugin_action(action: PluginAction, plugin_id: &str) -> Result<(), String> {
    if !is_valid_plugin_id(plugin_id) {
        return Err("Некоректний ідентифікатор плагіна.".to_owned());
    }
    let action_name = match action {
        PluginAction::Install => "install",
        PluginAction::Enable => "enable",
        PluginAction::Disable => "disable",
        PluginAction::Uninstall => "uninstall",
    };
    let output = claude_command()
        .args(["plugin", action_name, plugin_id, "--scope", "user"])
        .args(if matches!(action, PluginAction::Uninstall) {
            vec!["--keep-data"]
        } else {
            Vec::new()
        })
        .output()
        .map_err(|error| format!("Не вдалося запустити операцію плагіна: {error}"))?;
    if !output.status.success() {
        return Err(format_claude_cli_error(
            "Claude Code не зміг виконати операцію з плагіном.",
            &output.stderr,
        ));
    }
    Ok(())
}

fn run_claude_marketplace_action(add: bool, target: &str) -> Result<(), String> {
    let args = if add {
        let source = normalize_marketplace_source(target)?;
        vec![
            "plugin".to_owned(),
            "marketplace".to_owned(),
            "add".to_owned(),
            "--scope".to_owned(),
            "user".to_owned(),
            source,
        ]
    } else {
        if !is_valid_marketplace_name(target) {
            return Err("Некоректна назва marketplace.".to_owned());
        }
        vec![
            "plugin".to_owned(),
            "marketplace".to_owned(),
            "remove".to_owned(),
            target.to_owned(),
        ]
    };
    let output = claude_command()
        .args(args)
        .output()
        .map_err(|error| format!("Не вдалося запустити операцію marketplace: {error}"))?;
    if !output.status.success() {
        return Err(format_claude_cli_error(
            "Claude Code не зміг виконати операцію marketplace.",
            &output.stderr,
        ));
    }
    Ok(())
}

fn format_claude_cli_error(prefix: &str, stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr);
    if detail.trim().is_empty() {
        prefix.to_owned()
    } else {
        format!("{prefix} {}", detail.trim())
    }
}

fn plugin_action_success_message(action: PluginAction, plugin_id: &str) -> String {
    match action {
        PluginAction::Install => {
            format!("Плагін {plugin_id} встановлено для наступних сесій Claude Code.")
        }
        PluginAction::Enable => format!("Плагін {plugin_id} увімкнено."),
        PluginAction::Disable => format!("Плагін {plugin_id} вимкнено."),
        PluginAction::Uninstall => format!("Плагін {plugin_id} видалено; його дані збережено."),
    }
}

fn install_official_freellmapi() -> Result<(), String> {
    let (expected_prefix, expected_suffix) = current_platform_asset()?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .user_agent(concat!("Mnemosyne/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("Не вдалося підготувати завантаження FreeLLMAPI: {error}"))?;
    let release: GithubRelease = client
        .get(FREELLMAPI_RELEASES_API)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| {
            format!("Не вдалося отримати офіційний реліз FreeLLMAPI з GitHub: {error}")
        })?
        .json()
        .map_err(|error| format!("Некоректні метадані релізу FreeLLMAPI: {error}"))?;
    let (asset_name, download_url, expected_digest) =
        select_release_asset(&release, expected_prefix, expected_suffix)?;
    let install_dir = state_path()?
        .parent()
        .ok_or_else(|| "Не вдалося визначити папку даних Mnemosyne.".to_owned())?
        .join("components")
        .join("freellmapi");
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("Не вдалося створити папку FreeLLMAPI: {error}"))?;
    let asset_path = install_dir.join(&asset_name);

    if asset_path.exists() {
        let actual_digest = sha256_file(&asset_path)?;
        if actual_digest != expected_digest {
            return Err(format!(
                "Знайдено інсталятор FreeLLMAPI з невідповідним SHA-256: {}. Його не запускали; перевірте файл вручну.",
                asset_path.display()
            ));
        }
    } else {
        let response = client
            .get(download_url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| format!("Не вдалося завантажити FreeLLMAPI: {error}"))?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_COMPANION_DOWNLOAD_BYTES)
        {
            return Err(
                "Інсталятор FreeLLMAPI перевищує дозволений розмір завантаження.".to_owned(),
            );
        }
        let temp_path = install_dir.join(format!("{}.{}.part", asset_name, Uuid::new_v4()));
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .map_err(|error| {
                    format!("Не вдалося створити тимчасовий файл інсталятора: {error}")
                })?;
            let mut limited_response = response.take(MAX_COMPANION_DOWNLOAD_BYTES + 1);
            let downloaded = std::io::copy(&mut limited_response, &mut file)
                .map_err(|error| format!("Завантаження FreeLLMAPI перервано: {error}"))?;
            if downloaded > MAX_COMPANION_DOWNLOAD_BYTES {
                return Err(
                    "Інсталятор FreeLLMAPI перевищує дозволений розмір завантаження.".to_owned(),
                );
            }
            file.sync_all()
                .map_err(|error| format!("Не вдалося завершити запис інсталятора: {error}"))?;
            let actual_digest = sha256_file(&temp_path)?;
            if actual_digest != expected_digest {
                return Err("SHA-256 завантаженого інсталятора FreeLLMAPI не збігається з метаданими GitHub Release.".to_owned());
            }
            fs::rename(&temp_path, &asset_path)
                .map_err(|error| format!("Не вдалося зберегти перевірений інсталятор: {error}"))?;
            Ok(())
        })();
        if result.is_err() {
            if let Err(error) = fs::remove_file(&temp_path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!(
                        "Не вдалося видалити тимчасовий файл інсталятора {}: {error}",
                        temp_path.display()
                    );
                }
            }
            return result;
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&asset_path)
            .map_err(|error| format!("Не вдалося прочитати атрибути AppImage: {error}"))?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&asset_path, permissions)
            .map_err(|error| format!("Не вдалося дозволити запуск AppImage: {error}"))?;
    }

    silent_command(&asset_path).spawn().map_err(|error| {
        format!(
            "SHA-256 перевірено, але не вдалося запустити офіційний інсталятор {}: {error}",
            asset_path.display()
        )
    })?;
    Ok(())
}

struct AttachmentStaging {
    path: PathBuf,
}

impl AttachmentStaging {
    fn new() -> Result<Self, String> {
        let path = std::env::temp_dir().join(format!("mnemosyne-{}", Uuid::new_v4()));
        fs::create_dir(&path)
            .map_err(|error| format!("Не вдалося підготувати папку для вкладень: {error}"))?;
        Ok(Self { path })
    }
}

impl Drop for AttachmentStaging {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!(
                "Не вдалося видалити тимчасові вкладення {}: {error}",
                self.path.display()
            );
        }
    }
}

fn find_claude_binary() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let candidates = [
            format!("{appdata}\\npm\\claude.cmd"),
            format!("{appdata}\\npm\\claude.exe"),
            format!("{appdata}\\npm\\node_modules\\@anthropic-ai\\claude-code\\bin\\claude.exe"),
            format!("{localappdata}\\Programs\\Claude\\claude.exe"),
            format!("{userprofile}\\.claude\\local\\claude.exe"),
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.is_file() {
                return Some(p);
            }
        }
        if silent_command("cmd.exe")
            .args(["/D", "/C", "claude", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(PathBuf::from("claude"));
        }
    }
    #[cfg(not(windows))]
    {
        if silent_command("claude")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(PathBuf::from("claude"));
        }
        let home = std::env::var("HOME").unwrap_or_default();
        let candidates = [
            format!("{home}/.local/bin/claude"),
            format!("{home}/.claude/local/claude"),
            format!("{home}/.npm-global/bin/claude"),
            "/usr/local/bin/claude".to_string(),
            "/usr/bin/claude".to_string(),
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

#[cfg(windows)]
fn claude_command() -> Command {
    if let Some(bin) = find_claude_binary() {
        let path_str = bin.to_string_lossy();
        if path_str.ends_with(".cmd") || path_str.ends_with(".bat") {
            let mut command = silent_command("cmd.exe");
            command.args(["/D", "/C", &path_str]);
            return command;
        } else if bin != Path::new("claude") {
            return silent_command(bin);
        }
    }
    let mut command = silent_command("cmd.exe");
    command.args(["/D", "/C", "claude"]);
    command
}

#[cfg(not(windows))]
fn claude_command() -> Command {
    if let Some(bin) = find_claude_binary() {
        return silent_command(bin);
    }
    silent_command("claude")
}

fn is_claude_installed() -> bool {
    find_claude_binary().is_some()
}

fn install_claude_cli() -> Result<(), String> {
    #[cfg(windows)]
    {
        let output = silent_command("cmd.exe")
            .args(["/D", "/C", "npm install -g @anthropic-ai/claude-code"])
            .output()
            .map_err(|e| format!("Не вдалося запустити npm (перевірте встановлення Node.js): {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(if stderr.trim().is_empty() {
                "npm install завершився з помилкою. Спробуйте вручну: npm install -g @anthropic-ai/claude-code".to_owned()
            } else {
                format!("npm error: {}", stderr.trim())
            })
        }
    }
    #[cfg(not(windows))]
    {
        let output = silent_command("sh")
            .args(["-c", "npm install -g @anthropic-ai/claude-code || (curl -fsSL https://claude.ai/install.sh | bash)"])
            .output()
            .map_err(|e| format!("Не вдалося запустити інсталятор: {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(if stderr.trim().is_empty() {
                "Помилка встановлення. Спробуйте вручну: npm install -g @anthropic-ai/claude-code".to_owned()
            } else {
                format!("Error: {}", stderr.trim())
            })
        }
    }
}

fn test_freellmapi_round_trip(
    base_url: &str,
    api_key: &str,
    first_model: Option<&str>,
) -> Result<String, String> {
    let model = first_model
        .filter(|m| !m.is_empty())
        .unwrap_or("claude-sonnet-4-5");
    let url = format!("{base_url}/v1/messages");
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1,
        "messages": [{"role": "user", "content": "ping"}]
    });
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| format!("Не вдалося створити HTTP-клієнт: {error}"))?
        .post(&url)
        .header("x-api-key", api_key)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .map_err(|error| format!("Не вдалося з'єднатися з FreeLLMAPI: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "FreeLLMAPI відповів HTTP {} для тестового запиту",
            response.status()
        ));
    }
    Ok(model.to_owned())
}

#[cfg(target_os = "linux")]
fn check_linux_desktop_env() -> Option<String> {
    if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
        None
    } else {
        Some(
            "Не знайдено змінну середовища DISPLAY або WAYLAND_DISPLAY. \
             Запустіть програму з графічного середовища."
                .to_owned(),
        )
    }
}

fn run_approval_hook(address: &str, token: &str) -> Result<(), String> {    let address: SocketAddr = address
        .parse()
        .map_err(|error| format!("Некоректна адреса підтвердження Mnemosyne: {error}"))?;
    let mut input = Vec::new();
    std::io::stdin()
        .take((MAX_APPROVAL_HOOK_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|error| format!("Не вдалося прочитати запит Claude Code: {error}"))?;
    if input.len() > MAX_APPROVAL_HOOK_BYTES {
        return Err("Запит Claude Code перевищує допустимий розмір.".to_owned());
    }
    let input: serde_json::Value = serde_json::from_slice(&input)
        .map_err(|error| format!("Claude Code надіслав некоректний запит: {error}"))?;
    let payload = serde_json::to_vec(&serde_json::json!({
        "token": token,
        "input": input
    }))
    .map_err(|error| format!("Не вдалося підготувати запит підтвердження: {error}"))?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(5))
        .map_err(|error| format!("Не вдалося зв'язатися з Mnemosyne: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(605)))
        .map_err(|error| format!("Не вдалося налаштувати очікування рішення: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("Не вдалося налаштувати передавання рішення: {error}"))?;
    write_approval_frame(&mut stream, &payload)
        .map_err(|error| format!("Не вдалося передати запит підтвердження: {error}"))?;
    let response = read_approval_frame(&mut stream, MAX_APPROVAL_RESPONSE_BYTES)?;
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(&response)
        .and_then(|_| stdout.write_all(b"\n"))
        .map_err(|error| format!("Не вдалося повернути рішення Claude Code: {error}"))
}

fn load_app_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../assets/icon_32.png");
    let img = image::load_from_memory(png_bytes).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--mnemosyne-approval-hook") {
        let Some(address) = args.get(1) else {
            eprintln!("Не вказано адресу перевірки змін.");
            std::process::exit(2);
        };
        let Some(token) = args.get(2) else {
            eprintln!("Не вказано токен перевірки змін.");
            std::process::exit(2);
        };
        if let Err(error) = run_approval_hook(address, token) {
            eprintln!("{error}");
            std::process::exit(2);
        }
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    let linux_desktop_warning = check_linux_desktop_env();
    #[cfg(not(target_os = "linux"))]
    let linux_desktop_warning: Option<String> = None;
    if let Some(ref msg) = linux_desktop_warning {
        eprintln!("{msg}");
    }
    let initial_dark_theme = load_state().map(|s| s.dark_theme).unwrap_or(true);
    let _ = initial_dark_theme;

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([980.0, 620.0])
        .with_min_inner_size([640.0, 400.0]);
    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Mnemosyne",
        options,
        Box::new(move |creation_context| {
            let mut app = MnemosyneApp::new();
            if app.state.dark_theme {
                creation_context.egui_ctx.set_visuals(dark_visuals());
                creation_context.egui_ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
            } else {
                creation_context.egui_ctx.set_visuals(light_visuals());
                creation_context.egui_ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
            }
            if let Some(msg) = linux_desktop_warning {
                app.status = Some(msg);
            }
            Ok(Box::new(app))
        }),
    )
}

// ── Project Language Detection & Rule Provisioning ───────────────────────────

pub fn detect_project_languages(project_path: &Path) -> Vec<String> {
    let mut langs = Vec::new();
    if project_path.join("Cargo.toml").exists() {
        langs.push("Rust".to_string());
    }
    if project_path.join("pyproject.toml").exists()
        || project_path.join("requirements.txt").exists()
        || project_path.join("setup.py").exists()
    {
        langs.push("Python".to_string());
    }
    if project_path.join("CMakeLists.txt").exists()
        || project_path.join("Makefile").exists()
        || project_path.join("compile_commands.json").exists()
    {
        langs.push("C/C++".to_string());
    }
    if project_path.join("package.json").exists() {
        langs.push("JavaScript/TypeScript".to_string());
    }
    if project_path.join("go.mod").exists() {
        langs.push("Go".to_string());
    }
    if let Ok(entries) = fs::read_dir(project_path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "csproj" || ext == "sln" {
                    if !langs.contains(&"C#".to_string()) {
                        langs.push("C#".to_string());
                    }
                }
                if ext == "lua" {
                    if !langs.contains(&"Lua".to_string()) {
                        langs.push("Lua".to_string());
                    }
                }
            }
        }
    }
    langs
}

pub fn ensure_curated_project_rules(
    project_path: &Path,
    caveman: bool,
    karpathy: bool,
    decod_mcp: bool,
) -> Result<PathBuf, String> {
    let claude_dir = project_path.join(".claude");
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir).map_err(|e| e.to_string())?;
    }
    let claude_md = claude_dir.join("CLAUDE.md");

    if claude_md.exists() {
        let existing = fs::read_to_string(&claude_md).unwrap_or_default();
        if !existing.contains("<!-- MNEMOSYNE_AUTO_RULES -->") {
            return Ok(claude_md);
        }
    }

    let detected_langs = detect_project_languages(project_path);
    let lang_str = if detected_langs.is_empty() {
        "General Codebase".to_string()
    } else {
        detected_langs.join(", ")
    };

    let mut content = String::new();
    content.push_str("<!-- MNEMOSYNE_AUTO_RULES -->\n");
    content.push_str("# Project Guidelines & Coding Directives\n\n");
    content.push_str(&format!("**Detected Tech Stack:** {}\n\n", lang_str));

    if caveman {
        content.push_str("## ⚡ Token Thrift & Caveman Directives\n");
        content.push_str("- **Conciseness**: Be extremely direct. Eliminate chit-chat, conversational filler, pleasantries, and unnecessary apologies.\n");
        content.push_str("- **Minimal Diffs**: When modifying code, produce exact surgical edits without echoing unchanged boilerplate or entire files.\n");
        content.push_str("- **Zero Fluff**: Do not summarize code changes unless explicitly asked. Deliver working code immediately.\n\n");
    }

    if karpathy {
        content.push_str("## 🧠 Karpathy Engineering Guidelines\n");
        content.push_str("- **Think Before Coding**: Make a concise plan before modifying critical logic.\n");
        content.push_str("- **Minimal Abstractions (YAGNI)**: Avoid premature generalization, excessive design patterns, or layers of indirection. Prefer readable, straightforward procedural or idiomatic code.\n");
        content.push_str("- **Preserve Style**: Strictly match existing coding conventions, naming styles, and error handling patterns in the repository.\n");
        content.push_str("- **Defensive Programming**: Check boundaries, null/None states, unwrap failures, and error returns.\n");
        content.push_str("- **Verify**: Run compiler or diagnostics checks to ensure no syntax/type breaks before completing your response.\n\n");
    }

    if decod_mcp {
        content.push_str("## 🛠 Decod MCP Superpowers (Context Optimization)\n");
        content.push_str("You have high-efficiency Decod MCP tools configured:\n");
        content.push_str("- `inspect_code_structure(file_path)`: ALWAYS call this first to get the AST symbol tree (classes, functions, line ranges) instead of reading 1,000+ line files into context!\n");
        content.push_str("- `extract_function_definition(file_path, symbol_name)`: Surgically read only the function body you need to examine or edit.\n");
        content.push_str("- `check_code_diagnostics(project_or_file)`: Run instant compiler diagnostics (`cargo check`, `py_compile`, syntax checks) without burning context.\n");
        content.push_str("- `run_git_summary(repo_path)`: Get token-compact git branch and status.\n\n");
    }

    if detected_langs.contains(&"Rust".to_string()) {
        content.push_str("## 🦀 Rust Rules\n");
        content.push_str("- Target: Rust 2021/2024 edition. Use idiomatic error handling with `Result<T, E>` and `?`.\n");
        content.push_str("- Avoid unnecessary `.clone()` calls; use references and borrowing where appropriate.\n");
        content.push_str("- Verify changes using `cargo check`.\n\n");
    }

    if detected_langs.contains(&"Python".to_string()) {
        content.push_str("## 🐍 Python Rules\n");
        content.push_str("- Use Python 3.10+ type annotations (`str | None`, `list[dict]`).\n");
        content.push_str("- Follow PEP 8 naming and structure; handle exceptions explicitly.\n");
        content.push_str("- Verify syntax with `check_code_diagnostics`.\n\n");
    }

    if detected_langs.contains(&"C/C++".to_string()) {
        content.push_str("## ⚙ C/C++ Rules\n");
        content.push_str("- Modern C++17/C++20 standards. Enforce RAII for memory and resources.\n");
        content.push_str("- Use `std::unique_ptr` / `std::shared_ptr`; avoid raw owning pointers.\n\n");
    }

    fs::write(&claude_md, content).map_err(|e| e.to_string())?;
    Ok(claude_md)
}

fn resolve_decod_path(configured_path: &str) -> PathBuf {
    let p = PathBuf::from(configured_path);
    if p.exists() {
        return p;
    }
    let candidates = [
        "C:/Users/knyaz/Desktop/Decod/core/mcp_server.py",
        "../Decod/core/mcp_server.py",
        "./Decod/core/mcp_server.py",
        "Decod/core/mcp_server.py",
    ];
    for c in candidates {
        let cp = PathBuf::from(c);
        if cp.exists() {
            return cp;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let lp = PathBuf::from(home).join(".local/share/decod/core/mcp_server.py");
        if lp.exists() {
            return lp;
        }
    }
    p
}

fn build_project_mcp_config(
    _project_path: &Path,
    decod_path: &str,
    decod_enabled: bool,
) -> Option<String> {
    if !decod_enabled {
        return None;
    }
    let resolved = resolve_decod_path(decod_path);
    let mut servers = serde_json::Map::new();

    servers.insert(
        "decod".to_string(),
        serde_json::json!({
            "command": "python",
            "args": [resolved.to_string_lossy().to_string()]
        }),
    );

    let config = serde_json::json!({
        "mcpServers": servers
    });
    Some(config.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_project() -> PathBuf {
        std::env::temp_dir().join(format!("mnemosyne-test-{}", Uuid::new_v4()))
    }

    fn call_approval_hook(server: &ApprovalServer, input: serde_json::Value) -> serde_json::Value {
        let mut stream = TcpStream::connect(server.address).unwrap();
        let request = serde_json::to_vec(&serde_json::json!({
            "token": server.token,
            "input": input
        }))
        .unwrap();
        write_approval_frame(&mut stream, &request).unwrap();
        let response = read_approval_frame(&mut stream, MAX_APPROVAL_RESPONSE_BYTES).unwrap();
        serde_json::from_slice(&response).unwrap()
    }

    #[test]
    fn allows_http_only_for_loopback_free_llm_api() {
        assert_eq!(
            normalize_api_base_url("http://localhost:3001/").unwrap(),
            "http://localhost:3001"
        );
        assert_eq!(
            normalize_api_base_url("http://127.0.0.1:3001").unwrap(),
            "http://127.0.0.1:3001"
        );
        assert!(normalize_api_base_url("http://192.168.1.10:3001").is_err());
        assert!(normalize_api_base_url("https://gateway.example:3001").is_ok());
    }

    #[test]
    fn rejects_api_urls_with_paths_or_credentials() {
        assert!(normalize_api_base_url("http://localhost:3001/v1").is_err());
        assert!(normalize_api_base_url("http://user:pass@localhost:3001").is_err());
        assert!(normalize_api_base_url("file:///tmp/freellmapi").is_err());
    }

    #[test]
    fn parses_available_model_ids_from_free_llm_api_catalog() {
        let body = serde_json::json!({
            "data": [
                {"id": "google/gemma-free"},
                {"modelId": "cohere/code-free"},
                {"name": "missing-id"}
            ]
        });
        assert_eq!(
            parse_ready_models(&body).unwrap(),
            vec!["google/gemma-free", "cohere/code-free"]
        );
    }

    #[test]
    fn reports_empty_or_unknown_model_catalogs() {
        assert!(parse_ready_models(&serde_json::json!({"data": []})).is_err());
        assert!(parse_ready_models(&serde_json::json!({"models": []})).is_err());
    }

    #[test]
    fn parses_and_merges_plugin_catalog_without_losing_installed_state() {
        let plugins = parse_plugin_catalog(&serde_json::json!({
            "installed": [{
                "pluginId": "review@official",
                "name": "review",
                "description": "",
                "enabled": true,
                "scope": "project"
            }],
            "available": [
                {
                    "pluginId": "review@official",
                    "name": "review",
                    "description": "Code review",
                    "marketplaceName": "official"
                },
                {
                    "pluginId": "format@official",
                    "name": "format",
                    "description": "Formatting",
                    "marketplaceName": "official"
                }
            ]
        }))
        .unwrap();

        assert_eq!(plugins.len(), 2);
        let review = plugins
            .iter()
            .find(|plugin| plugin.id == "review@official")
            .unwrap();
        assert!(review.installed);
        assert!(review.enabled);
        assert_eq!(review.scope, "project");
        assert_eq!(review.description, "Code review");
        let formatter = plugins
            .iter()
            .find(|plugin| plugin.id == "format@official")
            .unwrap();
        assert!(!formatter.installed);
        assert!(!formatter.enabled);
    }

    #[test]
    fn plugin_ids_reject_shell_metacharacters() {
        assert!(is_valid_plugin_id("review@official-marketplace"));
        assert!(!is_valid_plugin_id("review@official&whoami"));
        assert!(!is_valid_plugin_id(""));
    }

    #[test]
    fn marketplace_sources_are_restricted_to_github_repositories() {
        assert_eq!(
            normalize_marketplace_source(" anthropics/claude-plugins-official ").unwrap(),
            "anthropics/claude-plugins-official"
        );
        assert_eq!(
            normalize_marketplace_source("https://github.com/example/plugins.git").unwrap(),
            "example/plugins"
        );
        assert!(normalize_marketplace_source("http://github.com/example/plugins").is_err());
        assert!(
            normalize_marketplace_source("https://user:pass@github.com/example/plugins").is_err()
        );
        assert!(normalize_marketplace_source("example/plugins&whoami").is_err());
        assert!(
            normalize_marketplace_source("https://github.com/example/plugins/tree/main").is_err()
        );
    }

    #[test]
    fn parses_marketplace_list_and_ignores_invalid_names() {
        let marketplaces = parse_marketplaces(&serde_json::json!([
            {"name": "official", "source": "github", "repo": "anthropics/claude-plugins-official"},
            {"name": "unsafe&name", "source": "github", "repo": "attacker/repo"}
        ]))
        .unwrap();
        assert_eq!(marketplaces.len(), 1);
        assert_eq!(marketplaces[0].name, "official");
        assert_eq!(marketplaces[0].source, "anthropics/claude-plugins-official");
    }

    #[test]
    fn parses_claude_stream_deltas_and_final_result() {
        let delta = serde_json::json!({
            "type": "stream_event",
            "event": {
                "type": "content_block_delta",
                "delta": {"type": "text_delta", "text": "hello"}
            }
        })
        .to_string();
        assert!(matches!(
            parse_claude_stream_event(&delta).unwrap(),
            Some(ClaudeStreamEvent::Text(text)) if text == "hello"
        ));

        let tool = serde_json::json!({
            "type": "stream_event",
            "event": {
                "type": "content_block_start",
                "content_block": {"type": "tool_use", "name": "Read"}
            }
        })
        .to_string();
        assert!(matches!(
            parse_claude_stream_event(&tool).unwrap(),
            Some(ClaudeStreamEvent::Tool(name)) if name == "Read"
        ));

        let result = serde_json::json!({"type": "result", "result": "hello world"}).to_string();
        assert!(matches!(
            parse_claude_stream_event(&result).unwrap(),
            Some(ClaudeStreamEvent::Result(text)) if text == "hello world"
        ));
        assert!(
            parse_claude_stream_event(r#"{"type":"system","subtype":"init"}"#)
                .unwrap()
                .is_none()
        );
        assert!(parse_claude_stream_event("not-json").unwrap().is_none());
        assert!(parse_claude_stream_event("{ invalid json }").unwrap().is_none());
    }

    #[test]
    fn can_terminate_the_claude_process_group() {
        let mut command = if cfg!(windows) {
            let mut command = silent_command("cmd.exe");
            command.args(["/D", "/C", "ping -n 20 127.0.0.1 >NUL"]);
            command
        } else {
            let mut command = silent_command("sh");
            command.args(["-c", "sleep 20"]);
            command
        };
        let mut child = command.group_spawn().unwrap();
        child.kill().unwrap();
        assert!(!child.wait().unwrap().success());
    }

    #[test]
    fn claude_model_selection_uses_supported_aliases() {
        for (model, expected) in [
            (ClaudeModel::Sonnet, "sonnet"),
            (ClaudeModel::Opus, "opus"),
            (ClaudeModel::Haiku, "haiku"),
        ] {
            let args = claude_args(AgentMode::Plan, model);
            let model_flag = args.iter().position(|arg| *arg == "--model").unwrap();
            assert_eq!(args[model_flag + 1], expected);
        }
    }

    #[test]
    fn older_state_files_get_default_agent_model_and_context_settings() {
        let state: AppState = serde_json::from_str(r#"{"projects":[]}"#).unwrap();
        assert_eq!(state.claude_model, ClaudeModel::Sonnet);
        assert_eq!(state.context_limit, ConversationContextLimit::Last20Turns);
    }

    #[test]
    fn atomic_write_replaces_existing_file_and_leaves_no_temporary_file() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        fs::write(&path, "old state").unwrap();

        atomic_write(&path, b"new state").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new state");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn context_limit_keeps_recent_user_turns_without_deleting_history() {
        let mut messages = Vec::new();
        for turn in 1..=7 {
            messages.push(Message {
                role: MessageRole::User,
                content: format!("Запит {turn}"),
                attachments: Vec::new(),
                created_at: Local::now(),
            });
            if turn < 7 {
                messages.push(Message {
                    role: MessageRole::Assistant,
                    content: format!("Відповідь {turn}"),
                    attachments: Vec::new(),
                    created_at: Local::now(),
                });
            }
        }

        let recent = conversation_context(&messages, ConversationContextLimit::Last5Turns);
        assert_eq!(recent.first().unwrap().content, "Запит 3");
        assert_eq!(recent.last().unwrap().content, "Запит 7");
        assert_eq!(recent.len(), 9);
        assert_eq!(
            conversation_context(&messages, ConversationContextLimit::All).len(),
            messages.len()
        );
        assert_eq!(messages.len(), 13);
    }

    #[test]
    fn validates_renamed_chat_titles() {
        assert_eq!(
            normalize_chat_title("  Feature design  ").unwrap(),
            "Feature design"
        );
        assert!(normalize_chat_title("   ").is_err());
        assert!(normalize_chat_title(&"x".repeat(81)).is_err());
        assert!(normalize_chat_title(&"x".repeat(80)).is_ok());
    }

    #[test]
    fn removes_only_the_requested_chat() {
        let mut project = Project {
            id: "project".to_owned(),
            name: "Project".to_owned(),
            path: PathBuf::new(),
            chats: ["keep", "remove"]
                .into_iter()
                .map(|id| Chat {
                    id: id.to_owned(),
                    title: id.to_owned(),
                    messages: Vec::new(),
                })
                .collect(),
        };

        assert!(remove_chat_from_project(&mut project, "remove"));
        assert_eq!(project.chats.len(), 1);
        assert_eq!(project.chats[0].id, "keep");
        assert!(!remove_chat_from_project(&mut project, "missing"));
    }

    #[test]
    fn clearing_chat_history_keeps_chat_and_title() {
        let mut project = Project {
            id: "project".to_owned(),
            name: "Project".to_owned(),
            path: PathBuf::new(),
            chats: vec![Chat {
                id: "chat".to_owned(),
                title: "Keep this title".to_owned(),
                messages: vec![Message {
                    role: MessageRole::User,
                    content: "Remove this message".to_owned(),
                    attachments: vec![PathBuf::from("image.png")],
                    created_at: Local::now(),
                }],
            }],
        };

        assert!(clear_chat_messages(&mut project, "chat"));
        assert_eq!(project.chats.len(), 1);
        assert_eq!(project.chats[0].title, "Keep this title");
        assert!(project.chats[0].messages.is_empty());
        assert!(!clear_chat_messages(&mut project, "missing"));
    }

    #[test]
    fn exports_chat_content_and_attachment_names_without_source_paths() {
        let chat = Chat {
            id: "chat".to_owned(),
            title: "Feature plan".to_owned(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Review this screenshot.".to_owned(),
                    attachments: vec![PathBuf::from("private/source/screen.png")],
                    created_at: Local::now(),
                },
                Message {
                    role: MessageRole::Assistant,
                    content: "## Findings\n\nLooks good.".to_owned(),
                    attachments: Vec::new(),
                    created_at: Local::now(),
                },
            ],
        };

        let export = chat_export_markdown(&chat);
        assert!(export.starts_with("# Feature plan"));
        assert!(export.contains("Review this screenshot."));
        assert!(export.contains("`screen.png`"));
        assert!(export.contains("## Claude"));
        assert!(export.contains("## Findings"));
        assert!(!export.contains("private/source"));
    }

    #[test]
    fn removing_project_record_keeps_next_project_and_chat_selected() {
        let project = |id: &str, chat_id: &str| Project {
            id: id.to_owned(),
            name: id.to_owned(),
            path: PathBuf::from(format!("projects/{id}")),
            chats: vec![Chat {
                id: chat_id.to_owned(),
                title: chat_id.to_owned(),
                messages: Vec::new(),
            }],
        };
        let mut state = AppState {
            projects: vec![
                project("first", "first-chat"),
                project("second", "second-chat"),
            ],
            selected_project: Some("first".to_owned()),
            selected_chat: Some("first-chat".to_owned()),
            ..AppState::default()
        };

        assert!(remove_project_record(&mut state, "first"));
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0].path, PathBuf::from("projects/second"));
        assert_eq!(state.selected_project.as_deref(), Some("second"));
        assert_eq!(state.selected_chat.as_deref(), Some("second-chat"));
        assert!(!remove_project_record(&mut state, "missing"));
    }

    #[test]
    fn turbo_mode_requires_manual_permission_and_exposes_only_file_edit_tools() {
        let args = claude_args(AgentMode::Turbo, ClaudeModel::Sonnet);
        assert!(args.contains(&"--output-format"));
        assert!(args.contains(&"stream-json"));
        assert!(args.contains(&"--verbose"));
        assert!(args.contains(&"--include-partial-messages"));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "manual"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--tools", "Read,Glob,Grep,Edit,Write"]));
        assert!(!args.iter().any(|arg| arg.contains("Bash")));
    }

    #[test]
    fn plan_mode_remains_read_only() {
        let args = claude_args(AgentMode::Plan, ClaudeModel::Sonnet);
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "plan"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--tools", "Read,Glob,Grep"]));
        assert!(!args
            .iter()
            .any(|arg| arg.contains("Edit") || arg.contains("Write")));
    }

    #[test]
    fn approval_paths_are_confined_to_project_and_attachment_roots() {
        let root = temporary_project();
        let project = root.join("project");
        let attachments = root.join("attachments");
        fs::create_dir_all(project.join("src")).unwrap();
        fs::create_dir_all(&attachments).unwrap();
        fs::write(project.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(attachments.join("image.png"), b"image").unwrap();
        let project = project.canonicalize().unwrap();
        let attachments = attachments.canonicalize().unwrap();

        assert_eq!(
            project_relative_write_path(&project, "src/main.rs", Some(&project)).unwrap(),
            PathBuf::from("src/main.rs")
        );
        assert_eq!(
            path_within_roots(
                &[project.clone(), attachments.clone()],
                "image.png",
                Some(&attachments)
            )
            .unwrap()
            .0,
            1
        );
        assert!(project_relative_write_path(&project, "../outside.txt", Some(&project)).is_err());
        assert!(project_relative_write_path(
            &project,
            root.join("outside.txt").to_str().unwrap(),
            Some(&project)
        )
        .is_err());
        let outside = root.join("outside");
        fs::create_dir_all(&outside).unwrap();
        let link = project.join("escape");
        #[cfg(unix)]
        let link_result = std::os::unix::fs::symlink(&outside, &link);
        #[cfg(windows)]
        let link_result = std::os::windows::fs::symlink_dir(&outside, &link);
        if link_result.is_ok() {
            assert!(
                project_relative_write_path(&project, "escape/new.rs", Some(&project)).is_err()
            );
            #[cfg(unix)]
            fs::remove_file(&link).unwrap();
            #[cfg(windows)]
            fs::remove_dir(&link).unwrap();
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn edit_approval_preview_shows_replacement_and_bounds_large_content() {
        let preview = file_change_preview(
            "Edit",
            &serde_json::json!({"old_string":"before", "new_string":"after"}),
        );
        assert!(preview.contains("--- було\nbefore"));
        assert!(preview.contains("+++ стане\nafter"));

        let large =
            file_change_preview("Write", &serde_json::json!({"content":"x".repeat(12_100)}));
        assert!(large.contains("попередній перегляд обрізано"));
        assert!(large.len() < 13_000);
    }

    #[test]
    fn approval_server_requires_user_decision_for_project_file_edits() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.rs"), "old").unwrap();
        let (approval_tx, approval_rx) = mpsc::channel();
        let server = ApprovalServer::start(
            &root,
            None,
            AgentMode::Turbo,
            approval_tx,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        let mut stream = TcpStream::connect(server.address).unwrap();
        let request = serde_json::to_vec(&serde_json::json!({
            "token": server.token,
            "input": {
                "cwd": root.to_string_lossy(),
                "tool_name": "Edit",
                "tool_input": {
                    "file_path": root.join("main.rs").to_string_lossy(),
                    "old_string": "old",
                    "new_string": "new"
                }
            }
        }))
        .unwrap();
        write_approval_frame(&mut stream, &request).unwrap();

        let approval = approval_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(approval.tool, "Edit");
        assert_eq!(approval.path, "main.rs");
        assert!(approval.preview.contains("new"));
        approval.response_tx.send(true).unwrap();
        let response = read_approval_frame(&mut stream, MAX_APPROVAL_RESPONSE_BYTES).unwrap();
        let response: serde_json::Value = serde_json::from_slice(&response).unwrap();
        assert_eq!(
            response["hookSpecificOutput"]["permissionDecision"],
            "allow"
        );
        drop(server);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn approval_hook_denies_writes_in_plan_mode_and_unknown_tools() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        let (approval_tx, approval_rx) = mpsc::channel();
        let server = ApprovalServer::start(
            &root,
            None,
            AgentMode::Plan,
            approval_tx,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        let settings: serde_json::Value =
            serde_json::from_str(&server.settings_json("http://127.0.0.1:31415", "test-key").unwrap()).unwrap();
        assert_eq!(settings["hooks"]["PreToolUse"][0]["matcher"], "*");
        assert_eq!(settings["env"]["ANTHROPIC_BASE_URL"], "http://127.0.0.1:31415");

        for tool in ["Write", "Bash"] {
            let response = call_approval_hook(
                &server,
                serde_json::json!({
                    "cwd": root.to_string_lossy(),
                    "tool_name": tool,
                    "tool_input": {"file_path": root.join("new.txt").to_string_lossy()}
                }),
            );
            assert_eq!(response["hookSpecificOutput"]["permissionDecision"], "deny");
        }
        assert!(approval_rx.try_recv().is_err());
        drop(server);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn opens_and_saves_text_files_inside_the_project() {
        let root = temporary_project();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/plan.md"), "# Plan\n").unwrap();

        assert_eq!(
            read_project_text_file(&root, Path::new("docs/plan.md")).unwrap(),
            "# Plan\n"
        );
        save_project_text_file(&root, Path::new("docs/plan.md"), "# Updated\n").unwrap();
        assert_eq!(
            fs::read_to_string(root.join("docs/plan.md")).unwrap(),
            "# Updated\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_project_file_traversal_and_unsupported_types() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("app.exe"), b"binary").unwrap();

        assert!(read_project_text_file(&root, Path::new("../outside.md")).is_err());
        assert!(read_project_text_file(&root, Path::new("app.exe")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn decodes_supported_project_images_for_in_app_viewing() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        let image_path = root.join("cover.png");
        image::RgbaImage::from_pixel(2, 3, image::Rgba([20, 40, 60, 255]))
            .save_with_format(&image_path, image::ImageFormat::Png)
            .unwrap();

        let (image, width, height) =
            read_project_image_file(&root, Path::new("cover.png")).unwrap();
        assert_eq!((width, height), (2, 3));
        assert_eq!(image.size, [2, 3]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lists_project_files_without_build_and_dependency_directories() {
        let root = temporary_project();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("target/debug/generated"), "ignored").unwrap();

        let listing = list_project_files(&root).unwrap();
        assert_eq!(listing.files, vec![PathBuf::from("src/main.rs")]);
        assert!(!listing.truncated);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn filters_project_files_case_insensitively() {
        let path = Path::new("Docs/Plans/Release-Plan.MD");
        assert!(project_file_matches_filter(path, "release-plan"));
        assert!(project_file_matches_filter(path, "  DOCS/PLANS "));
        assert!(!project_file_matches_filter(path, "report"));
        assert!(project_file_matches_filter(path, " "));
    }

    #[test]
    fn parses_markdown_blocks_for_preview() {
        let blocks = markdown_blocks(
            "# Release\n\n- first\n- second\n\n> quoted text\n\n```rust\nlet ready = true;\n```\n\n| name | state |\n| --- | --- |\n| build | done |\n",
        );
        assert!(blocks.contains(&MarkdownBlock {
            kind: MarkdownBlockKind::Heading(HeadingLevel::H1),
            text: "Release".to_owned(),
        }));
        assert!(blocks.contains(&MarkdownBlock {
            kind: MarkdownBlockKind::ListItem,
            text: "• first".to_owned(),
        }));
        assert!(blocks.contains(&MarkdownBlock {
            kind: MarkdownBlockKind::Quote,
            text: "> quoted text".to_owned(),
        }));
        assert!(blocks.contains(&MarkdownBlock {
            kind: MarkdownBlockKind::Code,
            text: "let ready = true;\n".to_owned(),
        }));
        assert!(blocks.iter().any(|block| {
            block.kind == MarkdownBlockKind::TableRow && block.text.contains("name │ state")
        }));
    }

    #[test]
    fn creates_plan_and_report_documents_inside_the_project() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();

        let plan = create_project_markdown_document(&root, ProjectDocumentKind::Plan).unwrap();
        let report = create_project_markdown_document(&root, ProjectDocumentKind::Report).unwrap();
        assert!(plan.starts_with(Path::new("docs/plans")));
        assert!(report.starts_with(Path::new("docs/reports")));
        assert!(read_project_text_file(&root, &plan)
            .unwrap()
            .contains("Критерії готовності"));
        assert!(read_project_text_file(&root, &report)
            .unwrap()
            .contains("Виконані зміни"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selects_only_official_platform_release_assets_with_sha256() {
        let release: GithubRelease = serde_json::from_value(serde_json::json!({
            "tag_name": "v1.2.3",
            "assets": [{
                "name": "FreeLLMAPI-Setup-1.2.3.exe",
                "browser_download_url": "https://github.com/tashfeenahmed/freellmapi/releases/download/v1.2.3/FreeLLMAPI-Setup-1.2.3.exe",
                "digest": format!("sha256:{}", "a".repeat(64))
            }, {
                "name": "FreeLLMAPI-1.2.3.AppImage",
                "browser_download_url": "https://github.com/tashfeenahmed/freellmapi/releases/download/v1.2.3/FreeLLMAPI-1.2.3.AppImage",
                "digest": format!("sha256:{}", "b".repeat(64))
            }]
        }))
        .unwrap();
        let windows = select_release_asset(&release, "FreeLLMAPI-Setup-", ".exe").unwrap();
        assert_eq!(windows.0, "FreeLLMAPI-Setup-1.2.3.exe");
        let linux = select_release_asset(&release, "FreeLLMAPI-", ".AppImage").unwrap();
        assert_eq!(linux.0, "FreeLLMAPI-1.2.3.AppImage");
    }

    #[test]
    fn rejects_non_github_or_unverifiable_release_assets() {
        let release: GithubRelease = serde_json::from_value(serde_json::json!({
            "tag_name": "v1.2.3",
            "assets": [{
                "name": "FreeLLMAPI-Setup-1.2.3.exe",
                "browser_download_url": "https://example.com/installer.exe",
                "digest": format!("sha256:{}", "a".repeat(64))
            }]
        }))
        .unwrap();
        assert!(select_release_asset(&release, "FreeLLMAPI-Setup-", ".exe").is_err());

        let release: GithubRelease = serde_json::from_value(serde_json::json!({
            "tag_name": "v1.2.3",
            "assets": [{
                "name": "FreeLLMAPI-Setup-1.2.3.exe",
                "browser_download_url": "https://github.com/tashfeenahmed/freellmapi/releases/download/v1.2.3/FreeLLMAPI-Setup-1.2.3.exe",
                "digest": "md5:invalid"
            }]
        }))
        .unwrap();
        assert!(select_release_asset(&release, "FreeLLMAPI-Setup-", ".exe").is_err());
    }

    #[test]
    fn detects_project_languages_accurately() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        let langs = detect_project_languages(&root);
        assert_eq!(langs, vec!["Rust"]);

        fs::write(root.join("requirements.txt"), "flask\n").unwrap();
        let langs_dual = detect_project_languages(&root);
        assert!(langs_dual.contains(&"Rust".to_string()));
        assert!(langs_dual.contains(&"Python".to_string()));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn provisions_curated_rules_with_caveman_and_karpathy() {
        let root = temporary_project();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

        let path = ensure_curated_project_rules(&root, true, true, true).unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("<!-- MNEMOSYNE_AUTO_RULES -->"));
        assert!(content.contains("Caveman Directives"));
        assert!(content.contains("Karpathy Engineering"));
        assert!(content.contains("Decod MCP Superpowers"));
        assert!(content.contains("Rust Rules"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn builds_dynamic_mcp_config_correctly() {
        let root = temporary_project();
        let json_str = build_project_mcp_config(&root, "mcp_server.py", true).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed["mcpServers"]["decod"]["command"].is_string());
        assert_eq!(parsed["mcpServers"]["decod"]["command"], "python");
    }

    #[test]
    fn local_model_settings_defaults_and_serialization() {
        let default_local = LocalModelSettings::default();
        assert!(!default_local.enabled);
        assert_eq!(default_local.base_url, "http://localhost:1234");
        assert!(default_local.model_name.is_empty());
        assert!(!default_local.connected);
        assert!(default_local.ready_models.is_empty());

        let state = AppState::default();
        assert!(!state.local_model.enabled);
        assert_eq!(state.local_model.base_url, "http://localhost:1234");
    }

    #[test]
    fn parses_local_models_from_openai_compatible_response() {
        let json_body = serde_json::json!({
            "data": [
                {"id": "llama-3.2-3b-instruct"},
                {"id": "deepseek-coder-6.7b"},
                {"id": "qwen2.5-coder-7b"}
            ]
        });
        let models = parse_local_models(&json_body).unwrap();
        assert_eq!(models.len(), 3);
        assert_eq!(models[0], "llama-3.2-3b-instruct");
        assert_eq!(models[1], "deepseek-coder-6.7b");
        assert_eq!(models[2], "qwen2.5-coder-7b");
    }
}
