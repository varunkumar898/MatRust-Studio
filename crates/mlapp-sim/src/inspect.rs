//! SLX archive inspection and compatibility reporting.

use crate::model::{BlockDiagram, BlockType};
use crate::parser::{parse_blockdiagram_xml, ParseError};
use mlapp_container::{ContainerError, ContainerLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, Read, Seek};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum InspectError {
    #[error("Container error: {0}")]
    Container(#[from] ContainerError),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parser error: {0}")]
    Parse(#[from] ParseError),

    #[error("Missing block diagram entry in .slx package: expected simulink/blockdiagram.xml")]
    MissingBlockDiagram,
}

/// Comprehensive compatibility and structure report for an .slx model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlxInspectReport {
    pub model_name: String,
    pub release: String,
    pub version: String,
    pub total_blocks: usize,
    pub total_lines: usize,
    pub block_type_counts: HashMap<String, usize>,
    pub subsystems: Vec<String>,
    pub required_toolboxes: Vec<String>,
    pub supported_blocks: Vec<String>,
    pub unsupported_blocks: Vec<String>,
    pub is_solver_supported: bool,
}

/// Inspects an .slx archive stream with strict security limits.
pub fn inspect_slx<R: Read + Seek>(
    mut reader: R,
    limits: ContainerLimits,
) -> Result<(BlockDiagram, SlxInspectReport), InspectError> {
    // 1. Verify zip boundaries and entry count under limits
    let mut zip = ZipArchive::new(&mut reader)?;

    if zip.len() > limits.max_file_count {
        return Err(InspectError::Container(ContainerError::TooManyFiles(
            zip.len(),
            limits.max_file_count,
        )));
    }

    let mut total_uncompressed: u64 = 0;
    let mut total_compressed: u64 = 0;
    let mut diagram_entry_idx = None;

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        let name = file.name()?.to_string();

        // Check for zip-slip path traversal
        if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
            return Err(InspectError::Container(ContainerError::ZipSlip(name)));
        }

        total_uncompressed += file.size();
        total_compressed += file.compressed_size();

        if total_uncompressed > limits.max_uncompressed_bytes {
            return Err(InspectError::Container(ContainerError::TooLarge(
                total_uncompressed,
                limits.max_uncompressed_bytes,
            )));
        }

        if name == "simulink/blockdiagram.xml" || name == "simulink/systems/system_root.xml" {
            diagram_entry_idx = Some(i);
        }
    }

    // Zip bomb compression ratio check
    if total_compressed > 0 {
        let ratio = total_uncompressed as f64 / total_compressed as f64;
        if ratio > limits.max_compression_ratio {
            return Err(InspectError::Container(ContainerError::ZipBomb(
                ratio,
                limits.max_compression_ratio,
            )));
        }
    }

    let idx = diagram_entry_idx.ok_or(InspectError::MissingBlockDiagram)?;
    let entry_file = zip.by_index(idx)?;
    let buf_reader = BufReader::new(entry_file);
    let diagram = parse_blockdiagram_xml(buf_reader)?;

    // Build compatibility report
    let mut block_counts = HashMap::new();
    let mut supported = Vec::new();
    let mut unsupported = Vec::new();
    let mut subsystems = Vec::new();

    for b in &diagram.blocks {
        let type_name = match &b.block_type {
            BlockType::Unknown(name) => name.clone(),
            other => format!("{other:?}"),
        };
        *block_counts.entry(type_name.clone()).or_insert(0) += 1;

        if b.block_type.is_solver_supported() {
            supported.push(b.name.clone());
        } else {
            unsupported.push(b.name.clone());
        }

        if matches!(b.block_type, BlockType::Subsystem) {
            subsystems.push(b.name.clone());
        }
    }

    let is_solver_supported = unsupported.is_empty();

    let report = SlxInspectReport {
        model_name: diagram.name.clone(),
        release: diagram
            .release
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        version: diagram.version.clone().unwrap_or_else(|| "1.0".to_string()),
        total_blocks: diagram.blocks.len(),
        total_lines: diagram.lines.len(),
        block_type_counts: block_counts,
        subsystems,
        required_toolboxes: diagram.toolboxes.clone(),
        supported_blocks: supported,
        unsupported_blocks: unsupported,
        is_solver_supported,
    };

    Ok((diagram, report))
}
