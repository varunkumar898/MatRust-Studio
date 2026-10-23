//! App framework for MatRust Studio.
//!
//! Rules enforced by design:
//! - Dependencies only on matlab_* crates; never on egui.
//! - The registry is the single source of truth for launcher status.
//! - `Ready` means UI + real backend + tested results + export. Backend-only
//!   work is `Partial`; unimplemented apps are `Planned` and disabled.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Ready,
    Partial,
    Planned,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verification {
    Unit,
    MathValidated,
    NotReferenceVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub status: Status,
    pub verification: Verification,
    pub needs_file_input: bool,
}

impl AppInfo {
    pub fn status_text(&self) -> &'static str {
        match self.status {
            Status::Ready => "Ready",
            Status::Partial => "Partial",
            Status::Planned => "Planned",
            Status::Blocked => "Blocked",
        }
    }

    pub fn verification_text(&self) -> &'static str {
        match self.verification {
            Verification::Unit => "Unit-tested",
            Verification::MathValidated => "Math-validated",
            Verification::NotReferenceVerified => "Not reference-verified",
        }
    }

    /// Only a Ready app has a working UI and may be launched.
    pub fn launchable(&self) -> bool {
        self.status == Status::Ready
    }
}

pub const REGISTRY: &[AppInfo] = &[
    AppInfo {
        id: "data_analysis",
        name: "Data Analysis",
        description: "Open CSV, preview columns, statistics, plot, export",
        status: Status::Ready,
        verification: Verification::Unit,
        needs_file_input: true,
    },
    AppInfo {
        id: "curve_fitter",
        name: "Curve Fitter",
        description:
            "Polynomial / linear least-squares fit on CSV columns, fit and residual plots, export",
        status: Status::Ready,
        verification: Verification::Unit,
        needs_file_input: true,
    },
    AppInfo {
        id: "simulation",
        name: "Simulation and Control",
        description: "RK4 first/second-order and state-space responses",
        status: Status::Planned,
        verification: Verification::NotReferenceVerified,
        needs_file_input: false,
    },
    AppInfo {
        id: "signal_analyzer",
        name: "Signal Analyzer",
        description: "FFT, windows, convolution, correlation, FIR filtering",
        status: Status::Planned,
        verification: Verification::NotReferenceVerified,
        needs_file_input: true,
    },
    AppInfo {
        id: "image_processing",
        name: "Image Processing",
        description: "Preview, grayscale, crop, resize, threshold, filters",
        status: Status::Planned,
        verification: Verification::NotReferenceVerified,
        needs_file_input: true,
    },
    AppInfo {
        id: "machine_learning",
        name: "Machine Learning",
        description: "Seeded regression, logistic baseline, k-means",
        status: Status::Planned,
        verification: Verification::NotReferenceVerified,
        needs_file_input: true,
    },
    AppInfo {
        id: "robotics",
        name: "Robotics Math",
        description: "Rotations, transforms, planar two-link FK/IK, PID",
        status: Status::Planned,
        verification: Verification::NotReferenceVerified,
        needs_file_input: false,
    },
    AppInfo {
        id: "embedded",
        name: "Embedded and Sensor Data",
        description: "Recorded CSV time series, filtering, statistics, export",
        status: Status::Planned,
        verification: Verification::NotReferenceVerified,
        needs_file_input: true,
    },
];

pub fn find_by_id(id: &str) -> Option<&'static AppInfo> {
    REGISTRY.iter().find(|app| app.id == id)
}

pub mod curve_fitter;
pub mod data_analysis;
