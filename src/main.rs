use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use chrono::{DateTime, Local};
use eframe::egui;
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

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct AppState {
    projects: Vec<Project>,
    selected_project: Option<String>,
    selected_chat: Option<String>,
    freellmapi: FreeLlmApiSettings,
    agent_mode: AgentMode,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct FreeLlmApiSettings {
    base_url: String,
    connected: bool,
    ready_models: Vec<String>,
}

impl Default for FreeLlmApiSettings {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:31415".to_owned(),
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

struct ClaudeResult {
    project_id: String,
    chat_id: String,
    result: Result<String, String>,
}

struct ProviderCheckResult {
    base_url: String,
    result: Result<Vec<String>, String>,
}

struct CompanionInstallResult {
    result: Result<(), String>,
}

struct MnemosyneApp {
    state: AppState,
    draft: String,
    pending_attachments: Vec<PathBuf>,
    new_project_name: String,
    creating_project: bool,
    show_settings: bool,
    freellmapi_key_draft: String,
    checking_provider: bool,
    installing_companion: bool,
    busy: bool,
    status: Option<String>,
    result_tx: Sender<ClaudeResult>,
    result_rx: Receiver<ClaudeResult>,
    provider_check_tx: Sender<ProviderCheckResult>,
    provider_check_rx: Receiver<ProviderCheckResult>,
    companion_install_tx: Sender<CompanionInstallResult>,
    companion_install_rx: Receiver<CompanionInstallResult>,
}

impl MnemosyneApp {
    fn new() -> Self {
        let (result_tx, result_rx) = mpsc::channel();
        let (provider_check_tx, provider_check_rx) = mpsc::channel();
        let (companion_install_tx, companion_install_rx) = mpsc::channel();
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
            show_settings: false,
            freellmapi_key_draft: String::new(),
            checking_provider: false,
            installing_companion: false,
            busy: false,
            status: load_error,
            result_tx,
            result_rx,
            provider_check_tx,
            provider_check_rx,
            companion_install_tx,
            companion_install_rx,
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

    fn add_project(&mut self, path: PathBuf) {
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
        self.state.projects.insert(0, project);
        self.save();
    }

    fn create_project_folder(&mut self) {
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
        let api_key = match load_freellmapi_key() {
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
        let conversation = chat.messages.clone();
        let api_base_url = self.state.freellmapi.base_url.clone();
        self.draft.clear();
        self.busy = true;
        self.save();

        let agent_mode = self.state.agent_mode;
        let tx = self.result_tx.clone();
        thread::spawn(move || {
            let result = run_claude(
                &project_path,
                &conversation,
                &api_base_url,
                &api_key,
                agent_mode,
            );
            let _ = tx.send(ClaudeResult {
                project_id,
                chat_id,
                result,
            });
        });
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

    fn receive_result(&mut self, ctx: &egui::Context) {
        while let Ok(response) = self.result_rx.try_recv() {
            self.busy = false;
            let response_content = match response.result {
                Ok(text) => text,
                Err(error) => format!("Помилка Claude Code: {error}"),
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
                    chat.messages.push(Message {
                        role: MessageRole::Assistant,
                        content: response_content,
                        attachments: Vec::new(),
                        created_at: Local::now(),
                    });
                }
            }
            self.save();
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
        while let Ok(response) = self.companion_install_rx.try_recv() {
            self.installing_companion = false;
            self.status = Some(match response.result {
                Ok(()) => "Офіційний FreeLLMAPI запущено. Додайте у ньому ключі безкоштовних провайдерів, потім збережіть unified key і перевірте підключення тут.".to_owned(),
                Err(error) => error,
            });
            ctx.request_repaint();
        }
        if self.busy {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.checking_provider {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
        if self.installing_companion {
            ctx.request_repaint_after(Duration::from_millis(150));
        }
    }
}

impl eframe::App for MnemosyneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.receive_result(ctx);

        egui::SidePanel::left("workspace")
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("Mnemosyne");
                ui.label("Твій простір для вайбкодингу");
                if ui.button("⚙ Налаштування AI").clicked() {
                    self.show_settings = true;
                }
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    if ui.button("+ Папка").clicked() {
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

                ui.add_space(12.0);
                ui.label(egui::RichText::new("ПРОЄКТИ").weak().size(11.0));
                let projects: Vec<(String, String)> = self
                    .state
                    .projects
                    .iter()
                    .map(|project| (project.id.clone(), project.name.clone()))
                    .collect();
                for (id, name) in projects {
                    let selected = self.state.selected_project.as_deref() == Some(id.as_str());
                    if ui
                        .selectable_label(selected, format!("📁 {name}"))
                        .clicked()
                    {
                        self.state.selected_project = Some(id);
                        self.state.selected_chat = None;
                        self.save();
                    }
                }

                if self.selected_project().is_some() {
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("ЧАТИ").weak().size(11.0));
                        if ui.small_button("+").clicked() {
                            self.create_chat();
                        }
                    });
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

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(status) = self.status.clone() {
                ui.colored_label(egui::Color32::LIGHT_RED, status);
            }

            match self.selected_chat() {
                Some(chat) => {
                    let title = chat.title.clone();
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
                    ui.heading(title);
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
                                        ui.strong(if is_user { "Ти" } else { "Claude" });
                                        ui.weak(time);
                                    });
                                    if !content.is_empty() {
                                        ui.label(content);
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
                            if self.busy {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label("Claude думає...");
                                });
                            }
                        });

                    ui.separator();
                    let mut mode_changed = false;
                    ui.horizontal(|ui| {
                        ui.label("Режим агента:");
                        mode_changed |= ui
                            .selectable_value(
                                &mut self.state.agent_mode,
                                AgentMode::Plan,
                                "План · читання",
                            )
                            .changed();
                        mode_changed |= ui
                            .selectable_value(
                                &mut self.state.agent_mode,
                                AgentMode::Turbo,
                                "Turbo · редагування",
                            )
                            .on_hover_text(
                                "Автоматично дозволяє зміни файлів; виконання shell-команд вимкнене.",
                            )
                            .changed();
                    });
                    if mode_changed {
                        self.save();
                    }
                    ui.weak(match self.state.agent_mode {
                        AgentMode::Plan => "Claude може читати проєкт і складати план, але не змінювати файли.",
                        AgentMode::Turbo => "Claude може автоматично редагувати файли. Shell-команди й інші інструменти вимкнені.",
                    });
                    ui.horizontal_wrapped(|ui| {
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
                    let mut remove_attachment = None;
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
                    if let Some(index) = remove_attachment {
                        self.pending_attachments.remove(index);
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .button("📎")
                            .on_hover_text("Додати файли або фото")
                            .clicked()
                        {
                            self.add_attachments();
                        }
                        let input = ui.add(
                            egui::TextEdit::multiline(&mut self.draft)
                                .desired_rows(3)
                                .hint_text("Напиши Claude..."),
                        );
                        let enter_pressed = input.has_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter))
                            && !ui.input(|input| input.modifiers.shift);
                        if enter_pressed {
                            self.send_message();
                        }
                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Надіслати"))
                            .clicked()
                        {
                            self.send_message();
                        }
                    });
                    ui.weak("Enter — надіслати · Shift+Enter — новий рядок");
                }
                None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("Почни з проєкту");
                        ui.label("Додай папку з кодом або створи новий проєкт.");
                    });
                }
            }
        });

        if self.show_settings {
            let mut show_settings = self.show_settings;
            egui::Window::new("Налаштування AI")
                .open(&mut show_settings)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| {
                    ui.heading("FreeLLMAPI · безкоштовні моделі");
                    ui.label("Завантажте й запустіть офіційний FreeLLMAPI з GitHub Releases. Після першого запуску додайте ключі безкоштовних провайдерів у його dashboard, а unified API key збережіть тут.");
                    if ui
                        .add_enabled(
                            !self.installing_companion,
                            egui::Button::new(if self.installing_companion {
                                "Завантажую та запускаю FreeLLMAPI…"
                            } else {
                                "Завантажити / запустити FreeLLMAPI"
                            }),
                        )
                        .clicked()
                    {
                        self.install_freellmapi_companion();
                    }
                    ui.add_space(8.0);
                    ui.label("Адреса сервера (Desktop: 31415 · server: 3001)");
                    ui.text_edit_singleline(&mut self.state.freellmapi.base_url);
                    ui.label("Unified API key");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.freellmapi_key_draft)
                            .password(true)
                            .hint_text("freellmapi-…"),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Зберегти ключ").clicked() {
                            self.save_provider_key();
                        }
                        if ui
                            .add_enabled(
                                !self.checking_provider,
                                egui::Button::new(if self.checking_provider {
                                    "Перевіряю…"
                                } else {
                                    "Перевірити підключення"
                                }),
                            )
                            .clicked()
                        {
                            if !self.freellmapi_key_draft.trim().is_empty() {
                                self.save_provider_key();
                            }
                            self.check_freellmapi();
                        }
                    });
                    if self.state.freellmapi.connected {
                        ui.colored_label(
                            egui::Color32::LIGHT_GREEN,
                            format!(
                                "Підключено · {} доступних моделей",
                                self.state.freellmapi.ready_models.len()
                            ),
                        );
                    } else {
                        ui.weak("Не підключено");
                    }
                    ui.weak("У FreeLLMAPI налаштуйте Keys → Anthropic на auto і Fallback Chain лише на безкоштовні провайдери. Вимкніть платні custom endpoints. Це зменшує ризик списань, але остаточні тарифи й квоти контролює сам провайдер; додаткові акаунти не обходять його правила.");

                    if !self.state.freellmapi.ready_models.is_empty() {
                        ui.add_space(8.0);
                        ui.collapsing("Моделі у FreeLLMAPI", |ui| {
                            for model in &self.state.freellmapi.ready_models {
                                ui.label(format!("• {model}"));
                            }
                        });
                    }
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
    serde_json::from_str(&content).map_err(|error| format!("Пошкоджені дані застосунку: {error}"))
}

