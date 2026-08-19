# winmerge-anydoc

English | [日本語](README.ja.md)

**Compare Word, Excel, and PowerPoint files in WinMerge. No Microsoft Office or Python required.**

`winmerge-anydoc` is a WinMerge unpacker plugin for comparing Office documents.

Instead of extracting documents as simple plain text, it converts them to Markdown while preserving structures such as tables, headings, and lists as much as possible, and then compares the result in WinMerge.

**Installation requires only copying a single DLL to the `MergePlugins` folder.**

## Comparison example

**Viewed in Word**

![Document displayed in Word](docs/images/word-document.png)

*Example of a Word (`.docx`) document containing headings, bullet lists, numbered lists, and a table.*

**Comparison in WinMerge**

![Comparing Word files with winmerge-anydoc](docs/images/word-comparison.png)

*Example of comparing two Word (`.docx`) files with winmerge-anydoc. The document structure remains visible in the comparison as Markdown.*

## Features

### Single-DLL installation

All functionality required for document conversion is built into `compare_office_with_anydoc.dll`.

You do not need to install or prepare any of the following:

* Microsoft Office
* Python environment
* Python packages
* Helper executables
* External document converters
* Visual C++ Redistributable

Simply place the DLL in the appropriate folder and restart WinMerge.

### Compare while preserving structures such as tables and headings

Instead of extracting Office document contents as simple plain text, the plugin converts them to Markdown for comparison.

Depending on the document content and format, structures such as the following can remain visible in the comparison:

* Headings
* Lists
* Tables

This can make differences easier to understand than simple text extraction, especially for Excel tables or changes to the document structure itself.

Internally, documents are converted to **GitHub-Flavored Markdown**.

### Microsoft Office is not required

The plugin can be used even on PCs where Word, Excel, and PowerPoint are not installed.

Rather than launching and controlling Office applications through COM, it parses the document files directly.

### No external converter process

Document conversion is powered by the Rust library [anydoc](https://github.com/firecrawl/anydoc).

Because `anydoc` is linked directly into the DLL as a library, no separate converter or helper program is launched during conversion.

Document conversion runs inside the WinMerge plugin.

### Fast local conversion

`anydoc` is a Pure Rust library designed for fast conversion of documents to Markdown.

According to the official anydoc benchmark, the median conversion time for its benchmark documents is reported to be under 5 milliseconds.

This figure represents the conversion time of `anydoc` itself and does not indicate the overall performance of the complete WinMerge comparison process.

See the [anydoc benchmark](https://github.com/firecrawl/anydoc#benchmark) for details.

## Supported file formats

| Type       | Extensions                                                 |
| ---------- | ---------------------------------------------------------- |
| Word       | `.doc`, `.docx`, `.docm`                                   |
| Excel      | `.xls`, `.xlsx`, `.xlsm`, `.xlsb`                          |
| PowerPoint | `.ppt`, `.pps`, `.pot`, `.pptx`, `.pptm`, `.ppsx`, `.ppsm` |

Both formats commonly used with Office 97-2003, such as `.doc`, `.xls`, and `.ppt`, and newer formats such as `.docx`, `.xlsx`, and `.pptx` are supported.

## Requirements

* Windows 10 or Windows 11 x64
* 64-bit [WinMerge](https://winmerge.org/)

Microsoft Office, Python, and an external document converter are not required.

## Installation and usage

1. Download the latest ZIP package from [GitHub Releases](https://github.com/namulab/winmerge-anydoc/releases).
2. Extract the ZIP file.
3. Copy `compare_office_with_anydoc.dll` to the `MergePlugins` folder of your WinMerge installation.
4. Restart WinMerge.
5. From the WinMerge menu, enable **[Plugins] → [Automatic Unpacking]**.
6. Compare supported Office documents as usual.

If [Automatic Unpacking] is already enabled, no settings need to be changed.

When automatic unpacking is enabled, WinMerge automatically selects an unpacker plugin that supports the file extension.

If multiple unpacker plugins supporting the same extension are installed, for example, you can also manually select `winmerge-anydoc` as the unpacker plugin.

## How it differs from other approaches

There are several approaches to comparing Office documents in WinMerge.

| Approach                                   | Typical characteristics                                                   | winmerge-anydoc                                                                          |
| ------------------------------------------ | ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| xdocdiff / xdoc2txt-style tools            | Extract document contents as text for comparison                          | Converts to Markdown and preserves structures such as tables and headings where possible |
| Python-based conversion such as MarkItDown | Requires a Python environment and format-specific packages                | No Python required. Runs with only the DLL                                               |
| Office automation                          | Uses installed Microsoft Office applications to open or convert documents | No Office required. Parses document files directly                                       |
| External converter approach                | Launches a separate converter program and passes the result to WinMerge   | Embeds the conversion library in the DLL and performs conversion inside the plugin       |

This comparison mainly describes differences in **deployment and document conversion methods**.

Each tool has different supported formats and use cases, and `winmerge-anydoc` focuses on making Office document comparison easy to use in WinMerge.

## How it works

```text
Office document
    |
    v
compare_office_with_anydoc.dll
    |
    |  anydoc (embedded as a Rust library)
    v
GitHub-Flavored Markdown
    |
    v
Text comparison by WinMerge
```

`anydoc` is not executed as an external program. It is linked directly into the plugin DLL as a Rust library.

Therefore, instead of using a process such as:

```text
WinMerge
  -> Launch external program
  -> Convert document
  -> Return result to WinMerge
```

the document-to-Markdown conversion is performed inside the WinMerge plugin.

The conversion is one-way and intended for comparison.

Editing the converted Markdown in the comparison view does not write changes back to the original Office document.

## Building from source

### Requirements

* Rust 1.97.1
* Visual Studio with MSVC C++ build tools
* Windows SDK with MIDL and Resource Compiler tools

The required Rust toolchain and target are defined in `rust-toolchain.toml`.

Build the release version:

```powershell
cargo build --release
```

The generated plugin DLL is located at:

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

The test uses a test Office document contained in the repository, and temporary conversion output is written only to the system temporary directory.

## Acknowledgements

Document conversion is powered by [anydoc](https://github.com/firecrawl/anydoc).

anydoc is a Rust library for converting Office documents and other formats to GitHub-Flavored Markdown.

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.

Third-party software licenses and copyright notices are listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Copyright (c) 2026 namulab
