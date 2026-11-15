# 🌌 Mnemosyne

> **Cross-Platform Vibe-Coding Workspace & Coding Agent Host**  
> **Кросплатформний десктопний простір для вайбкодингу та автономних агентів**

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey.svg)](#)
[![Decod MCP](https://img.shields.io/badge/Decod%20MCP-46%20Tools-purple.svg)](https://github.com/VorwerkUa2/DFM)
[![FreeLLMAPI](https://img.shields.io/badge/AI%20Cost-%240%20(FreeLLMAPI)-green.svg)](SETUP_GUIDE.md)

---

## 📖 Documentation & Guides / Документація

- **🇺🇦 Повний посібник із встановлення та налаштування:** [SETUP_GUIDE.md (Українська)](SETUP_GUIDE.md#-українська-версія)
- **🇬🇧 Complete Installation & Configuration Guide:** [SETUP_GUIDE.md (English)](SETUP_GUIDE.md#-english-version)

---

## ⚡ Quick Start / Швидкий старт

### 🪟 Windows 10 / 11
Завантажте та запустіть автономний інсталятор:
- **`Mnemosyne-Setup.exe`** (1-клік встановлення, ярлики на Робочому столі та в Меню Пуск, налаштування деінсталятора).
- Або двічі клікніть **`install.bat`** у корені репозиторію.

### 🐧 Linux (Void Linux, Fedora, Arch, Ubuntu/Debian)
Встановлення однією командою в терміналі:
```bash
curl -fsSL https://raw.githubusercontent.com/VorwerkUa2/Mnemosyne/main/install.sh | sh
```
Скрипт автоматично встановить залежності для вашого дистрибутива (`xbps`, `dnf`, `pacman`, `apt`), помістить бінарник у `~/.local/bin/mnemosyne` та додасть ярлик у меню програм.

---

## ✨ Features / Можливості

### 🎨 Інтерфейс та Локалізація (Bilingual & Dark Theme)
- **Темна тема за замовчуванням**: Глибокі відтінки сланцю (`#121218`) з м'яким фіолетово-синім акцентом. Миттєве перемикання на світлу тему.
- **Двомовний інтерфейс**: Повна підтримка **Української 🇺🇦** та **Англійської 🇬🇧** мов із перемиканням на льоту в Налаштуваннях.
- **Вбудований переглядач та редактор**: Підтримка тексту, Markdown (із рендерингом таблиць і тасклістів) та зображень (PNG, JPEG, GIF, WebP, BMP).

### 🤖 Інтеграція Claude Code CLI та FreeLLMAPI ($0 Вартість)
- **Потокові відповіді в реальному часі** з відображенням активних інструментів і можливістю скасування запиту в будь-який момент.
- **Підключення до FreeLLMAPI**: 100% безкоштовний кодинг через пули **Google Gemini API** (1 500 запитів/добу) та **Groq Cloud** (14 400 запитів/добу).
- **Збереження Unified API Key**: Ключі шифруються у системному сховищі облікових даних ОС.
- **Наскрізний тест з'єднання**: Кнопка перевірки пінгу та валідації моделі прямо з GUI.

### 🛡 Безпека та Режими агента
- **📋 План (Plan Mode)**: Тільки читання (`Read`, `Glob`, `Grep`). Зміни файлів заблоковані.
- **⚡ Turbo (File Edits with Approval)**: Модель може змінювати файли, але кожна дія вимагає підтвердження користувача з переглядом diff.

### ⚡ Суперсили кодингу (Vibe-Coding Superpowers)
- **Режим Caveman**: Економія 30–40% вихідних токенів шляхом усунення балаканини, формальностей і пустих прелюдій.
- **Принципи Карпатого (Karpathy Rules)**: Дотримання YAGNI, мінімум непотрібних абстракцій, збереження стилю репозиторію та захисне програмування.
- **Автопровізіонування `.claude/CLAUDE.md`**: Автоматичне визначення стеку проєкту (Rust, Python, C/C++, C#, Go, Lua, Web) і генерація індивідуальних директив для агента.
- **Кнопка `🔍 Перевірити діагностику`**: Миттєвий запуск `cargo check` або перевірки синтаксису Python прямо з інтерфейсу з **0 витратою токенів**.

### 🛠 Decod Reverse Engineering & Modding MCP (46 Інструментів)
Автоматично підключає повноцінний RE-рушій [Decod (DFM)](https://github.com/VorwerkUa2/DFM):
- **Стелс & Динаміка**: Frida runtime hooking, Scylla dumper, захист від анти-відлагодження (`libstealth.so`).
- **Статика & Дизасемблювання**: Capstone disassembler, LIEF аналіз бінарників (PE, ELF, Mach-O), Headless C decompiler.
- **Ігрові рушії**: Розпакування та пакування UE5 PAK / UTOC / UCAS, редагування сейвів (`uesave`), аналіз 3D моделей (OBJ) та маніпуляції аудіо/відео через FFmpeg.
- **Code Intelligence**: AST аналіз структури (`inspect_code_structure`) та хірургічне вилучення функцій (`extract_function_definition`), що знижує витрати токенів на **70%–85%**.

### 🔄 Оновлення та обслуговування в 1 клік
Кнопка в Налаштуваннях дозволяє оновити всі компоненти у фоні:
- Claude Code CLI (`npm update -g @anthropic-ai/claude-code`)
- Decod RE-рушій (`git pull origin main`)
- Модулі Python (`pip install --upgrade frida lief capstone psutil pyright`)

---

## 🛠 Збірка з вихідного коду (Building from Source)

```bash
# Клонування репозиторію
git clone https://github.com/VorwerkUa2/Mnemosyne.git
cd Mnemosyne

# Запуск у режимі розробки
cargo run

# Збірка оптимізованого релізу
cargo build --release

# Запуск тестів (усі 37 тестів)
cargo test
```

---

## 📜 Ліцензія

Проєкт розповсюджується під ліцензією [MIT](LICENSE).
