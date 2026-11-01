//! Robust XML parser for Simulink block diagram definitions with nesting limits.

use crate::model::{Block, BlockDiagram, BlockType, Line, PortRef};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::BufRead;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Security violation: XML element nesting depth limit exceeded ({0} > {1})")]
    DepthLimitExceeded(usize, usize),

    #[error("Invalid model format: {0}")]
    Format(String),
}

const MAX_XML_DEPTH: usize = 1000;

/// Parses a `simulink/blockdiagram.xml` stream into a `BlockDiagram`.
pub fn parse_blockdiagram_xml<R: BufRead>(reader: R) -> Result<BlockDiagram, ParseError> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);

    let mut diagram = BlockDiagram::new("Untitled");
    let mut depth: usize = 0;

    // Parser state tracking
    let mut current_block: Option<Block> = None;
    let mut current_p_name: Option<String> = None;
    let mut in_line = false;
    let mut line_src: Option<PortRef> = None;
    let mut line_dsts: Vec<PortRef> = Vec::new();

    let mut buf = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                depth += 1;
                if depth > MAX_XML_DEPTH {
                    return Err(ParseError::DepthLimitExceeded(depth, MAX_XML_DEPTH));
                }

                let tag_name = e.name().into_inner();
                match tag_name {
                    b"Model" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.into_inner() == b"Name" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    diagram.name = val.to_string();
                                }
                            } else if attr.key.into_inner() == b"Version" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    diagram.version = Some(val.to_string());
                                }
                            }
                        }
                    }
                    b"Block" => {
                        let mut b_type = BlockType::Unknown("Unknown".to_string());
                        let mut b_name = String::new();
                        let mut b_sid = String::new();

                        for attr in e.attributes().flatten() {
                            match attr.key.into_inner() {
                                b"BlockType" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        b_type = BlockType::from_str_name(val);
                                    }
                                }
                                b"Name" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        b_name = val.to_string();
                                    }
                                }
                                b"SID" => {
                                    if let Ok(val) = std::str::from_utf8(&attr.value) {
                                        b_sid = val.to_string();
                                    }
                                }
                                _ => {}
                            }
                        }
                        if b_sid.is_empty() {
                            b_sid = format!("sid_{}", diagram.blocks.len() + 1);
                        }
                        if b_name.is_empty() {
                            b_name = format!("Block_{}", b_sid);
                        }
                        current_block = Some(Block::new(b_sid, b_name, b_type));
                    }
                    b"Line" => {
                        in_line = true;
                        line_src = None;
                        line_dsts.clear();
                    }
                    b"Branch" => {}
                    b"P" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.into_inner() == b"Name" {
                                if let Ok(val) = std::str::from_utf8(&attr.value) {
                                    current_p_name = Some(val.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(t) => {
                if let Some(ref p_name) = current_p_name {
                    if let Ok(text) = std::str::from_utf8(t.as_ref()) {
                        let text_str = text.trim();
                        if let Some(ref mut blk) = current_block {
                            if p_name == "Position" {
                                if let Some(coords) = parse_coords(text_str) {
                                    blk.position = Some(coords);
                                }
                            }
                            blk.parameters.insert(p_name.clone(), text_str.to_string());
                        } else if in_line {
                            if p_name == "Src" {
                                if let Some(pr) = parse_port_ref(text_str) {
                                    line_src = Some(pr);
                                }
                            } else if p_name == "Dst" {
                                if let Some(pr) = parse_port_ref(text_str) {
                                    line_dsts.push(pr);
                                }
                            }
                        } else {
                            if p_name == "Release" || p_name == "SimulinkVersion" {
                                diagram.release = Some(text_str.to_string());
                            } else if p_name == "Toolbox" {
                                diagram.toolboxes.push(text_str.to_string());
                            }
                            diagram
                                .properties
                                .insert(p_name.clone(), text_str.to_string());
                        }
                    }
                }
            }
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                match e.name().into_inner() {
                    b"Block" => {
                        if let Some(b) = current_block.take() {
                            diagram.add_block(b);
                        }
                    }
                    b"Branch" => {}
                    b"Line" => {
                        if let Some(src) = line_src.take() {
                            for dst in &line_dsts {
                                diagram.lines.push(Line {
                                    src: src.clone(),
                                    dst: dst.clone(),
                                    name: None,
                                });
                            }
                        }
                        in_line = false;
                        line_dsts.clear();
                    }
                    b"P" => {
                        current_p_name = None;
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(diagram)
}

/// Parses port reference strings like "1#out:1" or "Gain#in:1" into a `PortRef`.
fn parse_port_ref(s: &str) -> Option<PortRef> {
    let s = s.trim();
    if let Some((sid_part, port_part)) = s.split_once('#') {
        let port_idx = if let Some((_, idx_str)) = port_part.split_once(':') {
            idx_str.parse::<usize>().unwrap_or(1)
        } else {
            1
        };
        Some(PortRef {
            block_sid: sid_part.to_string(),
            port_index: port_idx,
        })
    } else {
        // Fallback for simple sid
        Some(PortRef {
            block_sid: s.to_string(),
            port_index: 1,
        })
    }
}

/// Parses position string like "[100, 200, 300, 400]".
fn parse_coords(s: &str) -> Option<[f64; 4]> {
    let s = s.trim().trim_start_matches('[').trim_end_matches(']');
    let parts: Vec<&str> = s.split([',', ' ']).filter(|p| !p.is_empty()).collect();
    if parts.len() == 4 {
        let x1 = parts[0].parse::<f64>().ok()?;
        let y1 = parts[1].parse::<f64>().ok()?;
        let x2 = parts[2].parse::<f64>().ok()?;
        let y2 = parts[3].parse::<f64>().ok()?;
        Some([x1, y1, x2, y2])
    } else {
        None
    }
}
