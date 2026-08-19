# winmerge-anydoc

**Compare Microsoft Office documents in WinMerge as structured Markdown — with a single native DLL.**

No Microsoft Office. No Python runtime. No helper executable. No external converter process.

`winmerge-anydoc` is a native WinMerge unpacker plugin powered by [anydoc](https://github.com/firecrawl/anydoc).
It converts supported Microsoft Office documents directly to GitHub-Flavored Markdown in-process, then lets WinMerge compare the resulting text.

## Why winmerge-anydoc?

Office pre-processing plugins for WinMerge already exist. The goal of this plugin is not simply to extract text from Office files, but to provide a lightweight, structured, dependency-free comparison path.

### Single-DLL installation

The converter is linked directly into `compare_office_with_anydoc.dll`.

There is no Python environment to prepare, no package installation, no helper executable, and no separate converter process to manage. The distributed plugin also does not require a separate Visual C++ Redistributable.

Install the DLL, restart WinMerge, and compare documents.

### Structured Markdown comparison

Instead of flattening documents into plain extracted text, `winmerge-anydoc` converts them to GitHub-Flavored Markdown.

This gives WinMerge a more structured representation of the document, allowing elements such as headings, lists, and tables to remain visible in the comparison where the source format permits.

This is especially useful when changes are structural rather than limited to individual words.

### Microsoft Office is not required

Documents are parsed directly.

Microsoft Word, Excel, and PowerPoint do not need to be installed, and the plugin does not automate Office through COM.

### Native in-process conversion

`anydoc` is used directly as a Rust library inside the WinMerge plugin.

There is no shell-out to another converter. Document conversion happens inside the plugin process before WinMerge performs the text comparison.

### Fast conversion powered by anydoc

`anydoc` is a pure-Rust document-to-Markdown converter designed for fast local conversion.

Its upstream benchmark reports a median conversion time under 5 ms across its benchmark corpus. These figures measure `anydoc` conversion itself and are not an end-to-end WinMerge benchmark.

See the [anydoc benchmark](https://github.com/firecrawl/anydoc#benchmark) for methodology and current results.

## How it differs from common approaches

| Approach | Typical characteristics | winmerge-anydoc |
|---|---|---|
| xdocdiff / xdoc2txt-style text extraction | Extracts document contents for text comparison | Converts to Markdown so useful document structure can remain visible |
| MarkItDown-based integration | Python package with format-specific dependencies | Native DLL with no Python environment or package setup |
| Office automation plugins | Use installed Microsoft Office applications to open or export documents | Parses documents directly; Microsoft Office is not required |
| External converter plugins | Launch a helper program or converter process | Conversion is performed in-process through the linked Rust library |

The comparison above is about deployment and conversion architecture. Different tools support different formats and use cases, and `winmerge-anydoc` is intentionally focused on Office document comparison in WinMerge.

## Supported file extensions

| Type | Extensions |
|---|---|
| Word | `.doc`, `.docx`, `.docm` |
| Excel | `.xls`, `.xlsx`, `.xlsm`, `.xlsb` |
| PowerPoint | `.ppt`, `.pps`, `.pot`, `.pptx`, `.pptm`, `.ppsx`, `.ppsm` |

Both legacy binary Office formats and modern OOXML formats are supported.

## Requirements

- Windows 10 or 11 x64
- 64-bit [WinMerge](https://winmerge.org/)

Microsoft Office, Python, and an external document converter are **not** required.

## Installation

1. Download the latest ZIP package from [GitHub Releases](https://github.com/namulab/winmerge-anydoc/releases).
2. Extract the package.
3. Copy `compare_office_with_anydoc.dll` to the `MergePlugins` directory of your WinMerge installation.
4. Restart WinMerge.
5. Compare supported Office documents as usual.

WinMerge can select the plugin automatically for supported file extensions. It can also be selected manually as an unpacker plugin.

The release package contains:

```text
compare_office_with_anydoc.dll
LICENSE
THIRD_PARTY_NOTICES.md
README.md
```

## How it works

```text
Office document
      |
      v
compare_office_with_anydoc.dll
      |
      |  anydoc (linked Rust library)
      v
GitHub-Flavored Markdown
      |
      v
WinMerge text comparison
```

The conversion is one-way and intended only for comparison. Modified Markdown is not written back to the original Office document.

## Building from source

### Requirements

- Rust 1.97.1
- Visual Studio with MSVC C++ build tools
- Windows SDK with MIDL and Resource Compiler tools

The required Rust toolchain and target are defined in `rust-toolchain.toml`.

Build the release version:

```powershell
cargo build --release
```

The resulting plugin DLL is:

```text
target\release\compare_office_with_anydoc.dll
```

The release build statically links the MSVC C runtime used by the plugin.

## Testing

Build the debug version:

```powershell
cargo build
```

Run the COM plugin smoke test:

```powershell
.\tests\com-plugin\test-com-plugin.ps1
```

The test uses a synthetic Office document contained in the repository and writes temporary conversion output only to the system temporary directory.

## Releases

GitHub Releases are built automatically from version tags.

For example, pushing tag `v0.1.0` builds the release DLL on GitHub Actions, packages the DLL with the license and documentation files, and publishes:

```text
winmerge-anydoc-v0.1.0.zip
```

The workflow verifies that the Git tag matches the version in `Cargo.toml` before publishing the release.

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

## Acknowledgements

Document conversion is powered by [anydoc](https://github.com/firecrawl/anydoc), a Rust library for converting Office and other document formats to GitHub-Flavored Markdown.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).

Third-party software licenses and copyright notices are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Copyright (c) 2026 namulab
