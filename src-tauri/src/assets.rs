use std::io::{BufReader, Read, Seek, Write};
use std::path::{Component, Path};
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine};
use minisign_verify::{PublicKey, Signature};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;

use crate::net;

const RELEASE_API: &str =
    "https://api.github.com/repos/sattatrapy/sattatrackerUpdate/releases/tags/Assets";
const ARCHIVE_URL: &str =
    "https://github.com/sattatrapy/sattatrackerUpdate/releases/download/Assets/assets.zip";
const SIGNATURE_URL: &str =
    "https://github.com/sattatrapy/sattatrackerUpdate/releases/download/Assets/assets.zip.sig";
const PUBLIC_KEY_B64: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEEzRjg3OUExOUFFQjczMjEKUldRaGMrdWFvWG40bzJ3NXhaWkZ1aXVDRFlUeTM1VTdxUnlzdFZCSWtaU0k1YXBESHV6TlNHWnEK";

const MAX_API_BYTES: usize = 1024 * 1024;
const MAX_SIGNATURE_BYTES: usize = 4096;
const MAX_ARCHIVE_BYTES: u64 = 600 * 1024 * 1024;
const MAX_ENTRIES: usize = 20_000;
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;
const ALLOWED_DIRS: [&str; 2] = ["icons", "piganim"];
const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AssetUpdateInfo {
    pub has_update: bool,
    pub remote_version: String,
    pub local_version: String,
}

fn valid_version(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 64
        && v.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '.' | '_' | '-'))
}

fn public_key(b64: &str) -> Result<PublicKey, String> {
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|_| "invalid public key".to_string())?;
    let text = String::from_utf8(bytes).map_err(|_| "invalid public key".to_string())?;
    PublicKey::decode(&text).map_err(|_| "invalid public key".to_string())
}

fn signature(b64: &str) -> Result<Signature, String> {
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|_| "invalid signature".to_string())?;
    let text = String::from_utf8(bytes).map_err(|_| "invalid signature".to_string())?;
    Signature::decode(&text).map_err(|_| "invalid signature".to_string())
}

fn is_supported_image(buf: &[u8]) -> bool {
    let png = buf.len() >= PNG_MAGIC.len() && buf[..PNG_MAGIC.len()] == PNG_MAGIC;
    let webp = buf.len() >= 12 && &buf[..4] == b"RIFF" && &buf[8..12] == b"WEBP";
    png || webp
}

fn safe_component(part: &str) -> bool {
    !part.is_empty()
        && !part.starts_with('.')
        && part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

pub fn extract_safely<R: Read + Seek>(reader: R, dest: &Path) -> Result<usize, String> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
    if archive.len() > MAX_ENTRIES {
        return Err("archive has too many entries".to_string());
    }

    let mut total: u64 = 0;
    let mut written = 0usize;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }

        let rel = entry
            .enclosed_name()
            .ok_or_else(|| "unsafe path in archive".to_string())?;
        let mut parts: Vec<String> = Vec::new();
        for comp in rel.components() {
            match comp {
                Component::Normal(s) => {
                    let s = s
                        .to_str()
                        .ok_or_else(|| "unsafe path in archive".to_string())?;
                    if !safe_component(s) {
                        return Err("unsafe path in archive".to_string());
                    }
                    parts.push(s.to_string());
                }
                _ => return Err("unsafe path in archive".to_string()),
            }
        }
        if parts.len() < 2 || parts.len() > 3 || !ALLOWED_DIRS.contains(&parts[0].as_str()) {
            return Err("unexpected path in archive".to_string());
        }
        if !parts[parts.len() - 1].ends_with(".png") {
            return Err("unexpected file type in archive".to_string());
        }
        if entry.size() > MAX_FILE_BYTES {
            return Err("file too large in archive".to_string());
        }

        let mut buf = Vec::with_capacity(entry.size() as usize);
        (&mut entry)
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        if buf.len() as u64 > MAX_FILE_BYTES {
            return Err("file too large in archive".to_string());
        }
        if !is_supported_image(&buf) {
            return Err("invalid image in archive".to_string());
        }
        total += buf.len() as u64;
        if total > MAX_TOTAL_BYTES {
            return Err("archive too large".to_string());
        }

        let mut target = dest.to_path_buf();
        for p in &parts {
            target.push(p);
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|e| e.to_string())?;
        out.write_all(&buf).map_err(|e| e.to_string())?;
        written += 1;
    }

    if written == 0 {
        return Err("archive is empty".to_string());
    }
    Ok(written)
}

