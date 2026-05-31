//! `.mnlgprog` / `.mnlglib` Korg Sound Librarian container I/O.
//!
//! Both are ZIP archives. A library lists its programs in `FileInformation.xml`
//! (`<KorgMSLibrarian_Data><Product>minilogue</Product><Contents …>` with one
//! `<Program>` per slot naming `Prog_NNN.prog_bin` + `Prog_NNN.prog_info`). Each
//! `.prog_bin` is the 448-byte unpacked program payload ([`Program`]); each
//! `.prog_info` is small XML metadata. A single `.mnlgprog` is the same shape
//! with one program.
//!
//! The reader is lenient (parses `FileInformation.xml` when present, else globs
//! `*.prog_bin`) so it accepts real Librarian exports. The writer emits the
//! structure above. The 448-byte payload round-trips byte-exact; the ZIP
//! container itself is not byte-identical to Korg's (timestamps/compression
//! differ) but re-imports to the same program.

use std::io::{Cursor, Read, Write};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::codec::CodecError;
use crate::program::Program;

/// Per-program metadata from `Prog_NNN.prog_info`.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programmer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// A program plus its container metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MnlgProgram {
    pub program: Program,
    pub info: ProgInfo,
}

fn container<E: std::fmt::Display>(e: E) -> CodecError {
    CodecError::Container(e.to_string())
}

/// Read a `.mnlgprog` / `.mnlglib` archive into its programs.
pub fn read_library(bytes: &[u8]) -> Result<Vec<MnlgProgram>, CodecError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(container)?;
    let names: Vec<String> = archive.file_names().map(String::from).collect();

    // Prefer the order declared in FileInformation.xml; fall back to globbing.
    let mut bins: Vec<String> = read_file(&mut archive, "FileInformation.xml")
        .ok()
        .map(|b| all_tag_texts(&String::from_utf8_lossy(&b), "ProgramBinary"))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            let mut g: Vec<String> = names
                .iter()
                .filter(|n| n.ends_with(".prog_bin"))
                .cloned()
                .collect();
            g.sort();
            g
        });
    bins.retain(|n| names.contains(n));
    if bins.is_empty() {
        return Err(CodecError::Container("no .prog_bin entries found".into()));
    }

    let mut out = Vec::with_capacity(bins.len());
    for bin in bins {
        let payload = read_file(&mut archive, &bin)?;
        let program = Program::from_bytes(&payload)?;
        let info_name = bin.replace(".prog_bin", ".prog_info");
        let info = if names.contains(&info_name) {
            parse_prog_info(&String::from_utf8_lossy(&read_file(
                &mut archive,
                &info_name,
            )?))
        } else {
            ProgInfo::default()
        };
        out.push(MnlgProgram { program, info });
    }
    Ok(out)
}

/// Read a single-program `.mnlgprog` (the first program in the archive).
pub fn read_mnlgprog(bytes: &[u8]) -> Result<MnlgProgram, CodecError> {
    read_library(bytes)?
        .into_iter()
        .next()
        .ok_or_else(|| CodecError::Container("empty archive".into()))
}

/// Write programs to a `.mnlglib` / `.mnlgprog` archive (single program → prog).
pub fn write_library(programs: &[MnlgProgram]) -> Result<Vec<u8>, CodecError> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();

    zip.start_file("FileInformation.xml", opts)
        .map_err(container)?;
    zip.write_all(file_information_xml(programs.len()).as_bytes())
        .map_err(container)?;

    for (i, mp) in programs.iter().enumerate() {
        zip.start_file(format!("Prog_{i:03}.prog_bin"), opts)
            .map_err(container)?;
        zip.write_all(&mp.program.to_bytes()?).map_err(container)?;
        zip.start_file(format!("Prog_{i:03}.prog_info"), opts)
            .map_err(container)?;
        zip.write_all(prog_info_xml(&mp.info).as_bytes())
            .map_err(container)?;
    }

    Ok(zip.finish().map_err(container)?.into_inner())
}

/// Write a single program as a `.mnlgprog`.
pub fn write_mnlgprog(program: &Program, info: &ProgInfo) -> Result<Vec<u8>, CodecError> {
    write_library(&[MnlgProgram {
        program: program.clone(),
        info: info.clone(),
    }])
}

fn read_file(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<Vec<u8>, CodecError> {
    let mut f = archive.by_name(name).map_err(container)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(container)?;
    Ok(buf)
}

// --- tiny XML helpers (the files are small and fixed-shape) ---

fn parse_prog_info(xml: &str) -> ProgInfo {
    let pick = |t: &str| tag_text(xml, t).filter(|s| !s.is_empty()).map(String::from);
    ProgInfo {
        programmer: pick("Programmer"),
        comment: pick("Comment"),
    }
}

fn tag_text<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let start = xml.find(&format!("<{tag}>"))? + tag.len() + 2;
    let end = xml[start..].find(&format!("</{tag}>"))? + start;
    Some(xml[start..end].trim())
}

fn all_tag_texts(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(s) = rest.find(&open) {
        let from = s + open.len();
        if let Some(e) = rest[from..].find(&close) {
            out.push(rest[from..from + e].trim().to_string());
            rest = &rest[from + e + close.len()..];
        } else {
            break;
        }
    }
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn file_information_xml(n: usize) -> String {
    let mut programs = String::new();
    for i in 0..n {
        programs.push_str(&format!(
            "    <ProgramData>\n      <Information>Prog_{i:03}.prog_info</Information>\n      <ProgramBinary>Prog_{i:03}.prog_bin</ProgramBinary>\n    </ProgramData>\n"
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<KorgMSLibrarian_Data>\n  <Product>minilogue</Product>\n  <Contents NumProgramData=\"{n}\" NumPresetInformation=\"0\" NumTuneScaleData=\"0\" NumTuneOctData=\"0\" NumFavoriteData=\"0\">\n{programs}  </Contents>\n</KorgMSLibrarian_Data>\n"
    )
}

fn prog_info_xml(info: &ProgInfo) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<minilogue_ProgramInformation>\n  <Programmer>{}</Programmer>\n  <Comment>{}</Comment>\n</minilogue_ProgramInformation>\n",
        xml_escape(info.programmer.as_deref().unwrap_or("")),
        xml_escape(info.comment.as_deref().unwrap_or("")),
    )
}
