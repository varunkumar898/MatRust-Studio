//! Container parsing, zip-slip protection, zip-bomb mitigation,
//! and metadata/manifest extraction for MATLAB App Designer (.mlapp)
//! and App Installer (.mlappinstall) files.

use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};
use std::path::{Component as PathComponent, Path};
use thiserror::Error;

/// Strict container limits to prevent Denial of Service (DoS) attacks.
#[derive(Debug, Clone, Copy)]
pub struct ContainerLimits {
    pub max_file_count: usize,
    pub max_uncompressed_bytes: u64,
    pub max_compression_ratio: f64,
}

impl Default for ContainerLimits {
    fn default() -> Self {
        Self {
            max_file_count: 2048,
            max_uncompressed_bytes: 256 * 1024 * 1024, // 256 MiB
            max_compression_ratio: 100.0,
        }
    }
}

/// Errors occurring while opening, validating, or extracting an app package.
#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP decompression error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Security violation: path traversal / zip-slip detected in entry '{0}'")]
    ZipSlip(String),

    #[error("Security violation: maximum file count exceeded ({0} > {1})")]
    TooManyFiles(usize, usize),

    #[error("Security violation: maximum uncompressed size exceeded ({0} bytes > {1} bytes)")]
    TooLarge(u64, u64),

    #[error("Security violation: compression ratio exceeded threshold ({0:.1}:1 > {1:.1}:1)")]
    ZipBomb(f64, f64),

    #[error("Security violation: invalid entry path '{0}' ({1})")]
    InvalidPath(String, String),

    #[error("Security violation: duplicate entry name '{0}'")]
    DuplicateEntry(String),

    #[error("Security violation: symlink or unsupported entry '{0}'")]
    SymlinkNotAllowed(String),

    #[error("Invalid package format: neither .mlapp nor .mlappinstall content found")]
    InvalidFormat,

    #[error("Required entry not found: {0}")]
    MissingEntry(String),
}

/// The recognized role of an entry inside an .mlapp archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryRole {
    Code,
    UiLayout,
    Metadata,
    Thumbnail,
    BinaryData,
    Resource,
    Unknown(String),
}

/// Information about a single entry within the package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEntryInfo {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub role: EntryRole,
}

/// Classification of package type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    Mlapp,
    MlappInstall,
}

/// Metadata extracted from .mlappinstall or .mlapp packages.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AppManifest {
    pub app_name: String,
    pub version: String,
    pub author: String,
    pub summary: String,
    pub description: String,
    pub required_toolboxes: Vec<String>,
    pub screenshot_entry: Option<String>,
    pub icon_entry: Option<String>,
    pub inner_mlapp_entry: Option<String>,
    pub detected_matlab_release: Option<String>,
    pub warnings: Vec<String>,
}

/// Full diagnostic inspection report serialized for CLI inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageReport {
    pub package_type: PackageType,
    pub total_entries: usize,
    pub total_compressed_size: u64,
    pub total_uncompressed_size: u64,
    pub manifest: AppManifest,
    pub entries: Vec<PackageEntryInfo>,
    pub unsupported_features: Vec<String>,
}

/// Reader and inspector for `.mlapp` and `.mlappinstall` containers.
pub struct AppContainer<R: Read + Seek> {
    archive: zip::ZipArchive<R>,
    limits: ContainerLimits,
    entries: Vec<PackageEntryInfo>,
    package_type: PackageType,
    pub total_decompressed_bytes: u64,
}

impl<R: Read + Seek> std::fmt::Debug for AppContainer<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContainer")
            .field("entries", &self.entries)
            .field("package_type", &self.package_type)
            .field("total_decompressed_bytes", &self.total_decompressed_bytes)
            .finish()
    }
}

impl<R: Read + Seek> AppContainer<R> {
    /// Opens a container with default safety limits.
    pub fn open(reader: R) -> Result<Self, ContainerError> {
        Self::open_with_limits(reader, ContainerLimits::default())
    }