fn save_state(state: &AppState) -> Result<(), String> {
    let path = state_path()?;
    let content = serde_json::to_string_pretty(state)
        .map_err(|error| format!("Не вдалося підготувати дані до збереження: {error}"))?;
    fs::write(&path, content).map_err(|error| format!("Не вдалося зберегти дані: {error}"))
}

fn run_claude(
    project_path: &Path,
    messages: &[Message],
    api_base_url: &str,
    api_key: &str,
    agent_mode: AgentMode,
) -> Result<String, String> {
    let has_attachments = messages
        .iter()
        .any(|message| !message.attachments.is_empty());
    let staging = if !has_attachments {
        None
    } else {
        Some(AttachmentStaging::new()?)
    };
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
    command.args(claude_args(agent_mode));
    if let Some(staging) = &staging {
        command.args(["--add-dir", staging.path.to_string_lossy().as_ref()]);
    }
    let mut child = command
        .current_dir(project_path)
        .env("ANTHROPIC_BASE_URL", api_base_url)
        .env("ANTHROPIC_AUTH_TOKEN", api_key)
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!(
                "Не вдалося запустити Claude Code CLI. Перевірте встановлення Claude Code: {error}"
            )
        })?;

    child
        .stdin
        .take()
        .ok_or_else(|| "Не вдалося передати запит Claude Code.".to_owned())?
        .write_all(prompt.as_bytes())
        .map_err(|error| format!("Не вдалося передати запит Claude Code: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("Не вдалося отримати відповідь Claude Code: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if details.is_empty() {
            format!("Claude Code завершився з кодом {}.", output.status)
        } else {
            details
        })
    }
}

