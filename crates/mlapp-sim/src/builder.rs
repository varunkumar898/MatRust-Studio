//! Programmatic builder and serializer for clean-room synthetic .slx archives.

use crate::model::BlockDiagram;
use std::io::{Seek, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Serializes a `BlockDiagram` into valid XML string representation.
pub fn blockdiagram_to_xml(diagram: &BlockDiagram) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<ModelInformation Version=\"1.0\">\n");
    xml.push_str(&format!(
        "  <Model Name=\"{}\" Version=\"{}\">\n",
        diagram.name,
        diagram.version.as_deref().unwrap_or("1.0")
    ));

    if let Some(ref rel) = diagram.release {
        xml.push_str(&format!("    <P Name=\"Release\">{rel}</P>\n"));
    }
    for tb in &diagram.toolboxes {
        xml.push_str(&format!("    <P Name=\"Toolbox\">{tb}</P>\n"));
    }
    for (k, v) in &diagram.properties {
        xml.push_str(&format!("    <P Name=\"{k}\">{v}</P>\n"));
    }

    xml.push_str("    <System>\n");

    for b in &diagram.blocks {
        xml.push_str(&format!(
            "      <Block BlockType=\"{:?}\" Name=\"{}\" SID=\"{}\">\n",
            b.block_type, b.name, b.sid
        ));
        if let Some(pos) = b.position {
            xml.push_str(&format!(
                "        <P Name=\"Position\">[{}, {}, {}, {}]</P>\n",
                pos[0], pos[1], pos[2], pos[3]
            ));
        }
        for (k, v) in &b.parameters {
            xml.push_str(&format!("        <P Name=\"{k}\">{v}</P>\n"));
        }
        xml.push_str("      </Block>\n");
    }

    for l in &diagram.lines {
        xml.push_str("      <Line>\n");
        xml.push_str(&format!(
            "        <P Name=\"Src\">{}#out:{}</P>\n",
            l.src.block_sid, l.src.port_index
        ));
        xml.push_str(&format!(
            "        <P Name=\"Dst\">{}#in:{}</P>\n",
            l.dst.block_sid, l.dst.port_index
        ));
        xml.push_str("      </Line>\n");
    }

    xml.push_str("    </System>\n");
    xml.push_str("  </Model>\n");
    xml.push_str("</ModelInformation>\n");

    xml
}

/// Packages a `BlockDiagram` into an .slx (OPC ZIP) archive stream.
pub fn export_slx<W: Write + Seek>(
    writer: W,
    diagram: &BlockDiagram,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // 1. [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
</Types>"#,
    )?;

    // 2. _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.mathworks.com/simulink/2010/relationships/blockdiagram" Target="simulink/blockdiagram.xml"/>
</Relationships>"#,
    )?;

    // 3. metadata/coreProperties.xml
    zip.start_file("metadata/coreProperties.xml", options)?;
    let core_props = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>{}</dc:title>
  <cp:version>{}</cp:version>
</cp:coreProperties>"#,
        diagram.name,
        diagram.version.as_deref().unwrap_or("1.0")
    );
    zip.write_all(core_props.as_bytes())?;

    // 4. simulink/blockdiagram.xml
    let diagram_xml = blockdiagram_to_xml(diagram);
    zip.start_file("simulink/blockdiagram.xml", options)?;
    zip.write_all(diagram_xml.as_bytes())?;

    zip.finish()?;
    Ok(())
}