    /// Opens a container enforcing explicit safety limits against zip-slip & zip-bombs.
    pub fn open_with_limits(reader: R, limits: ContainerLimits) -> Result<Self, ContainerError> {
        let mut archive = zip::ZipArchive::new(reader)?;
        let file_count = archive.len();

        if file_count > limits.max_file_count {
            return Err(ContainerError::TooManyFiles(
                file_count,
                limits.max_file_count,
            ));
        }

        let mut total_uncompressed: u64 = 0;
        let mut total_compressed: u64 = 0;
        let mut entries = Vec::with_capacity(file_count);
        let mut seen_names = std::collections::HashSet::new();
        let mut has_mlapp_code = false;
        let mut has_manifest = false;

        for i in 0..file_count {
            let file = archive.by_index(i)?;
            let raw_name = file.name().to_string();

            // Zip-Slip, UNC, drive letter, and reserved device sanitization
            sanitize_path(&raw_name)?;

            // Reject duplicate entry names (case-insensitive and normalized)
            let norm_lower = raw_name.replace('\\', "/").to_lowercase();
            if !seen_names.insert(norm_lower) {
                return Err(ContainerError::DuplicateEntry(raw_name));
            }

            // Reject symlink entries
            if file.is_symlink()
                || file
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170000 == 0o120000)
            {
                return Err(ContainerError::SymlinkNotAllowed(raw_name));
            }

            let c_size = file.compressed_size();
            let u_size = file.size();

            total_compressed = total_compressed.saturating_add(c_size);
            total_uncompressed = total_uncompressed.saturating_add(u_size);

            if total_uncompressed > limits.max_uncompressed_bytes {
                return Err(ContainerError::TooLarge(
                    total_uncompressed,
                    limits.max_uncompressed_bytes,
                ));
            }

            if c_size > 0 {
                let ratio = u_size as f64 / c_size as f64;
                if ratio > limits.max_compression_ratio {
                    return Err(ContainerError::ZipBomb(ratio, limits.max_compression_ratio));
                }
            }

            let role = classify_role(&raw_name);
            if role == EntryRole::Code || role == EntryRole::UiLayout {
                has_mlapp_code = true;
            }
            if raw_name.contains("manifest") || raw_name.ends_with(".mlapp") {
                has_manifest = true;
            }

            entries.push(PackageEntryInfo {
                name: raw_name,
                compressed_size: c_size,
                uncompressed_size: u_size,
                role,
            });
        }

        // Determine package type by content
        let package_type = if entries.iter().any(|e| e.name.ends_with(".mlapp")) {
            PackageType::MlappInstall
        } else if has_mlapp_code {
            PackageType::Mlapp
        } else if has_manifest {
            PackageType::MlappInstall
        } else {
            PackageType::Mlapp
        };

