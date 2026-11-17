//! Album folders: grouping keys and the unit of work handed to the database.
//!
//! A folder is an album *candidate*, not a canonical album. Disc subfolders collapse into
//! their parent album, and files dropped loose in the scan root are each their own candidate.

use std::path::{Path, PathBuf};

use crate::metadata::fallback;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumKey {
    /// Folder whose path becomes `album.edition_key`.
    pub folder: PathBuf,
    /// Disc number inherited from a `Disc 1` / `Album (Disc 2)` folder.
    pub disc: Option<u32>,
    /// Album title embedded in a disc folder name, if any.
    pub name_override: Option<String>,
}

/// Resolve the album folder for a file. `root` is the scan root, used to detect loose files.
pub fn album_key(path: &Path, root: &Path) -> AlbumKey {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    // Loose files in the root are not an album: keep each one separate.
    if parent == root {
        return AlbumKey {
            folder: path.to_path_buf(),
            disc: None,
            name_override: None,
        };
    }

    let parent_name = parent
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if let Some((album, disc)) = fallback::disc_folder(&parent_name) {
        if album.is_some() {
            // "Album (Disc 2)": this folder is the album folder and also names the disc.
            return AlbumKey {
                folder: parent.to_path_buf(),
                disc: Some(disc),
                name_override: album,
            };
        }
        // "Disc 2": the album is the parent of this folder.
        return AlbumKey {
            folder: parent.parent().unwrap_or(parent).to_path_buf(),
            disc: Some(disc),
            name_override: None,
        };
    }

    AlbumKey {
        folder: parent.to_path_buf(),
        disc: None,
        name_override: None,
    }
}

#[derive(Debug)]
pub struct Candidate {
    pub key: AlbumKey,
    pub tracks: Vec<crate::metadata::AudioMetadata>,
    /// `Some(bool)` from a full scan; `None` from a single-file watcher event, meaning
    /// "leave the album's existing completeness unchanged".
    pub complete: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::album_key;
    use std::path::Path;

    #[test]
    fn ordinary_folder_is_its_own_album() {
        let key = album_key(
            Path::new("/music/宇多田ヒカル - BADモード/01 - BADモード.flac"),
            Path::new("/music"),
        );
        assert_eq!(key.folder, Path::new("/music/宇多田ヒカル - BADモード"));
        assert_eq!(key.disc, None);
        assert_eq!(key.name_override, None);
    }

    #[test]
    fn bare_disc_folder_collapses_to_parent() {
        let key = album_key(
            Path::new("/music/Album/Disc 2/03 - track.flac"),
            Path::new("/music"),
        );
        assert_eq!(key.folder, Path::new("/music/Album"));
        assert_eq!(key.disc, Some(2));
        assert_eq!(key.name_override, None);
    }

    #[test]
    fn disc_suffix_folder_keeps_album_name() {
        let key = album_key(
            Path::new("/music/Artist/Album (Disc 1)/01.flac"),
            Path::new("/music"),
        );
        assert_eq!(key.folder, Path::new("/music/Artist/Album (Disc 1)"));
        assert_eq!(key.disc, Some(1));
        assert_eq!(key.name_override.as_deref(), Some("Album"));
    }

    #[test]
    fn loose_root_files_are_separate_candidates() {
        let a = album_key(Path::new("/music/01.flac"), Path::new("/music"));
        let b = album_key(Path::new("/music/02.flac"), Path::new("/music"));
        assert_ne!(a.folder, b.folder);
        assert_eq!(a.folder, Path::new("/music/01.flac"));
    }
}
