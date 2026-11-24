# 🌌 Mnemosyne

> **Cross-Platform Vibe-Coding Workspace & Coding Agent Host**  
> **Кросплатформний десктопний простір для вайбкодингу та автономних агентів**

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey.svg)](#)
[![Decod MCP](https://img.shields.io/badge/Decod%20MCP-51%20Tools-purple.svg)](https://github.com/VorwerkUa2/DFM)
[![Release](https://img.shields.io/badge/Release-v0.1.0-brightgreen.svg)](https://github.com/VorwerkUa2/Mnemosyne/releases/tag/v0.1.0)
[![AI Cost](https://img.shields.io/badge/AI%20Cost-%240%20(FreeLLMAPI)-green.svg)](SETUP_GUIDE.md)

[**🇺🇦 Українська версія**](#-українська-версія) | [**🇬🇧 English Version**](#-english-version)

---

## 🇺🇦 Українська версія

**Mnemosyne** — це швидкий, нативний десктопний простір (написаний на Rust / `egui`), що об'єднує **Claude Code CLI**, локальний безкоштовний маршрутизатор моделей **FreeLLMAPI** ($0 витрат) та реверс-інжиніринг рушій **Decod** (51 інструмент MCP).

### ⚡ Швидкий старт

#### 🪟 Windows 10 / 11
1. Завантажте готовий автономний інсталятор: 👉 **[Mnemosyne-Setup.exe (v0.1.0)](https://github.com/VorwerkUa2/Mnemosyne/releases/tag/v0.1.0)** (10.8 МБ).
2. Запустіть файл — програма автоматично розпакується в `%LOCALAPPDATA%\Programs\Mnemosyne`, створить ярлики на Робочому столі та в Меню «Пуск» і налаштує деінсталятор.
3. Або запустіть скрипт `install.bat` у корені репозиторію.

#### 🐧 Linux (Void Linux, Fedora, Arch, Ubuntu/Debian)
Встановлення **однією командою** в терміналі:
```bash
curl -fsSL https://raw.githubusercontent.com/VorwerkUa2/Mnemosyne/main/install.sh | sh
```
Скрипт автоматично виявить ваш пакетний менеджер (`xbps`, `dnf`, `pacman`, `apt`), встановить необхідні графічні залежності Wayland/X11, помістить бінарник у `~/.local/bin/mnemosyne` та створить `.desktop`-ярлик у системному меню програм.

---

### ✨ Головні можливості

- **🎨 Інтерфейс та Локалізація**:
  - **Темна сланцева тема за замовчуванням** (`#121218`) з м'яким індиго-акцентом та миттєвим перемиканням на світлу.
  - **Нова верхня панель під колір теми**: брендинг, бейдж активного проєкту, індикатор режиму (`⚡ Turbo` / `📋 Plan`), швидкі перемикачі мови (`🇺🇦`/`🇬🇧`) та теми (`🌙`/`☀`).
  - **Компактні налаштування зі слайдером**: зручний вертикальний скрол, фіксована ширина (450px) без перекриття робочого простору.
  - **Повна двомовність**: Українська 🇺🇦 та Англійська 🇬🇧 на льоту.
  - **Вбудований переглядач**: перегляд Markdown із рендерингом таблиць, списків завдань, підсвічування тексту та перегляд зображень (PNG, JPEG, GIF, WebP, BMP).

- **🤖 Інтеграція Claude Code CLI та FreeLLMAPI ($0)**:
  - Потокова генерація відповіді в реальному часі з відображенням поточного інструмента та можливістю скасування в 1 клік.
  - Список моделей у чаті синхронізується з доступним каталогом **FreeLLMAPI**; можна вибрати конкретний ID або автоматичний fallback.
  - Усі вибрані моделі працюють через Claude Code CLI та зберігають інструменти агента й Decod MCP; FreeLLMAPI адаптує Anthropic Messages/tool calls до провайдера.
  - Безпечне збереження Unified API Key у захищеному сховищі облікових даних ОС.
  - Вбудовані кнопки перевірки з'єднання та наскрізного тесту запиту до моделі.

- **🛡 Безпека та Режими агента**:
  - **📋 План (Plan Mode)**: Тільки читання (`Read`, `Glob`, `Grep`). Зміни файлів заблоковані.
  - **⚡ Turbo (File Edits with Approval)**: Агент може змінювати файли проєкту, але кожна правка потребує схвалення користувача з наочним переглядом diff.

- **⚡ Суперсили кодингу (Vibe-Coding)**:
  - **Режим Caveman**: економія 30–40% токенів завдяки відсіканню балаканини, ввічливих прелюдій і зайвого повторення коду.
  - **Принципи Карпатого (Karpathy Rules) + Ponytail**: суворе дотримання YAGNI, мінімум непотрібних абстракцій, захисне програмування та збереження стилю репозиторію.
  - **Кнопка `🔍 Перевірити діагностику`**: миттєва перевірка компілятором (`cargo check` або синтаксис Python) прямо в чаті з **0 витратою токенів**!
  - **Автопровізіонування `.claude/CLAUDE.md`**: автоматичне визначення стеку проєкту (Rust, Python, C/C++, C#, Go, Lua, Web) і генерація індивідуальних правил для моделі.
  - **Локальна пам’ять чату**: повна історія зберігається на пристрої; звичайне семантичне оновлення підсумку передає архівні повідомлення вибраному API. Для офлайн-стиснення всіх чатів запустіть `Mnemosyne.exe --compact-chat-memory-locally`: буде створено резервну копію `state.json`, а оригінальні повідомлення залишаться незмінними. Локальний конспект є витягом, а не семантичним підсумком.

- **🛠 Повна інтеграція Decod RE-рушія (51 MCP Інструмент)**:
  - Динаміка та хуки пам'яті: Frida dynamic runner, Scylla process dumper, захист від анти-відлагодження (`libstealth.so`).
  - Статика та дизасемблювання: Capstone x86/x64, LIEF аналіз заголовків (PE/ELF/Mach-O), Headless C decompiler.
  - Модинг ігор: розпаковка/пакування контейнерів UE5 PAK/UTOC/UCAS (`repak`), читання та редагування сейвів (`uesave`), конвертація звуку та відео через FFmpeg.
  - Code Intelligence: аналіз AST структури (`inspect_code_structure`) та точкове вилучення функцій (`extract_function_definition`), що знижує жор токенів на **70%–85%**.

- **🎨 Designatio MCP**: універсальні інструменти дизайну та реалізації макетів. Увімкніть інтеграцію і вкажіть папку Designatio в налаштуваннях; особистий Figma token зберігається у сховищі облікових даних ОС і передається лише Designatio MCP. Імпорт локального JSON-макета працює без токена.

- **🔄 Оновлення та обслуговування в 1 клік**:
  - Оновлення Claude Code CLI (`npm update -g @anthropic-ai/claude-code`).
  - Оновлення RE-рушія Decod (`git pull origin main`).
  - Оновлення бібліотек Python (`pip install --upgrade frida lief capstone psutil pyright`).

---

## 🇬🇧 English Version

**Mnemosyne** is a fast, native desktop workspace (built with Rust / `egui`) that seamlessly connects **Claude Code CLI**, a local free AI routing companion **FreeLLMAPI** ($0 cost), and the headless reverse engineering engine **Decod** (51 MCP tools).

### ⚡ Quick Start

#### 🪟 Windows 10 / 11
1. Download the standalone 1-click installer: 👉 **[Mnemosyne-Setup.exe (v0.1.0)](https://github.com/VorwerkUa2/Mnemosyne/releases/tag/v0.1.0)** (10.8 MB).
2. Run the executable — it extracts into `%LOCALAPPDATA%\Programs\Mnemosyne`, creates Desktop & Start Menu shortcuts, and registers a standard uninstaller in Windows Settings.
3. Or double-click `install.bat` in the repository root.

#### 🐧 Linux (Void Linux, Fedora, Arch, Ubuntu/Debian)
Install with **a single terminal command**:
```bash
curl -fsSL https://raw.githubusercontent.com/VorwerkUa2/Mnemosyne/main/install.sh | sh
```
The script auto-detects your package manager (`xbps`, `dnf`, `pacman`, `apt`), installs Wayland/X11 runtime dependencies, installs the binary to `~/.local/bin/mnemosyne`, and sets up the `.desktop` launcher.

---

### ✨ Key Features

- **🎨 UI & Localization**:
  - **Default Dark Slate Theme** (`#121218`) with soft indigo accents and instant toggle to Light Theme.
  - **Matching Top Header Bar**: App branding, active project badge, agent status indicator (`⚡ Turbo` / `📋 Plan`), quick 1-click language toggle (`🇺🇦`/`🇬🇧`), and theme toggle (`🌙`/`☀`).
  - **Compact Settings Window with Slider**: Smooth vertical scroll area (`ScrollArea::vertical()`), bounded 450px width preventing workspace overflow.
  - **Full Bilingual Support**: Instant switching between Ukrainian 🇺🇦 and English 🇬🇧.
  - **Built-in Document & Image Viewer**: Markdown parser with tables, task lists, and syntax blocks; image viewer for PNG, JPEG, GIF, WebP, BMP.

- **🤖 Claude Code CLI & FreeLLMAPI ($0 Cost)**:
  - Real-time streaming output displaying active tool calls with instant 1-click request cancellation.
  - The chat model list syncs with the available **FreeLLMAPI** catalog; choose a concrete model ID or automatic fallback.
  - All selected models run through Claude Code CLI and keep agent/Decod MCP tools; FreeLLMAPI translates Anthropic Messages/tool calls for the selected provider.
  - Secure credential storage: Unified API keys are encrypted in the native OS keychain.
  - In-app connection checking and end-to-end ping test buttons.

- **🛡 Safety & Agent Modes**:
  - **📋 Plan Mode**: Read-only operations (`Read`, `Glob`, `Grep`). All file modifications are strictly blocked.
  - **⚡ Turbo Mode (File Edits with Approval)**: The agent can modify project files, but each edit requires manual user confirmation with a side-by-side diff preview.

- **⚡ Vibe-Coding Superpowers**:
  - **Caveman Mode**: Saves 30–40% tokens by cutting out conversational filler, greetings, and boilerplate code diffs.
  - **Karpathy Rules + Ponytail**: Strictly enforces YAGNI (You Aren't Gonna Need It), minimal abstractions, defensive checks, and repo style matching.
  - **`🔍 Check Diagnostics` Button**: Background compiler checks (`cargo check` or Python syntax) directly from the chat UI with **0 token cost**.
  - **Auto-Provisioned `.claude/CLAUDE.md`**: Detects project tech stack (Rust, Python, C/C++, C#, Go, Lua, Web) and injects tailor-made agent rules.
  - **Local Chat Memory**: Full chat history is stored on this device; normal semantic summary updates send archived messages to the selected API. For offline compaction of all chats, run `Mnemosyne.exe --compact-chat-memory-locally`: it backs up `state.json` and leaves original messages unchanged. The local note is extractive, not a semantic summary.

- **🛠 Decod Headless Reverse Engineering MCP (51 Tools)**:
  - Memory dynamics & hooking: Frida dynamic runner, Scylla process memory dumper, anti-anti-debugging (`libstealth.so`).
  - Static disassembly & analysis: Capstone x86/x64, LIEF binary parsing (PE/ELF/Mach-O), Headless C decompiler.
  - Game modding & assets: Unreal Engine 5 PAK/UTOC/UCAS repacking (`repak`), savefile decoding (`uesave`), multimedia conversion via FFmpeg.
  - Code Intelligence: AST structure outline (`inspect_code_structure`) and surgical single-function extraction (`extract_function_definition`), saving **70%–85% token burn**.

- **🎨 Designatio MCP**: universal design and mockup implementation tools. Enable it and set the Designatio directory in Settings; your personal Figma token is stored in the OS credential store and passed only to Designatio MCP. Local JSON mockup imports work without a token.

- **🔄 1-Click Updates & Maintenance**:
  - Updates Claude Code CLI (`npm update -g @anthropic-ai/claude-code`).
  - Updates Decod RE engine (`git pull origin main`).
  - Updates Python dependencies (`pip install --upgrade frida lief capstone psutil pyright`).

---

## 🛠 Збірка з вихідного коду (Building from Source)

```bash
# Clone the repository / Клонування репозиторію
git clone https://github.com/VorwerkUa2/Mnemosyne.git
cd Mnemosyne

# Run development build / Запуск у режимі розробки
cargo run

# Build optimized release / Збірка релізу
cargo build --release

# Run unit tests / Запуск тестів (all 37 tests passing)
cargo test
```

---

## 📜 Ліцензія / License

Distributed under the [MIT License](LICENSE).
Розповсюджується під ліцензією [MIT](LICENSE).