fn claude_args(agent_mode: AgentMode) -> Vec<&'static str> {
    let mut args = vec![
        "-p",
        "--output-format",
        "text",
        "--model",
        "claude-sonnet-4-6",
        "--permission-mode",
    ];
    match agent_mode {
        AgentMode::Plan => {
            args.extend(["plan", "--tools", "Read,Glob,Grep"]);
        }
        AgentMode::Turbo => {
            args.extend(["acceptEdits", "--tools", "Read,Glob,Grep,Edit,Write"]);
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

    Command::new(&asset_path).spawn().map_err(|error| {
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

#[cfg(windows)]
fn claude_command() -> Command {
    let mut command = Command::new("cmd.exe");
    command.args(["/D", "/C", "claude"]);
    command
}

#[cfg(not(windows))]
fn claude_command() -> Command {
    Command::new("claude")
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([720.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Mnemosyne",
        options,
        Box::new(|_creation_context| Ok(Box::new(MnemosyneApp::new()))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn turbo_mode_allows_file_edits_but_not_shell_commands() {
        let args = claude_args(AgentMode::Turbo);
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "acceptEdits"]));
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--tools", "Read,Glob,Grep,Edit,Write"]));
        assert!(!args.iter().any(|arg| arg.contains("Bash")));
    }

    #[test]
    fn plan_mode_remains_read_only() {
        let args = claude_args(AgentMode::Plan);
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
}
