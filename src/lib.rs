// rustc deliberately marks cdylib exports as PRIVATE in its generated .def file.
// MSVC reports that valid choice as LNK4104, which rustc otherwise surfaces as
// a linker_messages warning.
#![allow(linker_messages)]

pub mod winmerge;

mod com;
mod converter;