#[tauri::command]
pub async fn check_assets_update(app: AppHandle) -> Result<AssetUpdateInfo, String> {
    let local_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let version_file = local_dir.join("assets_version.json");

    let local_version = match tokio::fs::read_to_string(&version_file).await {
        Ok(v) if valid_version(v.trim()) => v.trim().to_string(),
        _ => "0.0.0".to_string(),
    };

    let client = net::client(Duration::from_secs(10))?;
    let res = client
        .get(RELEASE_API)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Ok(AssetUpdateInfo {
            has_update: local_version == "0.0.0",
            remote_version: local_version.clone(),
            local_version,
        });
    }

    let body = net::read_limited(res, MAX_API_BYTES).await?;
    let json: serde_json::Value = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    let remote_version = json["published_at"]
        .as_str()
        .filter(|v| valid_version(v))
        .unwrap_or("unknown")
        .to_string();

    Ok(AssetUpdateInfo {
        has_update: remote_version != "unknown"
            && (local_version != remote_version || local_version == "0.0.0"),
        remote_version,
        local_version,
    })
}

#[tauri::command]
pub async fn download_assets(
    app: AppHandle,
    remote_version: String,
    window: tauri::Window,
) -> Result<(), String> {
    if !valid_version(&remote_version) {
        return Err("invalid version".to_string());
    }

    let local_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    tokio::fs::create_dir_all(&local_dir)
        .await
        .map_err(|e| e.to_string())?;

    let key = public_key(PUBLIC_KEY_B64)?;

    let sig_client = net::client(Duration::from_secs(20))?;
    let sig_res = sig_client
        .get(SIGNATURE_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !sig_res.status().is_success() {
        return Err("assets signature is not available".to_string());
    }
    let sig_bytes = net::read_limited(sig_res, MAX_SIGNATURE_BYTES).await?;
    let sig_text = String::from_utf8(sig_bytes).map_err(|_| "invalid signature".to_string())?;
    let sig = signature(&sig_text)?;
    let mut verifier = key
        .verify_stream(&sig)
        .map_err(|_| "invalid signature".to_string())?;

    let client = net::client(Duration::from_secs(1800))?;
    let mut res = client
        .get(ARCHIVE_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("Download failed with status: {}", res.status()));
    }
    let expected = res.content_length().unwrap_or(0);
    if expected > MAX_ARCHIVE_BYTES {
        return Err("archive too large".to_string());
    }

    let zip_path = local_dir.join("assets.zip.part");
    let staging = local_dir.join("public_assets.new");
    let target = local_dir.join("public_assets");
    let previous = local_dir.join("public_assets.old");

    let result = async {
        let mut file = tokio::fs::File::create(&zip_path)
            .await
            .map_err(|e| e.to_string())?;
        let mut downloaded: u64 = 0;
        let progress_total = if expected > 0 { expected } else { 150_000_000 };
        let mut last_emit = std::time::Instant::now();

        while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
            downloaded += chunk.len() as u64;
            if downloaded > MAX_ARCHIVE_BYTES {
                return Err("archive too large".to_string());
            }
            verifier.update(&chunk);
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;

            if last_emit.elapsed().as_millis() > 500 {
                let progress = (downloaded as f64 / progress_total as f64 * 100.0).min(99.0);
                let _ = window.emit("asset-download-progress", progress);
                last_emit = std::time::Instant::now();
            }
        }
        file.flush().await.map_err(|e| e.to_string())?;
        drop(file);

        verifier
            .finalize()
            .map_err(|_| "assets signature check failed".to_string())?;

        let _ = window.emit("asset-download-progress", 100.0);
        let _ = window.emit("asset-extract-start", ());

        let zip_for_extract = zip_path.clone();
        let staging_for_extract = staging.clone();
        tokio::task::spawn_blocking(move || {
            if staging_for_extract.exists() {
                std::fs::remove_dir_all(&staging_for_extract).map_err(|e| e.to_string())?;
            }
            std::fs::create_dir_all(&staging_for_extract).map_err(|e| e.to_string())?;
            let file = std::fs::File::open(&zip_for_extract).map_err(|e| e.to_string())?;
            extract_safely(BufReader::new(file), &staging_for_extract).map(|_| ())
        })
        .await
        .map_err(|e| e.to_string())??;

        if previous.exists() {
            std::fs::remove_dir_all(&previous).map_err(|e| e.to_string())?;
        }
        if target.exists() {
            std::fs::rename(&target, &previous).map_err(|e| e.to_string())?;
        }
        if let Err(e) = std::fs::rename(&staging, &target) {
            if previous.exists() {
                let _ = std::fs::rename(&previous, &target);
            }
            return Err(e.to_string());
        }
        let _ = std::fs::remove_dir_all(&previous);

        std::fs::write(local_dir.join("assets_version.json"), &remote_version)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;

    let _ = std::fs::remove_file(&zip_path);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
        return result;
    }

    let _ = window.emit("asset-extract-done", ());
    Ok(())
}