        Ok(Self {
            archive,
            limits,
            entries,
            package_type,
            total_decompressed_bytes: 0,
        })
    }

    /// Read raw bytes of an entry by exact name with zip-slip verification and true decompressed byte limits.
    pub fn read_entry(&mut self, name: &str) -> Result<Vec<u8>, ContainerError> {
        sanitize_path(name)?;
        let mut file = self.archive.by_name(name)?;
        let remaining_budget = self
            .limits
            .max_uncompressed_bytes
            .saturating_sub(self.total_decompressed_bytes);
        if remaining_budget == 0 {
            return Err(ContainerError::TooLarge(
                self.total_decompressed_bytes,
                self.limits.max_uncompressed_bytes,
            ));
        }

        // Limit reader by remaining_budget + 1 to avoid reading unbounded data
        let mut limited = (&mut file).take(remaining_budget + 1);
        let mut buffer = Vec::new();
        limited.read_to_end(&mut buffer)?;

        if buffer.len() as u64 > remaining_budget {
            let attempted_total = self
                .total_decompressed_bytes
                .saturating_add(buffer.len() as u64);
            return Err(ContainerError::TooLarge(
                attempted_total,
                self.limits.max_uncompressed_bytes,
            ));
        }

        let c_size = file.compressed_size();
        if c_size > 0 {
            let actual_ratio = buffer.len() as f64 / c_size as f64;
            if actual_ratio > self.limits.max_compression_ratio {
                return Err(ContainerError::ZipBomb(
                    actual_ratio,
                    self.limits.max_compression_ratio,
                ));
            }
        }

        self.total_decompressed_bytes = self
            .total_decompressed_bytes
            .saturating_add(buffer.len() as u64);
        Ok(buffer)
    }

    /// Safely extracts all entries into the specified destination directory.
    pub fn extract_all(&mut self, out_dir: &Path) -> Result<(), ContainerError> {
        std::fs::create_dir_all(out_dir)?;

        for i in 0..self.entries.len() {
            let entry_name = self.entries[i].name.clone();
            sanitize_path(&entry_name)?;

            let dest_path = out_dir.join(Path::new(&entry_name));
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let data = self.read_entry(&entry_name)?;
            std::fs::write(&dest_path, data)?;
        }
        Ok(())
    }

    /// Read an entry as UTF-8 string.
    pub fn read_entry_to_string(&mut self, name: &str) -> Result<String, ContainerError> {
        let bytes = self.read_entry(name)?;
        String::from_utf8(bytes).map_err(|e| {
            ContainerError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })
    }

    /// Safely opens an inner `.mlapp` archive with inherited remaining budget to mitigate nested zip bombs.
    pub fn open_inner_mlapp(
        &mut self,
        inner_name: &str,
    ) -> Result<AppContainer<std::io::Cursor<Vec<u8>>>, ContainerError> {
        let bytes = self.read_entry(inner_name)?;
        let remaining_budget = self
            .limits
            .max_uncompressed_bytes
            .saturating_sub(self.total_decompressed_bytes);
        let inner_limits = ContainerLimits {
            max_file_count: self.limits.max_file_count,
            max_uncompressed_bytes: remaining_budget,
            max_compression_ratio: self.limits.max_compression_ratio,
        };
        AppContainer::open_with_limits(std::io::Cursor::new(bytes), inner_limits)
    }

    /// Inspect the container and generate a complete manifest and report.
    pub fn inspect(&mut self) -> Result<PackageReport, ContainerError> {
        let mut manifest = AppManifest::default();
        let mut unsupported_features = Vec::new();

        // 1. Detect binary .mat data or unsupported parts
        for entry in &self.entries {
            if entry.role == EntryRole::BinaryData {
                let warn = format!(
                    "Binary data found in entry '{}': unsupported-binary",
                    entry.name
                );
                manifest.warnings.push(warn.clone());
                unsupported_features.push(warn);
            }
        }

        // 2. Discover manifest / metadata depending on package type
        if self.package_type == PackageType::MlappInstall {
            self.extract_mlappinstall_manifest(&mut manifest)?;
        } else {
            self.extract_mlapp_manifest(&mut manifest)?;
        }

        let total_c: u64 = self.entries.iter().map(|e| e.compressed_size).sum();
        let total_u: u64 = self.entries.iter().map(|e| e.uncompressed_size).sum();

        Ok(PackageReport {
            package_type: self.package_type,
            total_entries: self.entries.len(),
            total_compressed_size: total_c,
            total_uncompressed_size: total_u,
            manifest,
            entries: self.entries.clone(),
            unsupported_features,
        })
    }

    fn extract_mlappinstall_manifest(
        &mut self,
        manifest: &mut AppManifest,
    ) -> Result<(), ContainerError> {
        // Locate manifest XML across variants: metadata/manifest.xml, manifest.xml, appinfo.xml
        let manifest_candidate = self.entries.iter().map(|e| e.name.clone()).find(|n| {
            let lower = n.to_lowercase();
            lower.ends_with("manifest.xml")
                || lower.ends_with("appinfo.xml")
                || lower.ends_with("appdata.xml")
        });

        if let Some(manifest_file) = manifest_candidate {
            if let Ok(xml_str) = self.read_entry_to_string(&manifest_file) {
                parse_manifest_xml(&xml_str, manifest);
            }
        }

        // Locate inner .mlapp
        if let Some(inner) = self.entries.iter().find(|e| e.name.ends_with(".mlapp")) {
            manifest.inner_mlapp_entry = Some(inner.name.clone());
            if manifest.app_name.is_empty() {
                let name = Path::new(&inner.name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("App");
                manifest.app_name = name.to_string();
            }
        }

        // Locate icon or screenshot
        for entry in &self.entries {
            let lower = entry.name.to_lowercase();
            if lower.contains("screenshot") || lower.contains("preview") {
                manifest.screenshot_entry = Some(entry.name.clone());
            } else if lower.contains("icon") {
                manifest.icon_entry = Some(entry.name.clone());
            }
        }

        Ok(())
    }

    fn extract_mlapp_manifest(&mut self, manifest: &mut AppManifest) -> Result<(), ContainerError> {
        // Find core properties XML or app metadata
        let mut core_props = None;
        for entry in &self.entries {
            let lower = entry.name.to_lowercase();
            if lower == "metadata/coreproperties.xml" || lower.ends_with("core.xml") {
                core_props = Some(entry.name.clone());
            } else if lower.ends_with("screenshot.png") || lower.ends_with("thumbnail.png") {
                manifest.screenshot_entry = Some(entry.name.clone());
            } else if lower.ends_with("icon.png") {
                manifest.icon_entry = Some(entry.name.clone());
            }
        }

        if let Some(prop_file) = core_props {
            if let Ok(xml_str) = self.read_entry_to_string(&prop_file) {
                parse_core_properties_xml(&xml_str, manifest);
            }
        }

        // Check appdesigner/app.xml or matlab/document.xml to detect release clues
        if let Ok(app_xml) = self.read_entry_to_string("appdesigner/app.xml") {
            manifest.detected_matlab_release = detect_release_from_text(&app_xml);
        }
        if manifest.detected_matlab_release.is_none() {
            if let Ok(doc_xml) = self.read_entry_to_string("matlab/document.xml") {
                manifest.detected_matlab_release = detect_release_from_text(&doc_xml);
            }
        }

        if manifest.app_name.is_empty() {
            manifest.app_name = "UntitledApp".to_string();
        }

        Ok(())
    }

    pub fn limits(&self) -> ContainerLimits {
        self.limits
    }

    pub fn entries(&self) -> &[PackageEntryInfo] {
        &self.entries
    }

    pub fn package_type(&self) -> PackageType {
        self.package_type
    }
}

