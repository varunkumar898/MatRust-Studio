//! Electrical netlist resolution, union-find circuit extraction, and domain validation.

use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::model::{BlockDiagram, BlockType, PortKind};
use std::collections::{HashMap, HashSet};

/// Resolved electrical network topology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElectricalTopology {
    /// Mapping from (block_sid, terminal_index) to net_id (Node 0 is always Ground).
    pub terminal_to_net: HashMap<(String, usize), usize>,
    /// Sorted list of unique net IDs present in the model (e.g. [0, 1, 2, ...]).
    pub net_ids: Vec<usize>,
    /// Mapping from net_id to all connected terminals [(block_sid, terminal_index)].
    pub net_terminals: HashMap<usize, Vec<(String, usize)>>,
    /// Number of distinct electrical islands (connected components) detected.
    pub island_count: usize,
}

/// Result of netlist extraction containing the topology (if valid) and all diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetlistResult {
    pub topology: Option<ElectricalTopology>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Disjoint-set union-find for electrical terminal indices.
struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return;
        }
        if self.rank[root_x] < self.rank[root_y] {
            self.parent[root_x] = root_y;
        } else if self.rank[root_x] > self.rank[root_y] {
            self.parent[root_y] = root_x;
        } else {
            self.parent[root_y] = root_x;
            self.rank[root_x] += 1;
        }
    }
}

