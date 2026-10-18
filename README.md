# MdPad

MdPad is an ultra-lightweight, standalone native Windows desktop application specifically built for Markdown. It delivers the distraction-free simplicity of classic Windows Notepad combined with instant viewing and editing modes.

## Highlights
- **Sub-50ms Cold Start**: Written in native Rust with Slint for zero-runtime startup latency and minimal memory footprint.
- **Single Standalone Executable**: Self-contained `.exe` binary with zero external DLL requirements.
- **Reading Mode by Default**: Opens markdown documents directly into a clean rendered document with zero syntax noise.
- **Single Window Lifecycle**: One window per document, strictly tabless.
- **Full Encoding Integrity**: Automatic UTF-8 BOM detection and roundtrip preservation of original CRLF versus LF line endings.

## The 4 Viewing and Editing Modes
1. **Mode 1: Reading Mode (Default)**
   - Clean rendered layout with all syntax markers hidden.
   - Headings, lists, code fences, blockquotes, and tables render cleanly.
   - Task checkboxes (`- [ ]` / `- [x]`) are fully interactive. Clicking a checkbox toggles the markdown state in place.
2. **Mode 2: Live Preview Mode**
   - Inline hybrid formatting inspired by Obsidian.
   - Renders visually while allowing direct editing of the active text block.
3. **Mode 3: Source Mode**
   - Proportional document font with dimmed syntax tokens (`#`, `**`, `-`, `---`).
   - Headings are bold and distinguished, with zero cursor jump glitches during editing.
4. **Mode 4: Raw Plain Text Mode**
   - Strict unstyled monospaced text (`Consolas`), identical to classic Windows Notepad.
   - **Invisibility Rule**: The dynamic formatting toolbar automatically hides in this mode.

## Dynamic Top Toolbar
Offers quick formatting actions:
- Headings (`H`)
- Bold (`B`)
- Italic (`I`)
- Bullets (`- List`)
- Checkboxes (`[ ] Task`)
- Blockquotes (`Quote`)
- Links (`Link`)
- Tables (`Table`)

Visible in Modes 1, 2, and 3. Automatically hides in Mode 4.

## Unified Find and Replace
- Compact find bar (`Ctrl+F`), expandable into full replacement drawer (`Ctrl+H`).
- Match counter with instant search feedback.
- Case sensitive matching (`Aa`).
- Whole word matching (`\b`).
- Mutually exclusive toggle between Normal, Fuzzy (`~`), and Regex (`.*`) search modes.

## Bottom Status Bar
Provides real-time document telemetry:
- Line and Column coordinates
- Word and character count
- Encoding format (`UTF-8` or `UTF-8 BOM`)
- Line ending format (`CRLF` or `LF`)
- Zoom level indicator

## Building and Testing

### Prerequisites
- Rust compiler with the `x86_64-pc-windows-gnu` target

### Build Release Executable
```powershell
cargo build --release
```
The optimized binary is generated at `target/release/mdpad.exe`.

### Run Deterministic Tests
```powershell
cargo test
```
Executes the full test suite covering backend encoding roundtrips, search engine modes, and markdown block parsing.

### Run Cold-Start Benchmark
```powershell
pwsh -File benches/bench_startup.ps1 -Iterations 5 -Config release
```

## License
Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
