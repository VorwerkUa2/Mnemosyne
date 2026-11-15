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

### 2. Налаштування FreeLLMAPI (100% Безкоштовний ШІ)

Mnemosyne спроєктовано так, щоб працювати без жодних витрат ($0) через маршрутизатор **FreeLLMAPI**:

1. **Отримайте безкоштовні API-ключі**:
   - **Google Gemini API**: Перейдіть на [Google AI Studio](https://aistudio.google.com/) $\rightarrow$ створіть безкоштовний API ключ (ліміт: **1 500 запитів на добу**, 15 RPM).
   - **Groq Cloud (Llama 3.3 70B)**: Перейдіть на [Groq Console](https://console.groq.com/) $\rightarrow$ створіть ключ (ліміт: **14 400 запитів на добу**, 30 RPM).
   - **OpenRouter (Free tier)**: Створіть безкоштовний ключ на [OpenRouter](https://openrouter.ai/).
2. **Встановлення FreeLLMAPI**:
   - Відкрийте Mnemosyne $\rightarrow$ перейдіть у **⚙ Налаштування**.
   - Натисніть кнопку **«Завантажити й встановити FreeLLMAPI»**. Mnemosyne автоматично завантажить офіційний реліз із GitHub (перевіривши SHA-256).
   - Запустіть FreeLLMAPI Dashboard (зазвичай `http://127.0.0.1:31415`).
   - У розділі **Providers** додайте ваші ключі (Gemini, Groq, OpenRouter).
   - У розділі **Keys $\rightarrow$ Anthropic** встановіть маршрутизацію для сімейств `Sonnet`, `Opus`, `Haiku` на безкоштовний пул (наприклад, Gemini Flash / Llama 3.3).
3. **Підключення до Mnemosyne**:
   - Скопіюйте Unified API Key із FreeLLMAPI.
   - Вставте його у поле **«Unified API Key»** у налаштуваннях Mnemosyne та натисніть **«Зберегти ключ»** (ключ шифрується у системному сховищі облікових даних ОС).
   - Натисніть **«Перевірити підключення»** $\rightarrow$ з'явиться зелений статус `✓ Підключено`.
   - Натисніть **«Тест запиту»** для перевірки наскрізного пінгу моделі.

---

### 3. Налаштування Claude Code CLI

1. Якщо Claude Code ще не встановлено, натисніть кнопку **«Встановити Claude Code CLI»** у налаштуваннях Mnemosyne (потрібен Node.js).
2. Або виконайте в терміналі:
   ```bash
   npm install -g @anthropic-ai/claude-code
   ```
3. Mnemosyne автоматично виявляє Claude Code CLI у стандартних системних шляхах Windows та Linux.

---

### 4. Підключення RE-рушія Decod (46 Інструментів)

Decod надає Claude Code повний стек реверс-інжинірингу, компіляторної діагностики та модингу.
1. У терміналі папки `Decod` виконайте одноразове встановлення залежностей:
   ```bash
   python setup_decod.py --install-all
   ```
   Це автоматично встановить `frida`, `lief`, `capstone`, `psutil`, `pyright`, `rust-analyzer`, завантажить портативні `repak.exe` та `uesave.exe`, та збілдить `libstealth.so` на Linux.
2. У **Налаштуваннях Mnemosyne** переконайтеся, що прапорець **«Увімкнути Decod MCP»** увімкнено (за замовчуванням увімкнено).
3. Натисніть **«Перевірити»** для підтвердження готовності MCP-сервера.

---

### 5. Суперсили кодингу (Vibe-Coding)

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

---

### 6. Оновлення компонентів в 1 клік

У розділі **Налаштування $\rightarrow$ Оновлення та обслуговування** натисніть:
**«Оновити всі компоненти зараз»**.
Програма у фоновому режимі оновить:
- Claude Code CLI (`npm update -g @anthropic-ai/claude-code`)
- Decod RE рушій (`git pull origin main`)
- Бібліотеки Python (`pip install --upgrade frida lief capstone psutil pyright`)

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

### 2. FreeLLMAPI Configuration (100% Free AI)

Mnemosyne connects to free model providers at $0 cost via **FreeLLMAPI**:

1. **Obtain Free API Keys**:
   - **Google Gemini API**: Visit [Google AI Studio](https://aistudio.google.com/) $\rightarrow$ generate a free API key (**1,500 requests/day**, 15 RPM).
   - **Groq Cloud (Llama 3.3 70B)**: Visit [Groq Console](https://console.groq.com/) $\rightarrow$ generate a free key (**14,400 requests/day**, 30 RPM).
   - **OpenRouter Free Tier**: Generate a key on [OpenRouter](https://openrouter.ai/).
2. **Install FreeLLMAPI**:
   - Open Mnemosyne $\rightarrow$ navigate to **⚙ Settings**.
   - Click **"Download & install FreeLLMAPI"**. Mnemosyne verifies SHA-256 against official GitHub release metadata and downloads the package.
   - Open FreeLLMAPI Dashboard (`http://127.0.0.1:31415`).
   - Add your free provider keys in the **Providers** tab.
   - Under **Keys $\rightarrow$ Anthropic**, route `Sonnet`, `Opus`, and `Haiku` to your free models (e.g., Gemini Flash / Llama 3.3).
3. **Connect to Mnemosyne**:
   - Copy the Unified API Key from FreeLLMAPI.
   - Paste it into **"Unified API Key"** in Mnemosyne Settings and click **"Save key"** (securely stored in OS credential store).
   - Click **"Check connection"** $\rightarrow$ shows green `✓ Connected`.
   - Click **"Test request"** to verify end-to-end messaging.

---

### 3. Claude Code CLI Setup

1. If not installed yet, click **"Install Claude Code CLI"** in Mnemosyne Settings (requires Node.js).
2. Or run manually in your terminal:
   ```bash
   npm install -g @anthropic-ai/claude-code
   ```
3. Mnemosyne automatically discovers Claude Code CLI across standard Windows and Linux locations.

---

### 4. Decod MCP Engine Integration (46 Tools)

Decod provides Claude Code with a comprehensive reverse engineering, diagnostics, and modding toolkit.
1. In the `Decod` directory, run one-time dependency installation:
   ```bash
   python setup_decod.py --install-all
   ```
   This installs `frida`, `lief`, `capstone`, `psutil`, `pyright`, `rust-analyzer`, downloads `repak` and `uesave`, and builds `libstealth.so` on Linux.
2. In **Mnemosyne Settings**, ensure **"Enable Decod MCP"** is checked (enabled by default).
3. Click **"Verify"** to test JSON-RPC MCP connectivity.

---

### 5. Vibe-Coding Superpowers

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

---

### 6. One-Click Updates & Maintenance

In **Settings $\rightarrow$ Updates & Maintenance**, click **"Update All Components Now"** to update:
- Claude Code CLI (`npm update -g @anthropic-ai/claude-code`)
- Decod RE engine (`git pull origin main`)
- Python packages (`pip install --upgrade frida lief capstone psutil pyright`)
