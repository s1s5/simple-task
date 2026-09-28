use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub fn write(dir: &Path, slug: &str, content: &str) -> Result<()> {
    let path = path_for(dir, slug)?;
    std::fs::create_dir_all(dir)?;
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(Error::SlugExists(slug.to_owned()));
        }
        Err(e) => return Err(Error::Io(e)),
    };
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn read(dir: &Path, slug: &str) -> Result<String> {
    let path = path_for(dir, slug)?;
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(Error::SlugNotFound(slug.to_owned()))
        }
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn list_slugs(dir: &Path) -> Result<Vec<String>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::Io(e)),
    };
    let mut slugs: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .filter_map(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_owned)
        })
        .collect();
    slugs.sort();
    Ok(slugs)
}

fn path_for(dir: &Path, slug: &str) -> Result<PathBuf> {
    let bytes = slug.as_bytes();
    let head = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let valid = (1..=64).contains(&bytes.len())
        && head(bytes[0])
        && bytes[1..]
            .iter()
            .all(|&b| head(b) || matches!(b, b'.' | b'_' | b'-'));
    if !valid {
        return Err(Error::InvalidSlug(slug.to_owned()));
    }
    Ok(dir.join(format!("{slug}.md")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> Result<PathBuf> {
        let path =
            std::env::temp_dir().join(format!("simple-task-store-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[test]
    fn content_roundtrip() -> Result<()> {
        let dir = dir("roundtrip")?;
        write(&dir, "alpha", "alpha の本文")?;
        assert_eq!(read(&dir, "alpha")?, "alpha の本文");
        Ok(())
    }

    #[test]
    fn write_creates_directory() -> Result<()> {
        let dir = dir("create")?.join("nested");
        write(&dir, "alpha", "本文")?;
        assert_eq!(read(&dir, "alpha")?, "本文");
        Ok(())
    }

    #[test]
    fn duplicate_slug_is_rejected() -> Result<()> {
        let dir = dir("duplicate")?;
        write(&dir, "alpha", "first")?;
        assert!(
            matches!(write(&dir, "alpha", "second"), Err(Error::SlugExists(s)) if s == "alpha")
        );
        assert_eq!(read(&dir, "alpha")?, "first");
        Ok(())
    }

    #[test]
    fn missing_slug_is_not_found() -> Result<()> {
        let dir = dir("missing")?;
        assert!(matches!(read(&dir, "ghost"), Err(Error::SlugNotFound(s)) if s == "ghost"));
        Ok(())
    }

    #[test]
    fn list_slugs_is_sorted_and_md_only() -> Result<()> {
        let dir = dir("list")?;
        for slug in ["gamma", "alpha", "beta"] {
            write(&dir, slug, "本文")?;
        }
        std::fs::write(dir.join("notes.txt"), "無視される")?;
        assert_eq!(list_slugs(&dir)?, vec!["alpha", "beta", "gamma"]);
        Ok(())
    }

    #[test]
    fn list_slugs_of_missing_directory_is_empty() -> Result<()> {
        let dir = dir("list-missing")?.join("nope");
        assert!(list_slugs(&dir)?.is_empty());
        Ok(())
    }

    #[test]
    fn traversal_and_invalid_slugs_are_rejected() -> Result<()> {
        let dir = dir("traversal")?;
        for slug in [
            "..",
            "../etc/passwd",
            "a/b",
            "/absolute",
            ".hidden",
            "",
            "日本語",
            "has space",
            "Upper",
            "-leading",
            "_leading",
        ] {
            assert!(
                matches!(write(&dir, slug, "x"), Err(Error::InvalidSlug(s)) if s == slug),
                "{slug:?} should be rejected"
            );
            assert!(matches!(read(&dir, slug), Err(Error::InvalidSlug(s)) if s == slug));
        }
        Ok(())
    }

    #[test]
    fn slug_length_boundary_is_64() -> Result<()> {
        let dir = dir("length")?;
        let max = "a".repeat(64);
        let too_long = "a".repeat(65);
        write(&dir, &max, "本文")?;
        assert_eq!(read(&dir, &max)?, "本文");
        assert!(matches!(
            write(&dir, &too_long, "x"),
            Err(Error::InvalidSlug(_))
        ));
        Ok(())
    }
}
