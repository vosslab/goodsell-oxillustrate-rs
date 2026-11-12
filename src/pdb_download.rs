//! Synchronous RCSB coordinate loading without an external program or disk cache.

use std::io::{self, Read};
use std::time::Duration;

use crate::{Error, Structure, cif_input, input::MAX_INPUT_BYTES};

/// Download an RCSB entry's mmCIF coordinates and return an owned structure.
///
/// Accepts case-insensitive four-character IDs (such as `1lmb`) and extended
/// IDs (such as `pdb_00001lmb`). Uses verified HTTPS, a 60-second total timeout,
/// no redirects, and a 1 GiB response limit. Coordinates are not written to disk.
///
/// # Errors
/// Returns `Error::Selection` for an invalid accession, or `Error::Download`
/// retaining the cause of an HTTP, TLS, timeout, read, size, or parsing failure.
pub fn load_pdb(pdb_id: &str) -> Result<Structure, Error> {
    let id = normalize_id(pdb_id)?;
    // ASVS 2.2.1, 12.3.1-2: validated ID, fixed HTTPS host, and verified TLS.
    let url = format!("https://files.rcsb.org/download/{id}.cif");
    let config = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(60)))
        .build();
    let mut response = ureq::Agent::new_with_config(config)
        .get(&url)
        .call()
        .map_err(|source| download_error(&id, source))?;
    if !response.status().is_success() {
        return Err(download_error(
            &id,
            io::Error::other(format!("HTTP {}", response.status())),
        ));
    }
    let text = read_body(response.body_mut().as_reader(), MAX_INPUT_BYTES)
        .map_err(|source| download_error(&id, source))?;
    cif_input::parse_cif(text.trim_start_matches('\u{feff}'))
        .map_err(|source| download_error(&id, source))
}

fn normalize_id(id: &str) -> Result<String, Error> {
    let legacy = id.len() == 4
        && matches!(id.as_bytes()[0], b'1'..=b'9')
        && id.bytes().all(|byte| byte.is_ascii_alphanumeric());
    let extended = id.len() == 12
        && id
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("pdb_"))
        && id.as_bytes()[4..].iter().all(u8::is_ascii_alphanumeric);
    if !legacy && !extended {
        return Err(Error::Selection(
            "PDB ID must be four characters starting with 1-9 (for example 1lmb), or pdb_ followed by eight letters/digits".into(),
        ));
    }
    Ok(id.to_ascii_lowercase())
}

fn read_body(reader: impl Read, limit: u64) -> io::Result<String> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "RCSB response is empty or exceeds the structure input size limit",
        ));
    }
    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn download_error(id: &str, source: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::Download {
        pdb_id: id.into(),
        source: Box::new(source),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accession_validation_prevents_url_and_path_input() {
        assert_eq!(normalize_id("1LMB").unwrap(), "1lmb");
        assert_eq!(normalize_id("PDB_00001LMB").unwrap(), "pdb_00001lmb");
        assert_eq!(normalize_id("pdb_1000abcd").unwrap(), "pdb_1000abcd");
        for id in [
            "",
            "1lm",
            "1lmb.cif",
            "../x",
            "https://x",
            "1lm?",
            "0lmb",
            "pdb_0000../x",
        ] {
            assert!(matches!(load_pdb(id), Err(Error::Selection(_))));
        }
    }

    #[test]
    fn response_reader_enforces_size_and_utf8_before_parsing() {
        assert_eq!(read_body(&b"data_test"[..], 9).unwrap(), "data_test");
        assert!(read_body(&b"data_test"[..], 8).is_err());
        assert!(read_body(&b""[..], 8).is_err());
        assert!(read_body(&b"\xff"[..], 8).is_err());
    }
}
