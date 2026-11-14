# ADR 0001: Record Architecture Decisions

- **Status:** Accepted
- **Date:** 2026-11-14
- **Deciders:** Project Owner, AI Engineering Team

## Context

As MatRust Studio evolves beyond Milestone M12 into multi-phase simulation extensions (Phases F1 through F16), complex architectural choices must be made regarding solver algorithms, external bridging, memory models, and subsystem interfaces. Without a formal decision record, technical rationale is easily forgotten, leading to conflicting architectural patterns and regressions against core engineering principles.

## Decision

We adopt Architecture Decision Records (ADRs) as the canonical documentation mechanism for significant architectural, structural, and dependency choices:

1. **Storage Location**: ADRs reside in `docs/adr/` numbered sequentially (`NNNN-title.md`).
2. **Format**: Every ADR follows the lightweight Nygard structure: Context, Decision, Status, and Consequences.
3. **Mandatory Triggers**: An ADR is required for:
   - Any roadmap item decomposed from an **XL** task (such as DAE solvers, Radau IIA, FMU export, HDL generation, GPU compute).
   - Any proposed new external crate dependency.
   - Any architectural boundary adjustment between numerical crates and user interfaces.
4. **Approval Gate**: ADRs require explicit project owner approval prior to merging implementation code.

## Consequences

- Architectural decisions have clear provenance and recorded rationale.
- Proposed structural changes cannot be made ad-hoc in implementation pull requests.
- Decisions are immutable historical records; if a decision is superseded, a new ADR is authored that explicitly references and deprecates the prior record.