/// Resolves electrical networks and nets in a block diagram using union-find.
///
/// Implements deterministic net ID assignment (Ground = 0, invariant to line ordering),
/// floating terminal warnings (W-OPEN), duplicate wire warnings (W-WIRE),
/// domain mismatch errors (E-DOMAIN), and missing ground errors (E-GND).
pub fn resolve_electrical_nets(diagram: &BlockDiagram) -> NetlistResult {
    let mut diagnostics = Vec::new();

    let has_electrical = diagram.blocks.iter().any(|b| b.block_type.is_electrical());
    if !has_electrical {
        return NetlistResult {
            topology: Some(ElectricalTopology {
                terminal_to_net: HashMap::new(),
                net_ids: Vec::new(),
                net_terminals: HashMap::new(),
                island_count: 0,
            }),
            diagnostics: Vec::new(),
        };
    }

    // 1. Validate parameters for all blocks
    for block in &diagram.blocks {
        diagnostics.extend(crate::spec::validate_block_params(
            &block.block_type,
            &block.sid,
            &block.parameters,
        ));
    }

    // 2. Inventory all blocks and terminals
    let block_map: HashMap<&str, &crate::model::Block> =
        diagram.blocks.iter().map(|b| (b.sid.as_str(), b)).collect();

    // Map each electrical terminal (block_sid, term_idx) to a contiguous index for union-find
    let mut terminal_list = Vec::new();
    let mut terminal_to_id = HashMap::new();
    let mut ground_term_ids = Vec::new();

    let elec_blocks: Vec<&crate::model::Block> = diagram
        .blocks
        .iter()
        .filter(|b| b.block_type.is_electrical())
        .collect();
    let elec_block_to_id: HashMap<&str, usize> = elec_blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.sid.as_str(), i))
        .collect();
    let mut block_uf = DisjointSet::new(elec_blocks.len());
    let mut block_has_wire = vec![false; elec_blocks.len()];

    for block in &diagram.blocks {
        let term_count = block.terminal_count();
        for t in 0..term_count {
            let id = terminal_list.len();
            let key = (block.sid.clone(), t);
            terminal_to_id.insert(key.clone(), id);
            terminal_list.push(key);
            if block.block_type == BlockType::Ground {
                ground_term_ids.push(id);
            }
        }
    }

    let mut uf = DisjointSet::new(terminal_list.len());
    let mut wire_counts = vec![0usize; terminal_list.len()];
    let mut seen_pairs: HashSet<((String, usize), (String, usize))> = HashSet::new();

    // 3. Process lines
    for line in &diagram.lines {
        let src_block = match block_map.get(line.src.block_sid.as_str()) {
            Some(b) => *b,
            None => {
                diagnostics.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Line connects from unknown block SID '{}' -> block does not exist in diagram -> verify connection line references an existing block",
                        line.src.block_sid
                    ),
                    vec![line.src.block_sid.clone()],
                ));
                continue;
            }
        };

        let dst_block = match block_map.get(line.dst.block_sid.as_str()) {
            Some(b) => *b,
            None => {
                diagnostics.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Line connects to unknown block SID '{}' -> block does not exist in diagram -> verify connection line references an existing block",
                        line.dst.block_sid
                    ),
                    vec![line.dst.block_sid.clone()],
                ));
                continue;
            }
        };

        // Determine port kind for src
        let (src_kind, src_term) = if src_block.block_type.is_electrical() {
            if src_block.block_type == BlockType::IdealSwitch
                && line.src.port_kind == Some(PortKind::Signal)
            {
                (PortKind::Signal, None)
            } else if line.src.port_index < src_block.terminal_count() {
                (PortKind::Electrical, Some(line.src.port_index))
            } else {
                diagnostics.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Unknown terminal index {} on block '{}' -> block only has {} terminal(s) -> use a valid terminal index (0..{})",
                        line.src.port_index,
                        line.src.block_sid,
                        src_block.terminal_count(),
                        src_block.terminal_count().saturating_sub(1)
                    ),
                    vec![line.src.block_sid.clone()],
                ));
                continue;
            }
        } else {
            (PortKind::Signal, None)
        };

        // Determine port kind for dst
        let (dst_kind, dst_term) = if dst_block.block_type.is_electrical() {
            if dst_block.block_type == BlockType::IdealSwitch
                && (line.dst.port_kind == Some(PortKind::Signal)
                    || line.src.port_kind == Some(PortKind::Signal)
                    || !src_block.block_type.is_electrical())
            {
                (PortKind::Signal, None)
            } else if line.dst.port_index < dst_block.terminal_count() {
                (PortKind::Electrical, Some(line.dst.port_index))
            } else {
                diagnostics.push(Diagnostic::error(
                    "E-PARAM",
                    format!(
                        "Unknown terminal index {} on block '{}' -> block only has {} terminal(s) -> use a valid terminal index (0..{})",
                        line.dst.port_index,
                        line.dst.block_sid,
                        dst_block.terminal_count(),
                        dst_block.terminal_count().saturating_sub(1)
                    ),
                    vec![line.dst.block_sid.clone()],
                ));
                continue;
            }
        } else {
            (PortKind::Signal, None)
        };

        // Check for domain mismatch
        if src_kind != dst_kind {
            diagnostics.push(Diagnostic::error(
                "E-DOMAIN",
                format!(
                    "Direct wire between signal port and electrical terminal on blocks '{}' and '{}' -> signal and electrical domains cannot be directly connected -> use a sensor or controlled source block to interface between domains",
                    line.src.block_sid,
                    line.dst.block_sid
                ),
                vec![line.src.block_sid.clone(), line.dst.block_sid.clone()],
            ));
            continue;
        }

        // If electrical, connect terminals
        if src_kind == PortKind::Electrical && dst_kind == PortKind::Electrical {
            let s_term = src_term.unwrap_or(0);
            let d_term = dst_term.unwrap_or(0);
            let s_key = (line.src.block_sid.clone(), s_term);
            let d_key = (line.dst.block_sid.clone(), d_term);

            // Self-loop wire
            if s_key == d_key {
                diagnostics.push(Diagnostic::warning(
                    "W-WIRE",
                    format!(
                        "Wire connects terminal {} on block '{}' to itself -> self-loop connection is redundant -> remove the self-loop wire",
                        s_term,
                        line.src.block_sid
                    ),
                    vec![line.src.block_sid.clone()],
                ));
                continue;
            }

            // Duplicate wire check (undirected)
            let pair = if s_key < d_key {
                (s_key.clone(), d_key.clone())
            } else {
                (d_key.clone(), s_key.clone())
            };

            if seen_pairs.contains(&pair) {
                diagnostics.push(Diagnostic::warning(
                    "W-WIRE",
                    format!(
                        "Duplicate wire between terminal {} on block '{}' and terminal {} on block '{}' -> multiple wires connect the same terminal pair -> remove redundant wire",
                        s_term,
                        line.src.block_sid,
                        d_term,
                        line.dst.block_sid
                    ),
                    vec![line.src.block_sid.clone(), line.dst.block_sid.clone()],
                ));
                continue;
            }
            seen_pairs.insert(pair);

            if let (Some(&u_id), Some(&v_id)) =
                (terminal_to_id.get(&s_key), terminal_to_id.get(&d_key))
            {
                uf.union(u_id, v_id);
                wire_counts[u_id] += 1;
                wire_counts[v_id] += 1;

                if let (Some(&b_u), Some(&b_v)) = (
                    elec_block_to_id.get(line.src.block_sid.as_str()),
                    elec_block_to_id.get(line.dst.block_sid.as_str()),
                ) {
                    block_uf.union(b_u, b_v);
                    block_has_wire[b_u] = true;
                    block_has_wire[b_v] = true;
                }
            }
        }
    }

    // 4. Check for open terminals (W-OPEN)
    for (idx, (sid, term)) in terminal_list.iter().enumerate() {
        if wire_counts[idx] == 0 {
            diagnostics.push(Diagnostic::warning(
                "W-OPEN",
                format!(
                    "Unconnected electrical terminal {} on block '{}' -> terminal has no wires attached -> connect the terminal or remove the unused component",
                    term,
                    sid
                ),
                vec![sid.clone()],
            ));
        }
    }

    // 5. Merge all Ground blocks into common reference Node 0
    let mut ground_root = None;
    if !ground_term_ids.is_empty() {
        let first_gnd = ground_term_ids[0];
        for &other_gnd in &ground_term_ids[1..] {
            uf.union(first_gnd, other_gnd);
        }
        ground_root = Some(uf.find(first_gnd));
    }

    // 6. Group electrical blocks into connected components (islands)
    let mut island_blocks: HashMap<usize, Vec<&crate::model::Block>> = HashMap::new();
    for (idx, &b) in elec_blocks.iter().enumerate() {
        let root = block_uf.find(idx);
        island_blocks.entry(root).or_default().push(b);
    }

    // 7. Check each island for Ground connection (E-GND)
    // An island of electrical blocks with NO Ground gives E-GND naming the disconnected blocks.
    // Completely idle/isolated blocks with no connected wires are covered by W-OPEN warnings,
    // so only islands where at least one block has an attached wire need to be reported as E-GND.
    for blocks in island_blocks.values() {
        let has_ground = blocks.iter().any(|b| b.block_type == BlockType::Ground);
        if !has_ground {
            let has_any_wire = blocks.iter().any(|b| {
                if let Some(&b_idx) = elec_block_to_id.get(b.sid.as_str()) {
                    block_has_wire[b_idx]
                } else {
                    false
                }
            });
            if has_any_wire {
                let mut sids: Vec<String> = blocks.iter().map(|b| b.sid.clone()).collect();
                sids.sort();
                sids.dedup();
                diagnostics.push(Diagnostic::error(
                    "E-GND",
                    format!(
                        "Electrical island containing blocks {:?} has no Ground reference -> every connected electrical circuit needs a Ground block -> connect at least one terminal in this island to a Ground block",
                        sids
                    ),
                    sids,
                ));
            }
        }
    }

    // 8. Assign deterministic net IDs
    // Ground net is always 0.
    // Non-ground nets are numbered 1, 2, ... in order of first appearance of terminals
    // when iterating blocks in diagram block order, then terminals in terminal order.
    let mut root_to_net: HashMap<usize, usize> = HashMap::new();
    if let Some(g_root) = ground_root {
        root_to_net.insert(g_root, 0);
    }

    let mut next_net_id = 1usize;
    for block in &diagram.blocks {
        let term_count = block.terminal_count();
        for t in 0..term_count {
            let key = (block.sid.clone(), t);
            if let Some(&idx) = terminal_to_id.get(&key) {
                let root = uf.find(idx);
                if let std::collections::hash_map::Entry::Vacant(e) = root_to_net.entry(root) {
                    e.insert(next_net_id);
                    next_net_id += 1;
                }
            }
        }
    }

    let mut terminal_to_net = HashMap::new();
    let mut net_terminals: HashMap<usize, Vec<(String, usize)>> = HashMap::new();

    for (idx, key) in terminal_list.iter().enumerate() {
        let root = uf.find(idx);
        if let Some(&net_id) = root_to_net.get(&root) {
            terminal_to_net.insert(key.clone(), net_id);
            net_terminals.entry(net_id).or_default().push(key.clone());
        }
    }

    let mut net_ids: Vec<usize> = net_terminals.keys().copied().collect();
    net_ids.sort_unstable();

    let has_errors = diagnostics
        .iter()
        .any(|d| d.severity == DiagnosticSeverity::Error);

    let topology = if has_errors {
        None
    } else {
        Some(ElectricalTopology {
            terminal_to_net,
            net_ids,
            net_terminals,
            island_count: island_blocks.len(),
        })
    };

    NetlistResult {
        topology,
        diagnostics,
    }
}
