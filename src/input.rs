//! Bounded local structure loading, decompression, and format dispatch.
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use flate2::read::MultiGzDecoder;

use crate::{Error, Structure, cif_input, pdb_input, xml_input};

// A generous macromolecular text limit also bounds decompression of untrusted gzip files.
pub(crate) const MAX_INPUT_BYTES: u64 = 1024 * 1024 * 1024;

/// Load PDB, mmCIF, or PDBML coordinates, optionally gzip compressed.
///
/// Decompressed input is limited to 1 GiB. Recognized content takes precedence over
/// the filename; extension dispatch handles inputs without a recognizable header.
///
/// # Errors
/// Returns `Error::Io` for file access or read failures and `Error::Parse` for invalid
/// UTF-8, unsupported or malformed formats, invalid gzip data, or inputs over the limit.
pub fn load_structure(path: impl AsRef<Path>) -> Result<Structure, Error> {
    let path = path.as_ref();
    let io_error = |source| Error::Io {
        path: path.to_owned(),
        source,
    };
    let file = File::open(path).map_err(io_error)?;
    let mut reader = BufReader::new(file);
    let gzip = reader
        .fill_buf()
        .map_err(io_error)?
        .starts_with(&[0x1f, 0x8b]);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let compressed_extension = name.ends_with(".gz");
    if compressed_extension && !gzip {
        return Err(input_error(
            "file ends in .gz but does not contain gzip data",
        ));
    }
    let mut source: Box<dyn Read> = if gzip {
        Box::new(MultiGzDecoder::new(reader))
    } else {
        Box::new(reader)
    };
    let mut bytes = Vec::new();
    source
        .by_ref()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(input_error(
            "decompressed structure exceeds the 1 GiB input limit",
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| input_error(format!("structure text must be UTF-8: {error}")))?;
    let text = text.trim_start_matches('\u{feff}');
    let beginning = text.trim_start();
    let base = name.strip_suffix(".gz").unwrap_or(&name);
    let extension = base.rsplit('.').next().unwrap_or("");
    if beginning.starts_with('<') {
        xml_input::parse_xml(text)
    } else if beginning.get(..5).is_some_and(|prefix| {
        prefix.eq_ignore_ascii_case("data_") || prefix.eq_ignore_ascii_case("loop_")
    }) || beginning.starts_with('_')
    {
        cif_input::parse_cif(text)
    } else {
        match extension {
            "pdb" | "ent" => pdb_input::parse_pdb(text),
            "cif" | "mmcif" => cif_input::parse_cif(text),
            "xml" | "pdbml" => xml_input::parse_xml(text),
            _ if text
                .lines()
                .any(|line| line.starts_with("ATOM  ") || line.starts_with("HETATM")) =>
            {
                pdb_input::parse_pdb(text)
            }
            _ => Err(input_error(
                "cannot identify structure format; use .pdb, .ent, .cif, .mmcif, .xml, or .pdbml (optionally .gz)",
            )),
        }
    }
}

fn input_error(message: impl ToString) -> Error {
    Error::Parse {
        format: "structure",
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use flate2::{Compression, write::GzEncoder};

    use super::*;

    fn load_bytes(suffix: &str, bytes: &[u8]) -> Result<Structure, Error> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "oxillustrate_input_{}_{}.{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            suffix
        ));
        std::fs::write(&path, bytes).unwrap();
        let result = load_structure(&path);
        std::fs::remove_file(&path).unwrap();
        result
    }

    #[test]
    fn gzip_and_plain_inputs_share_dispatch_and_errors() {
        let xml = r#"<datablock><atom_siteCategory><atom_site id="1"><type_symbol>C</type_symbol><label_atom_id>CA</label_atom_id><label_comp_id>GLY</label_comp_id><Cartn_x>1</Cartn_x><Cartn_y>2</Cartn_y><Cartn_z>3</Cartn_z></atom_site></atom_siteCategory></datablock>"#;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(xml.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        let plain = load_bytes("pdbml", xml.as_bytes()).unwrap();
        assert_eq!(load_bytes("xml.gz", &compressed).unwrap(), plain);
        assert_eq!(load_bytes("unknown", &compressed).unwrap(), plain);
        assert!(load_bytes("xml.gz", xml.as_bytes()).is_err());
        assert!(load_bytes("xml.gz", &compressed[..compressed.len() - 5]).is_err());
        assert!(load_bytes("pdb", &[0xff]).is_err());
        assert!(load_bytes("unknown", b"not coordinates").is_err());
    }
}