/// Classifies an entry's role based on MATLAB container architecture.
pub fn classify_role(path_str: &str) -> EntryRole {
    let normalized = path_str
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_lowercase();

    if normalized == "matlab/document.xml" || normalized.ends_with("/document.xml") {
        EntryRole::Code
    } else if normalized == "appdesigner/app.xml" || normalized.ends_with("/app.xml") {
        EntryRole::UiLayout
    } else if normalized.ends_with(".png")
        || normalized.ends_with(".jpg")
        || normalized.ends_with(".jpeg")
        || normalized.ends_with(".ico")
    {
        if normalized.contains("thumbnail")
            || normalized.contains("screenshot")
            || normalized.contains("preview")
            || normalized.contains("icon")
        {
            EntryRole::Thumbnail
        } else {
            EntryRole::Resource
        }
    } else if normalized.starts_with("metadata/")
        || normalized == "[content_types].xml"
        || normalized.ends_with(".xml")
    {
        EntryRole::Metadata
    } else if normalized.ends_with(".mat") || normalized.ends_with(".bin") {
        EntryRole::BinaryData
    } else {
        EntryRole::Unknown(path_str.to_string())
    }
}

/// Strict path sanitization.
/// Rejects Zip-Slip, UNC network paths, drive letters, NUL bytes, and Windows reserved device names.
pub fn sanitize_path(path_str: &str) -> Result<(), ContainerError> {
    if path_str.contains('\0') {
        return Err(ContainerError::InvalidPath(
            path_str.to_string(),
            "contains NUL byte".to_string(),
        ));
    }
    if path_str.starts_with(r"\\") || path_str.starts_with("//") {
        return Err(ContainerError::InvalidPath(
            path_str.to_string(),
            "UNC network path not allowed".to_string(),
        ));
    }
    if path_str.len() >= 2
        && path_str.as_bytes()[1] == b':'
        && path_str.as_bytes()[0].is_ascii_alphabetic()
    {
        return Err(ContainerError::InvalidPath(
            path_str.to_string(),
            "drive letter prefix not allowed".to_string(),
        ));
    }
    if path_str.contains("..") || path_str.starts_with('/') || path_str.starts_with('\\') {
        return Err(ContainerError::ZipSlip(path_str.to_string()));
    }

    // Windows reserved device names: CON, PRN, AUX, NUL, COM1-9, LPT1-9
    for segment in path_str.split(['/', '\\']) {
        let base = segment
            .split('.')
            .next()
            .unwrap_or("")
            .trim()
            .to_uppercase();
        if matches!(
            base.as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        ) {
            return Err(ContainerError::InvalidPath(
                path_str.to_string(),
                format!("reserved device name '{base}' not allowed"),
            ));
        }
    }

    let p = Path::new(path_str);
    for component in p.components() {
        match component {
            PathComponent::ParentDir => return Err(ContainerError::ZipSlip(path_str.to_string())),
            PathComponent::RootDir | PathComponent::Prefix(_) => {
                return Err(ContainerError::ZipSlip(path_str.to_string()))
            }
            _ => {}
        }
    }
    Ok(())
}

