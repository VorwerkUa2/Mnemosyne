# 🌌 Mnemosyne

> **Cross-Platform Vibe-Coding Workspace & Coding Agent Host**  
> **Кросплатформний десктопний простір для вайбкодингу та автономних агентів**

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey.svg)](#)
[![Decod MCP](https://img.shields.io/badge/Decod%20MCP-58%20Tools-purple.svg)](https://github.com/VorwerkUa2/DFM)
[![Release](https://img.shields.io/badge/Release-v0.1.0-brightgreen.svg)](https://github.com/VorwerkUa2/Mnemosyne/releases/tag/v0.1.0)
[![AI Engine](https://img.shields.io/badge/AI%20Engine-LM%20Studio-blue.svg)](SETUP_GUIDE.md)

[**🇺🇦 Українська версія**](#-українська-версія) | [**🇬🇧 English Version**](#-english-version)

---

## 🇺🇦 Українська версія

**Mnemosyne** — це кросплатформна оболонка агента для локального **LM Studio** (через Anthropic-compatible API), **Claude Code CLI** та двох MCP-модулів: **Decod** для розробки й аналізу та **Designatio** для дизайну й реалізації макетів. **Ollama** залишається необов'язковою локальною альтернативою.

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
  - **Дії з повідомленнями**: контекстне меню ПКМ для копіювання, редагування запиту та перегенерації у новій гілці. Центроване підтвердження окремо пропонує відкотити відстежені зміни `Write`/`Edit`; файл не перезаписується, якщо після відповіді його змінено.

- **🤖 Локальний AI через LM Studio**:
  - Потокова генерація відповіді в реальному часі з відображенням поточного інструмента та можливістю скасування в 1 клік.
  - Mnemosyne автоматично перевіряє `http://127.0.0.1:1234` під час запуску, знаходить завантажені моделі та дозволяє вибрати модель для чату.
  - Усі запити йдуть до локального API; підписка на зовнішній API не потрібна. LM Studio має бути запущений із завантаженою моделлю та Anthropic-compatible API.
  - Ollama можна підключити як локальну альтернативу, якщо потрібні маршрути `/v1/models` і `/v1/messages`.

- **🛡 Безпека та Режими агента**:
  - **📋 План (Plan Mode)**: Тільки читання (`Read`, `Glob`, `Grep`). Зміни файлів заблоковані.
  - **⚡ Turbo (File Edits with Approval)**: Агент може змінювати файли проєкту, але кожна правка потребує схвалення користувача з наочним переглядом diff.

- **⚡ Суперсили кодингу (Vibe-Coding)**:
  - **Режим Caveman**: економія 30–40% токенів завдяки відсіканню балаканини, ввічливих прелюдій і зайвого повторення коду.
  - **Принципи Карпатого (Karpathy Rules) + Ponytail**: суворе дотримання YAGNI, мінімум непотрібних абстракцій, захисне програмування та збереження стилю репозиторію.
  - **Кнопка `🔍 Перевірити діагностику`**: миттєва перевірка компілятором (`cargo check` або синтаксис Python) прямо в чаті з **0 витратою токенів**!
  - **Автопровізіонування `.claude/CLAUDE.md`**: автоматичне визначення стеку проєкту (Rust, Python, C/C++, C#, Go, Lua, Web) і генерація індивідуальних правил для моделі.
  - **Локальна пам’ять чату та граф знань**: повна історія, підсумки й окремий для кожного проєкту граф сутностей і зв’язків зберігаються на пристрої у `state.json`. Граф періодично витягується локальною моделлю, додається до контексту агента й переглядається в інтерфейсі. Для офлайн-стиснення всіх чатів запустіть `Mnemosyne.exe --compact-chat-memory-locally`: буде створено резервну копію `state.json`, а оригінальні повідомлення залишаться незмінними.

- **🛠 Повна інтеграція Decod RE-рушія (58 MCP Інструментів)**:
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

**Mnemosyne** is a cross-platform agent host for local **LM Studio** (via its Anthropic-compatible API), **Claude Code CLI**, and two MCP modules: **Decod** for development and analysis, and **Designatio** for design and mockup implementation. **Ollama** remains an optional local alternative.

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
  - **Message actions**: right-click to copy, edit a prompt, or regenerate in a new branch. A centered confirmation dialog optionally reverts tracked `Write`/`Edit` changes and refuses to overwrite files changed since the response.

- **🤖 Local AI with LM Studio**:
  - Real-time streaming output displaying active tool calls with instant 1-click request cancellation.
  - Mnemosyne checks `http://127.0.0.1:1234` on startup, discovers loaded models, and lets you select one in chat.
  - All requests go to the local API; no external model API subscription is required. LM Studio must be running with a model loaded and its Anthropic-compatible API enabled.
  - Ollama can be used as an optional local alternative with `/v1/models` and `/v1/messages` routes.

- **🛡 Safety & Agent Modes**:
  - **📋 Plan Mode**: Read-only operations (`Read`, `Glob`, `Grep`). All file modifications are strictly blocked.
  - **⚡ Turbo Mode (File Edits with Approval)**: The agent can modify project files, but each edit requires manual user confirmation with a side-by-side diff preview.

- **⚡ Vibe-Coding Superpowers**:
  - **Caveman Mode**: Saves 30–40% tokens by cutting out conversational filler, greetings, and boilerplate code diffs.
  - **Karpathy Rules + Ponytail**: Strictly enforces YAGNI (You Aren't Gonna Need It), minimal abstractions, defensive checks, and repo style matching.
  - **`🔍 Check Diagnostics` Button**: Background compiler checks (`cargo check` or Python syntax) directly from the chat UI with **0 token cost**.
  - **Auto-Provisioned `.claude/CLAUDE.md`**: Detects project tech stack (Rust, Python, C/C++, C#, Go, Lua, Web) and injects tailor-made agent rules.
  - **Local Chat Memory & Knowledge Graph**: Full history, summaries, and a per-project graph of entities and relationships are stored on this device in `state.json`. The graph is periodically extracted by the local model, included in agent context, and viewable in the UI. For offline compaction of all chats, run `Mnemosyne.exe --compact-chat-memory-locally`: it backs up `state.json` and leaves original messages unchanged.

- **🛠 Decod Headless Reverse Engineering MCP (58 Tools)**:
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

# Run unit tests / Запуск тестів
cargo test
```

---

## 📜 Ліцензія / License

Distributed under the [MIT License](LICENSE).
Розповсюджується під ліцензією [MIT](LICENSE).
