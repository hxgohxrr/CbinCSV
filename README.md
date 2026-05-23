# CBinCSV

A desktop tool for extracting, editing, and reimporting dialogue text from Level-5 `.cfg.bin` files. Designed for game translation workflows.

## Features

- Import `.cfg.bin` files by file, multiple files, or entire folder
- Display all text entries in an editable multi-language pivot table
- Export to CSV (single file or one per event)
- Reimport edited CSV back to `.cfg.bin`
- Supports Standard (index-based) and NNK (address-based) parser modes
- Automatic language detection from filename (`ev00_0010_ja.cfg.bin` → `ja`)
- Configurable theme (dark/light/system) and accent color
- Interface in Spanish, English, French, German, Japanese, and Italian

## File Naming Convention

Files must follow the Level-5 naming pattern:

```
evXX_XXXX_<lang>.cfg.bin
```

Where `<lang>` is a two-letter language code: `ja`, `es`, `en`, `fr`, `de`, `it`.

## Supported Games

| Game type | Parser mode | Notes |
|-----------|-------------|-------|
| Standard Level-5 | Standard | Index-based full rebuild |
| Ni no Kuni (NNK) | NNK | Address-based in-place patch |

## CSV Format

```
EV_NAME;INDEX;JA;ES
ev00_0010;0;日本語テキスト;Texto en español
ev00_0010;1;次のテキスト;
```

- `EV_NAME`: base event name without language suffix
- `INDEX`: text entry index within the file
- One column per language; empty cells = untranslated

## Building from Source

**Prerequisites:** Rust 1.78+, Node.js 20+

### Windows
```bash
git clone https://github.com/hxgohxrr/CbinCSV.git
cd cbincsv
npm install
npm run tauri build
# Output: src-tauri/target/release/bundle/msi/  and  /nsis/
```

### macOS
```bash
# Xcode Command Line Tools required: xcode-select --install
git clone https://github.com/hxgohxrr/CbinCSV.git
cd cbincsv
npm install
npm run tauri build
# Output: src-tauri/target/release/bundle/dmg/  and  /macos/
```

### Linux
```bash
# Ubuntu/Debian — install WebKit2GTK and other deps:
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

git clone https://github.com/hxgohxrr/CbinCSV.git
cd cbincsv
npm install
npm run tauri build
# Output: src-tauri/target/release/bundle/appimage/  and  /deb/
```

## Usage

1. **Import** — click "Import files" or drag `.cfg.bin` files onto the window
2. **Edit** — click any cell in the table to edit the text directly
3. **Export CSV** — choose format (single/per-file), select languages, click Export
4. **Reimport** — click "Reimport CSV", select your edited CSV, then choose output folder

The original `.cfg.bin` files are never overwritten; output always goes to a folder you select.

## Language Support

The UI language defaults to the operating system locale. If the OS locale is not supported, it falls back to Spanish. Supported UI languages: Spanish, English, French, German, Japanese, Italian.