#[tauri::command]
pub fn get_assets_path(app: AppHandle) -> Result<String, String> {
    let local_dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    let target_dir = local_dir.join("public_assets");
    Ok(target_dir.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    const TEST_PUB: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDYwNUIzNDc1RTA5ODQ0MzgKUldRNFJKamdkVFJiWUZoMGtHbDNtMFczMksrWUErblVsS25MaFp3M0YwUVVxRkVZY3ladjZjM1UK";
    const TEST_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRNFJKamdkVFJiWU5NNVl5MGpBRitxSEQ2Y2NXOEZZT3NRcHhRWFRtU1Vrb0VkZEQxcVlaZ1lINkYrdzBCMldqNllIL3NLZHg3Mk94NG15V0l4SXkwQ1Vac05sMXdpWUFVPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkyNzkyMTk5CWZpbGU6cGF5bG9hZC5iaW4KNEs4ZlNtc0xsbW9FOGhnUHhHMTB3R1lWbVR1N2VoRFA0NGVNcGZPaHliOUozMXQraVVrbUxYWFRaMm5VZURXdzcybVMwRDZVT0Z2VTNMbE41V05jQVE9PQo=";
    const PAYLOAD: &[u8] = b"TLI assets test payload";

    fn stream_verify(key_b64: &str, sig_b64: &str, data: &[u8]) -> Result<(), String> {
        let key = public_key(key_b64)?;
        let sig = signature(sig_b64)?;
        let mut v = key.verify_stream(&sig).map_err(|e| e.to_string())?;
        for chunk in data.chunks(7) {
            v.update(chunk);
        }
        v.finalize().map_err(|e| e.to_string())
    }

    fn png(extra: usize) -> Vec<u8> {
        let mut v = PNG_MAGIC.to_vec();
        v.extend(std::iter::repeat(0u8).take(extra));
        v
    }

    fn make_zip(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default();
        for (name, data) in entries {
            w.start_file(*name, opts).unwrap();
            w.write_all(data).unwrap();
        }
        w.finish().unwrap().into_inner()
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("tli_assets_test_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn extract(name: &str, entries: &[(&str, Vec<u8>)]) -> (Result<usize, String>, PathBuf) {
        let dir = scratch(name);
        let bytes = make_zip(entries);
        (extract_safely(Cursor::new(bytes), &dir), dir)
    }

    #[test]
    fn valid_signature_is_accepted() {
        assert!(stream_verify(TEST_PUB, TEST_SIG, PAYLOAD).is_ok());
    }

    #[test]
    fn tampered_payload_is_rejected() {
        assert!(stream_verify(TEST_PUB, TEST_SIG, b"TLI assets test payloaD").is_err());
        assert!(stream_verify(TEST_PUB, TEST_SIG, b"").is_err());
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let mut bytes = STANDARD.decode(TEST_SIG).unwrap();
        let idx = bytes.len() - 20;
        bytes[idx] ^= 0x01;
        let tampered = STANDARD.encode(bytes);
        assert!(stream_verify(TEST_PUB, &tampered, PAYLOAD).is_err());
    }

    #[test]
    fn signature_from_another_key_is_rejected() {
        assert!(stream_verify(PUBLIC_KEY_B64, TEST_SIG, PAYLOAD).is_err());
    }

    #[test]
    fn garbage_key_and_signature_are_rejected() {
        assert!(public_key("not base64 !!").is_err());
        assert!(signature("not base64 !!").is_err());
        assert!(signature(&STANDARD.encode("two\nlines")).is_err());
    }

    #[test]
    fn production_public_key_parses() {
        assert!(public_key(PUBLIC_KEY_B64).is_ok());
    }

    fn webp() -> Vec<u8> {
        let mut v = b"RIFF".to_vec();
        v.extend_from_slice(&[4, 0, 0, 0]);
        v.extend_from_slice(b"WEBPVP8 ");
        v
    }

    #[test]
    fn webp_content_in_png_named_files_is_accepted() {
        let (res, dir) = extract("webp", &[("icons/100.png", webp())]);
        assert_eq!(res.unwrap(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn other_image_formats_and_executables_are_rejected() {
        let mut riff_wave = b"RIFF".to_vec();
        riff_wave.extend_from_slice(&[4, 0, 0, 0]);
        riff_wave.extend_from_slice(b"WAVEfmt ");
        let cases: Vec<Vec<u8>> = vec![
            vec![0xff, 0xd8, 0xff, 0xe0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![b'G', b'I', b'F', b'8', b'9', b'a', 0, 0, 0, 0, 0, 0],
            vec![b'M', b'Z', 0x90, 0, 3, 0, 0, 0, 4, 0, 0, 0],
            riff_wave,
            b"RIFF".to_vec(),
        ];
        for data in cases {
            let (res, dir) = extract("other", &[("icons/a.png", data)]);
            assert!(res.is_err());
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn good_archive_is_extracted() {
        let (res, dir) = extract(
            "good",
            &[
                ("icons/100.png", png(10)),
                ("piganim/coin/coin_0.png", png(5)),
                ("piganim/idle_1.png", png(3)),
            ],
        );
        assert_eq!(res.unwrap(), 3);
        assert!(dir.join("icons").join("100.png").exists());
        assert!(dir.join("piganim").join("coin").join("coin_0.png").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn path_traversal_is_rejected() {
        for name in [
            "../evil.png",
            "icons/../../evil.png",
            "/abs/evil.png",
            "C:/evil.png",
            "icons/.hidden.png",
        ] {
            let (res, dir) = extract("trav", &[(name, png(4))]);
            assert!(res.is_err(), "accepted {}", name);
            assert!(!dir.parent().unwrap().join("evil.png").exists());
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn unexpected_locations_and_types_are_rejected() {
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("other/a.png", png(4)),
            ("a.png", png(4)),
            ("icons/a.exe", png(4)),
            ("icons/a.png.exe", png(4)),
            ("icons/a.PNG", png(4)),
            ("icons/a/b/c.png", png(4)),
            ("icons/a b.png", png(4)),
            ("icons/a.png", b"MZ not an image".to_vec()),
            ("icons/a.png", Vec::new()),
        ];
        for (name, data) in cases {
            let (res, dir) = extract("types", &[(name, data)]);
            assert!(res.is_err(), "accepted {}", name);
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn oversized_file_is_rejected() {
        let (res, dir) = extract("big", &[("icons/huge.png", png(MAX_FILE_BYTES as usize))]);
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn empty_archive_is_rejected() {
        let (res, dir) = extract("empty", &[]);
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn one_bad_entry_fails_the_whole_extraction() {
        let (res, dir) = extract(
            "mixed",
            &[("icons/1.png", png(4)), ("icons/evil.exe", png(4))],
        );
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn version_strings_are_validated() {
        assert!(valid_version("2026-10-19T12:42:19Z"));
        assert!(!valid_version(""));
        assert!(!valid_version("../x"));
        assert!(!valid_version("a b"));
        assert!(!valid_version(&"1".repeat(65)));
    }
}
