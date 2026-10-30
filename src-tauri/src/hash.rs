//! The content hash of an original (data contract 3.5): `sha256:` and 64 hexadecimal characters,
//! read in blocks of 4 MB (F018/S01).

use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 4 << 20];
    loop {
        let n = f.read(&mut buf).map_err(|e| format!("{}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!("sha256:{hex}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_hash_of_a_known_text() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("abc.txt");
        std::fs::write(&p, b"abc").unwrap();
        assert_eq!(
            super::sha256_file(&p).unwrap(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(super::sha256_file(&d.path().join("missing")).unwrap_err().contains("missing"));
    }
}
