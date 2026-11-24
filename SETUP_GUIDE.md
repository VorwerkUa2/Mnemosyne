# 🚀 Mnemosyne & Decod: Complete Setup & User Guide
# 📘 Повний посібник із встановлення та налаштування Mnemosyne і Decod

[**🇺🇦 Українська версія**](#-українська-версія) | [**🇬🇧 English Version**](#-english-version)

---

## 🇺🇦 Українська версія

### 1. Швидке встановлення (Quick Install)

#### 🪟 Windows 10 / 11
Ви можете встановити Mnemosyne двома зручними способами:
1. **Автономний інсталятор (Рекомендовано)**:
   - Завантажте або запустіть файл `Mnemosyne-Setup.exe` (10.8 МБ).
   - Інсталятор автоматично:
     - Розпакує бінарник у `%LOCALAPPDATA%\Programs\Mnemosyne`.
     - Створить ярлики в Меню «Пуск» та на Робочому столі.
     - Зареєструє деінсталятор у «Параметри → Програми».
     - Перевірить наявність Node.js та Claude Code CLI.
2. **Через скрипт `install.bat`**:
   - Двічі клікніть `install.bat` у корені репозиторію.

#### 🐧 Linux (Void Linux, Fedora, Arch, Ubuntu/Debian)
Встановлення виконується **однією командою в терміналі**:
```bash
curl -fsSL https://raw.githubusercontent.com/VorwerkUa2/Mnemosyne/main/install.sh | sh
```
Скрипт автоматично:
- Визначає дистрибутив і пакетний менеджер:
  - **Void Linux**: `xbps-install -Sy libxkbcommon-devel fontconfig-devel wayland-devel ...`
  - **Fedora**: `dnf install -y libxkbcommon-devel fontconfig-devel wayland-devel ...`
  - **Arch Linux**: `pacman -S --needed libxkbcommon fontconfig wayland ...`
  - **Ubuntu / Debian**: `apt-get install -y libxkbcommon-dev libfontconfig1-dev ...`
- Встановлює бінарник у `~/.local/bin/mnemosyne`.
- Створює `.desktop`-ярлик для системного меню програм.
- Для видалення достатньо виконати: `mnemosyne-uninstall`.

---

### 2. Підключення локальної моделі (LM Studio)

Mnemosyne використовує **LM Studio як основний рушій** через локальний Anthropic-compatible API. Запити чату залишаються на вашому пристрої; зовнішній API-ключ не потрібен. Обчислювальні ресурси та електроенергія залежать від обраної моделі й обладнання.

1. Встановіть [LM Studio](https://lmstudio.ai/) для Windows або Linux і завантажте модель, що підтримує роботу з інструментами.
2. У LM Studio відкрийте локальний сервер, увімкніть Anthropic-compatible API та запустіть його на типовій адресі `http://127.0.0.1:1234`.
3. Запустіть Mnemosyne. Програма автоматично перевірить сервер і завантажені моделі; статус та вибір моделі доступні в **⚙ Налаштуваннях** і у верхній панелі чату.
4. Якщо сервер має іншу адресу, змініть її в налаштуваннях і натисніть **«Перевірити API та знайти моделі»**. Щоб перейти на іншу модель, спершу завантажте її в LM Studio, а потім виберіть у списку Mnemosyne.
5. **Ollama** можна використовувати як необов'язкову локальну альтернативу на `http://127.0.0.1:11434`, якщо її версія підтримує маршрути `/v1/models` і Anthropic `/v1/messages`.

Для кодування також потрібен **Claude Code CLI** (див. наступний розділ). Якість та швидкість залежать від розміру моделі, підтримки виклику інструментів і доступних RAM/VRAM.

**Локальні чати й пам’ять агента** зберігаються у `state.json`: `%LOCALAPPDATA%\Mnemosyne\state.json` на Windows або `~/.local/share/Mnemosyne/state.json` на Linux. Повна історія залишається на пристрої; у запит потрапляє обмежений контекст, за замовчуванням — останні 10 ходів користувача й не більше 48 000 символів історії. Старі повідомлення підсумовуються локальною моделлю в окрему пам’ять чату (до 8 000 символів). Для кожного проєкту також зберігається граф сутностей і зв’язків: він періодично витягується локальною моделлю, додається до контексту агента й переглядається в чаті. Офлайн-команда `--compact-chat-memory-locally` створює витяг без API, але не видаляє оригінальну історію.

---

### 3. Налаштування Claude Code CLI

1. Якщо Claude Code ще не встановлено, натисніть кнопку **«Встановити Claude Code CLI»** у налаштуваннях Mnemosyne (потрібен Node.js).
2. Або виконайте в терміналі:
   ```bash
   npm install -g @anthropic-ai/claude-code
   ```
3. Mnemosyne автоматично виявляє Claude Code CLI у стандартних системних шляхах Windows та Linux.

---

### 4. Підключення RE-рушія Decod (58 Інструментів)

Decod надає Claude Code повний стек реверс-інжинірингу, компіляторної діагностики та модингу.
1. У терміналі папки `Decod` виконайте одноразове встановлення залежностей:
   ```bash
   # Windows
   python setup_decod.py --install-all

   # Linux
   python3 setup_decod.py --install-all
   ```
   Це автоматично встановить `frida`, `lief`, `capstone`, `psutil`, `pyright`, `rust-analyzer`, завантажить портативні `repak.exe` та `uesave.exe`, та збілдить `libstealth.so` на Linux.
2. У **Налаштуваннях Mnemosyne** переконайтеся, що прапорець **«Увімкнути Decod MCP»** увімкнено (за замовчуванням увімкнено).
3. Натисніть **«Перевірити»** для підтвердження готовності MCP-сервера.

---

### 5. Підключення Designatio MCP

Designatio додає до агента аналіз макетів, генерацію дизайн-систем і текстур, перевірки адаптивності та доступності, а також реалізацію web/native UI.

1. Встановіть залежності в папці Designatio:
   - Windows: `.\install.ps1`
   - Linux: `./install.sh`
2. У **Налаштуваннях Mnemosyne** перевірте шлях до папки Designatio та стан Designatio MCP.
3. Figma token необов'язковий: він потрібен лише функціям, що читають Figma API. Локальний JSON-макет можна імпортувати без токена.

### 6. Суперсили кодингу (Vibe-Coding)

У Mnemosyne реалізовано комплекс суперсил для швидкого кодингу без переплат:

1. **Режим Caveman (Економія токенів на 30–40%)**:
   - Прибирає зайву ввічливість, привітання та водянисті описи. Модель одразу видає хірургічний diff або робочий код.
2. **Принципи Карпатого (Karpathy Rules)**:
   - Мінімум зайвих абстракцій (YAGNI), атомарні правки, точне збереження стилю проєкту, захисне програмування та обов'язкова компіляторна перевірка.
3. **Автоматичне налаштування `.claude/CLAUDE.md`**:
   - При виборі або створенні проєкту Mnemosyne визначає стек (Rust, Python, C/C++, C#, Go, Lua, Web) і генерує індивідуальний конфіг правил.
4. **Кнопка `🔍 Перевірити діагностику` у чаті**:
   - Розташована над полем введення. Запускає `cargo check` або перевірку синтаксису Python у фоні **з 0 витратою токенів** і показує статус прямо в чаті.
5. **Інструменти `inspect_code_structure` та `extract_function_definition`**:
   - Модель зчитує лише сигнатури або окремі функції замість того, щоб завантажувати у контекст файли на тисячі рядків. Економія контексту: **70%–85%**.

**Локальна пам’ять чату та граф знань**: повна історія, підсумок і граф сутностей/зв’язків для кожного проєкту зберігаються у `state.json` на цьому пристрої. Граф періодично створюється локальною моделлю, додається до контексту агента та доступний для перегляду в чаті. Для оновлення підсумку й графа потрібен доступ до налаштованого локального API.

---

### 7. Оновлення компонентів в 1 клік

У розділі **Налаштування $\rightarrow$ Оновлення та обслуговування** натисніть:
**«Оновити всі компоненти зараз»**.
Програма у фоновому режимі оновить:
- Claude Code CLI (`npm update -g @anthropic-ai/claude-code`)
- Decod RE рушій (`git pull origin main`)
- Бібліотеки Python (`python -m pip install --upgrade frida lief capstone psutil pyright` на Windows; `python3 -m pip ...` на Linux)

---
---

## 🇬🇧 English Version

### 1. Quick Install

#### 🪟 Windows 10 / 11
Install Mnemosyne using one of two options:
1. **Standalone Installer (Recommended)**:
   - Download and run `Mnemosyne-Setup.exe` (10.8 MB).
   - The installer will automatically:
     - Extract the binary to `%LOCALAPPDATA%\Programs\Mnemosyne`.
     - Create Start Menu and Desktop shortcuts.
     - Register an uninstaller in Windows Settings $\rightarrow$ Installed Apps.
     - Verify Node.js and Claude Code CLI availability.
2. **Using `install.bat`**:
   - Double-click `install.bat` in the repository root.

#### 🐧 Linux (Void Linux, Fedora, Arch, Ubuntu/Debian)
Install with **a single terminal command**:
```bash
curl -fsSL https://raw.githubusercontent.com/VorwerkUa2/Mnemosyne/main/install.sh | sh
```
The installer automatically:
- Detects your Linux distribution and package manager:
  - **Void Linux**: `xbps-install -Sy libxkbcommon-devel fontconfig-devel wayland-devel ...`
  - **Fedora**: `dnf install -y libxkbcommon-devel fontconfig-devel wayland-devel ...`
  - **Arch Linux**: `pacman -S --needed libxkbcommon fontconfig wayland ...`
  - **Ubuntu / Debian**: `apt-get install -y libxkbcommon-dev libfontconfig1-dev ...`
- Installs the binary to `~/.local/bin/mnemosyne`.
- Creates a `.desktop` application menu entry.
- To uninstall anytime, simply run: `mnemosyne-uninstall`.

---

### 2. Connect a Local Model (LM Studio)

Mnemosyne uses **LM Studio as its primary model engine** through the local Anthropic-compatible API. Chat requests stay on your machine, and no external API key is required. Compute and electricity costs depend on the selected model and hardware.

1. Install [LM Studio](https://lmstudio.ai/) for Windows or Linux and download a model that supports tool use.
2. In LM Studio, open the local server, enable its Anthropic-compatible API, and start it at the default `http://127.0.0.1:1234` address.
3. Start Mnemosyne. It checks the server and loaded models automatically; connection status and model selection are available in **⚙ Settings** and the chat toolbar.
4. If the server uses another address, update it in Settings and click **"Verify API & discover models"**. To switch models, load the model in LM Studio first, then select it in Mnemosyne.
5. **Ollama** is an optional local alternative at `http://127.0.0.1:11434` when its version supports Mnemosyne's `/v1/models` and Anthropic `/v1/messages` routes.

**Claude Code CLI** is also required for coding tasks (see the next section). Quality and speed depend on model size, tool-calling support, and available RAM/VRAM.

**Local chats and agent memory** are stored in `state.json`: `%LOCALAPPDATA%\Mnemosyne\state.json` on Windows or `~/.local/share/Mnemosyne/state.json` on Linux. Full history remains on the device; prompts use bounded context, defaulting to the last 10 user turns and at most 48,000 history characters. Older messages are summarized by the local model into a per-chat memory (up to 8,000 characters). Each project also stores a graph of entities and relationships, periodically extracted by the local model, included in agent context, and viewable in chat. The offline `--compact-chat-memory-locally` command creates an extractive note without an API call, but does not delete the original history.

---

### 3. Claude Code CLI Setup

1. If not installed yet, click **"Install Claude Code CLI"** in Mnemosyne Settings (requires Node.js).
2. Or run manually in your terminal:
   ```bash
   npm install -g @anthropic-ai/claude-code
   ```
3. Mnemosyne automatically discovers Claude Code CLI across standard Windows and Linux locations.

---

### 4. Decod MCP Engine Integration (58 Tools)

Decod provides Claude Code with a comprehensive reverse engineering, diagnostics, and modding toolkit.
1. In the `Decod` directory, run one-time dependency installation:
   ```bash
   # Windows
   python setup_decod.py --install-all

   # Linux
   python3 setup_decod.py --install-all
   ```
   This installs `frida`, `lief`, `capstone`, `psutil`, `pyright`, `rust-analyzer`, downloads `repak` and `uesave`, and builds `libstealth.so` on Linux.
2. In **Mnemosyne Settings**, ensure **"Enable Decod MCP"** is checked (enabled by default).
3. Click **"Verify"** to test JSON-RPC MCP connectivity.

---

### 5. Connect Designatio MCP

Designatio adds mockup analysis, design-system and texture generation, responsive/accessibility checks, and web/native UI implementation.

1. Install dependencies from the Designatio directory:
   - Windows: `.\install.ps1`
   - Linux: `./install.sh`
2. In **Mnemosyne Settings**, verify the Designatio directory and MCP status.
3. A Figma token is optional and only needed for features that access the Figma API. Local JSON mockup import works without one.

### 6. Vibe-Coding Superpowers

1. **Caveman Mode (30–40% Token Savings)**:
   - Eliminates fluff, greetings, and pleasantries. Direct code diffs only.
2. **Karpathy Engineering Rules**:
   - YAGNI (You Aren't Gonna Need It), minimal abstractions, defensive checks, strict repository style matching, and compiler verification.
3. **Auto-Provisioned `.claude/CLAUDE.md`**:
   - Automatically detects project tech stack (Rust, Python, C/C++, C#, Go, Lua, Web) and configures rules tailored to that language.
4. **`🔍 Check Diagnostics` Button in Chat**:
   - Runs `cargo check` or Python syntax verification in the background with **0 token cost**.
5. **Token Thrift via Decod AST Tools**:
   - Uses `inspect_code_structure` and `extract_function_definition` to avoid loading thousand-line files into the model context (**70%–85% token burn reduction**).

**Local chat memory and knowledge graph**: Full history, summaries, and a per-project entity/relationship graph are stored in `state.json` on this device. The graph is periodically created by the local model, included in agent context, and viewable in chat. Updating summaries and the graph requires access to the configured local API.

---

### 7. One-Click Updates & Maintenance

In **Settings $\rightarrow$ Updates & Maintenance**, click **"Update All Components Now"** to update:
- Claude Code CLI (`npm update -g @anthropic-ai/claude-code`)
- Decod RE engine (`git pull origin main`)
- Python packages (`python -m pip install --upgrade frida lief capstone psutil pyright` on Windows; `python3 -m pip ...` on Linux)