/// Parse metadata XML for `.mlappinstall` manifests.
fn parse_manifest_xml(xml: &str, manifest: &mut AppManifest) {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut current_tag = String::new();

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(e) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
            }
            Event::Text(e) => {
                if let Ok(text) = e.unescape() {
                    let val = text.trim().to_string();
                    if !val.is_empty() {
                        match current_tag.as_str() {
                            "name" | "appname" | "title" => {
                                if manifest.app_name.is_empty() {
                                    manifest.app_name = val;
                                }
                            }
                            "version" => manifest.version = val,
                            "author" | "creator" => manifest.author = val,
                            "summary" => manifest.summary = val,
                            "description" => manifest.description = val,
                            "toolbox" | "product" | "requiredtoolbox"
                                if !manifest.required_toolboxes.contains(&val) =>
                            {
                                manifest.required_toolboxes.push(val);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::End(_) => {
                current_tag.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
}

/// Parse core properties XML (OPC format).
fn parse_core_properties_xml(xml: &str, manifest: &mut AppManifest) {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut current_tag = String::new();

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let local = name.split(':').next_back().unwrap_or(&name).to_lowercase();
                current_tag = local;
            }
            Event::Text(e) => {
                if let Ok(text) = e.unescape() {
                    let val = text.trim().to_string();
                    if !val.is_empty() {
                        match current_tag.as_str() {
                            "title" => manifest.app_name = val,
                            "creator" => manifest.author = val,
                            "description" => manifest.description = val,
                            "version" => manifest.version = val,
                            _ => {}
                        }
                    }
                }
            }
            Event::End(_) => {
                current_tag.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
}

/// Extract MATLAB release tag from XML text (e.g. "R2021a", "R2024b").
fn detect_release_from_text(text: &str) -> Option<String> {
    for pattern in [
        "R2018a", "R2018b", "R2019a", "R2019b", "R2020a", "R2020b", "R2021a", "R2021b", "R2022a",
        "R2022b", "R2023a", "R2023b", "R2024a", "R2024b", "R2025a", "R2025b",
    ] {
        if text.contains(pattern) {
            return Some(pattern.to_string());
        }
    }
    None
}
