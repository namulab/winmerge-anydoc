# winmerge-anydoc

A WinMerge unpacker plugin that converts Microsoft Office documents to Markdown using [anydoc](https://github.com/firecrawl/anydoc), allowing their contents to be compared as text in WinMerge.

## Features

- Converts supported Office documents to Markdown before comparison
- Works as a native WinMerge COM unpacker plugin
- Uses `anydoc` directly as a Rust library
- No external converter executable is required
- Intended for comparison only; modified Markdown is not written back to the original Office document

## Supported file extensions

| Type | Extensions |
|---|---|
| Word | `.doc`, `.docx`, `.docm` |
| Excel | `.xls`, `.xlsx`, `.xlsm`, `.xlsb` |
| PowerPoint | `.ppt`, `.pps`, `.pot`, `.pptx`, `.pptm`, `.ppsx`, `.ppsm` |

## Requirements

- Windows 10 or 11 x64
- 64-bit [WinMerge](https://winmerge.org/)
- Microsoft Visual C++ 2015-2026 Redistributable (x64)

## Installation

1. Download the release package.
2. Copy `compare_office_with_anydoc.dll` to the `MergePlugins` directory of your WinMerge installation.
3. Restart WinMerge.
4. Compare supported Office documents as usual.

WinMerge can select the plugin automatically for supported file extensions. It can also be selected manually as an unpacker plugin.

## Building from source

### Requirements

- Rust 1.97.1
- Visual Studio with MSVC C++ build tools
- Windows SDK with MIDL and Resource Compiler tools

The required Rust toolchain is defined in `rust-toolchain.toml`.

Build the release DLL:

```powershell
cargo build --release --lib
```

The resulting plugin is:

```text
target\release\compare_office_with_anydoc.dll
```

## Testing

Build the debug DLL:

```powershell
cargo build --lib
```

Run the COM plugin smoke test:

```powershell
.\tests\com-plugin\test-com-plugin.ps1
```

The test uses a synthetic Office document contained in the repository and writes temporary conversion output only to the system temporary directory.

## License checks

Third-party Rust dependency licenses are checked with [cargo-deny](https://github.com/EmbarkStudios/cargo-deny):

```powershell
cargo deny check licenses
```

Third-party license notices are generated with [cargo-about](https://github.com/EmbarkStudios/cargo-about):

```powershell
cargo about generate --locked --fail -o .\THIRD_PARTY_NOTICES.md .\about.hbs
```

`THIRD_PARTY_NOTICES.md` is generated automatically and should not be edited manually.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).

Third-party software licenses and copyright notices are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Copyright (c) 2026 namulab