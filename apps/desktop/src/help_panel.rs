//! Help and Documentation Panel (Offline, Clean-Room, Zero AI).
//!
//! Provides searchable, categorized function references, syntax guides,
//! and example code for MATLAB-compatible functions implemented in MatRust Studio.

use crate::theme::Theme;
use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};

/// Documentation record for a built-in or library function.
#[derive(Clone, Debug)]
pub struct DocEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub syntax: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub example: &'static str,
}

/// Offline documentation catalog of supported MATLAB-compatible functions.
pub static DOC_CATALOG: &[DocEntry] = &[
    DocEntry {
        name: "abs",
        category: "Math & Elementary",
        syntax: "y = abs(x)",
        summary: "Absolute value and complex magnitude.",
        description: "Returns the absolute value of each element in array x. For complex elements, returns the magnitude sqrt(re^2 + im^2).",
        example: "abs([-5 3 -2.5])  % returns [5 3 2.5]",
    },
    DocEntry {
        name: "sin",
        category: "Math & Elementary",
        syntax: "y = sin(x)",
        summary: "Sine of argument in radians.",
        description: "Computes the trigonometric sine for each element of x in radians.",
        example: "x = 0:0.1:pi; y = sin(x);",
    },
    DocEntry {
        name: "cos",
        category: "Math & Elementary",
        syntax: "y = cos(x)",
        summary: "Cosine of argument in radians.",
        description: "Computes the trigonometric cosine for each element of x in radians.",
        example: "y = cos([0, pi/2, pi]);",
    },
    DocEntry {
        name: "exp",
        category: "Math & Elementary",
        syntax: "y = exp(x)",
        summary: "Exponential function.",
        description: "Computes the base-e exponential e^x for each element of x.",
        example: "exp([0 1 2])",
    },
    DocEntry {
        name: "log",
        category: "Math & Elementary",
        syntax: "y = log(x)",
        summary: "Natural logarithm.",
        description: "Computes the natural (base-e) logarithm for each positive element of x.",
        example: "log(exp(1))  % returns 1",
    },
    DocEntry {
        name: "log2",
        category: "Math & Elementary",
        syntax: "y = log2(x)",
        summary: "Base-2 logarithm.",
        description: "Computes the base-2 logarithm for each positive element of x. Frequently used in information theory and entropy computations.",
        example: "log2([1 2 4 8 16])  % returns [0 1 2 3 4]",
    },
    DocEntry {
        name: "log10",
        category: "Math & Elementary",
        syntax: "y = log10(x)",
        summary: "Common (base-10) logarithm.",
        description: "Computes the base-10 logarithm for each element of x.",
        example: "log10([1 10 100 1000])",
    },
    DocEntry {
        name: "sqrt",
        category: "Math & Elementary",
        syntax: "y = sqrt(x)",
        summary: "Square root.",
        description: "Returns the square root of each element of array x.",
        example: "sqrt([4 9 16 25])",
    },
    DocEntry {
        name: "sum",
        category: "Statistics & Data",
        syntax: "s = sum(A)\ns = sum(A, dim)",
        summary: "Sum of array elements.",
        description: "Computes the sum of elements along the first non-singleton dimension, or along dimension dim if specified.",
        example: "sum([1 2 3 4])  % returns 10",
    },
    DocEntry {
        name: "mean",
        category: "Statistics & Data",
        syntax: "m = mean(A)\nm = mean(A, 'omitnan')",
        summary: "Average or mean value of arrays.",
        description: "Returns the arithmetic mean of the values in array A. Supports 'omitnan' flag to ignore NaN values.",
        example: "mean([10 20 30])  % returns 20",
    },
    DocEntry {
        name: "std",
        category: "Statistics & Data",
        syntax: "s = std(A)\ns = std(A, w, dim)",
        summary: "Standard deviation.",
        description: "Computes the sample standard deviation normalized by N-1 (or N if w=1).",
        example: "std([2 4 4 4 5 5 7 9])",
    },
    DocEntry {
        name: "var",
        category: "Statistics & Data",
        syntax: "v = var(A)",
        summary: "Variance of array elements.",
        description: "Computes the variance of the elements of array A.",
        example: "var([1 2 3 4 5])",
    },
    DocEntry {
        name: "median",
        category: "Statistics & Data",
        syntax: "m = median(A)",
        summary: "Median value of array.",
        description: "Returns the middle value after sorting the elements along the working dimension.",
        example: "median([1 5 2 8 7])  % returns 5",
    },
    DocEntry {
        name: "prctile",
        category: "Statistics & Data",
        syntax: "y = prctile(x, p)",
        summary: "Percentiles of sample.",
        description: "Returns the p-th percentile of data in x using linear interpolation between empirical values.",
        example: "prctile([1 2 3 4 5], 25)  % returns 1.75",
    },
    DocEntry {
        name: "inv",
        category: "Linear Algebra",
        syntax: "X = inv(A)",
        summary: "Matrix inverse.",
        description: "Computes the matrix inverse X such that A*X = I. Issues a warning if matrix is singular or nearly singular.",
        example: "A = [1 2; 3 4]; inv(A)",
    },
    DocEntry {
        name: "det",
        category: "Linear Algebra",
        syntax: "d = det(A)",
        summary: "Matrix determinant.",
        description: "Calculates the determinant of square matrix A using LU decomposition.",
        example: "det([1 2; 3 4])  % returns -2",
    },
    DocEntry {
        name: "eig",
        category: "Linear Algebra",
        syntax: "d = eig(A)\n[V, D] = eig(A)",
        summary: "Eigenvalues and eigenvectors.",
        description: "Returns a column vector d of eigenvalues, or diagonal matrix D of eigenvalues and matrix V whose columns are eigenvectors.",
        example: "A = [2 0; 0 3]; [V, D] = eig(A)",
    },
    DocEntry {
        name: "svd",
        category: "Linear Algebra",
        syntax: "s = svd(A)\n[U, S, V] = svd(A)",
        summary: "Singular value decomposition.",
        description: "Decomposes matrix A into U*S*V' where U and V are unitary matrices and S contains singular values in descending order.",
        example: "A = [1 2; 3 4]; [U, S, V] = svd(A)",
    },
    DocEntry {
        name: "lu",
        category: "Linear Algebra",
        syntax: "[L, U, P] = lu(A)",
        summary: "LU matrix factorization.",
        description: "Factors matrix A into lower triangular matrix L and upper triangular matrix U with permutation matrix P such that P*A = L*U.",
        example: "[L, U, P] = lu([1 2; 3 4])",
    },
    DocEntry {
        name: "qr",
        category: "Linear Algebra",
        syntax: "[Q, R] = qr(A)",
        summary: "Orthogonal-triangular QR decomposition.",
        description: "Produces orthogonal matrix Q and upper triangular matrix R such that A = Q*R.",
        example: "[Q, R] = qr([1 2; 3 4])",
    },
    DocEntry {
        name: "rank",
        category: "Linear Algebra",
        syntax: "k = rank(A)",
        summary: "Matrix rank.",
        description: "Returns the number of singular values of A that exceed a tolerance based on matrix size and machine precision.",
        example: "rank([1 2; 2 4])  % returns 1",
    },
    DocEntry {
        name: "plot",
        category: "Plotting & Graphics",
        syntax: "plot(x, y)\nplot(y)",
        summary: "2-D line plot.",
        description: "Creates a 2-D line plot of the data in y versus the corresponding values in x.",
        example: "x = 0:0.1:10; y = sin(x); plot(x, y); title('Sine Wave');",
    },
    DocEntry {
        name: "scatter",
        category: "Plotting & Graphics",
        syntax: "scatter(x, y)",
        summary: "Scatter plot.",
        description: "Displays circle markers at locations specified by vectors x and y.",
        example: "scatter(rand(1,50), rand(1,50));",
    },
    DocEntry {
        name: "bar",
        category: "Plotting & Graphics",
        syntax: "bar(y)\nbar(x, y)",
        summary: "Bar graph.",
        description: "Draws a bar graph with one bar for each element of y at positions in x.",
        example: "bar([23 45 12 67 89]);",
    },
    DocEntry {
        name: "hist",
        category: "Plotting & Graphics",
        syntax: "hist(y)\nhist(y, nbins)",
        summary: "Histogram.",
        description: "Groups data into bins and plots a bar for each bin showing frequency.",
        example: "hist(randn(1, 1000), 20);",
    },
    DocEntry {
        name: "title",
        category: "Plotting & Graphics",
        syntax: "title('text')",
        summary: "Add title to current plot axes.",
        description: "Displays the specified text above the current plot axes.",
        example: "title('Frequency Response');",
    },
    DocEntry {
        name: "xlabel",
        category: "Plotting & Graphics",
        syntax: "xlabel('label')",
        summary: "Label x-axis.",
        description: "Attaches a label to the x-axis of the active plot axes.",
        example: "xlabel('Time (seconds)');",
    },
    DocEntry {
        name: "ylabel",
        category: "Plotting & Graphics",
        syntax: "ylabel('label')",
        summary: "Label y-axis.",
        description: "Attaches a label to the y-axis of the active plot axes.",
        example: "ylabel('Amplitude (V)');",
    },
    DocEntry {
        name: "grid",
        category: "Plotting & Graphics",
        syntax: "grid on\ngrid off",
        summary: "Toggle major grid lines on plot.",
        description: "Displays or hides the background grid lines on current axes.",
        example: "grid('on');",
    },
    DocEntry {
        name: "load",
        category: "File & I/O",
        syntax: "load(filename)\ns = load(filename)",
        summary: "Load workspace variables from MAT-file.",
        description: "Loads variables saved in MAT-file (Level 5 format) into the current workspace.",
        example: "load('data.mat');",
    },
    DocEntry {
        name: "save",
        category: "File & I/O",
        syntax: "save(filename)\nsave(filename, 'var1', 'var2')",
        summary: "Save workspace variables to MAT-file.",
        description: "Writes specified or all variables to a binary MAT-file.",
        example: "x = 1:10; save('results.mat', 'x');",
    },
    DocEntry {
        name: "readtable",
        category: "File & I/O",
        syntax: "T = readtable(filename)",
        summary: "Read tabular data from CSV or text file.",
        description: "Reads column-oriented data from a file into a Table object with named columns and type inference.",
        example: "T = readtable('experiment_data.csv');",
    },
    DocEntry {
        name: "writetable",
        category: "File & I/O",
        syntax: "writetable(T, filename)",
        summary: "Write Table to CSV file.",
        description: "Exports tabular data T to a comma-separated text file with column header names.",
        example: "writetable(T, 'export.csv');",
    },
    DocEntry {
        name: "sim",
        category: "Simulink & Systems",
        syntax: "out = sim(model_name)\nout = sim(model_name, 'StopTime', 10.0)",
        summary: "Simulate a block diagram model.",
        description: "Executes fixed-step simulation of a Simulink or BlockDiagram model and returns time-series signal outputs.",
        example: "out = sim('feedback_controller', 'StopTime', 5.0);",
    },
    DocEntry {
        name: "disp",
        category: "Programming & Language",
        syntax: "disp(X)",
        summary: "Display value or text.",
        description: "Prints array or character string X to the Command Window without displaying the variable name.",
        example: "disp('Simulation completed successfully.')",
    },
    DocEntry {
        name: "num2str",
        category: "Programming & Language",
        syntax: "str = num2str(x)",
        summary: "Convert numbers to character representation.",
        description: "Converts scalar or matrix numeric values into character array suitable for display or string concatenation.",
        example: "entropy = 1.8464; disp(['Entropy: ', num2str(entropy)]);",
    },
    DocEntry {
        name: "clc",
        category: "Programming & Language",
        syntax: "clc",
        summary: "Clear Command Window.",
        description: "Clears all displayed input and output text from the Command Window.",
        example: "clc",
    },
    DocEntry {
        name: "clear",
        category: "Programming & Language",
        syntax: "clear\nclear var1 var2",
        summary: "Clear variables from workspace memory.",
        description: "Removes variables from the active session workspace, releasing allocated memory.",
        example: "clear",
    },
];

/// State of the Help & Documentation Panel.
pub struct HelpPanel {
    pub search_query: String,
    pub selected_category: String,
    pub selected_entry: Option<&'static DocEntry>,
}

impl Default for HelpPanel {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_category: "All Categories".to_string(),
            selected_entry: DOC_CATALOG.iter().find(|e| e.name == "log2"),
        }
    }
}

impl HelpPanel {
    /// Selects a specific function in the help catalog by name.
    pub fn select_function(&mut self, name: &str) {
        let name_lower = name.trim().to_lowercase();
        if let Some(entry) = DOC_CATALOG
            .iter()
            .find(|e| e.name.to_lowercase() == name_lower)
        {
            self.selected_entry = Some(entry);
            self.search_query.clear();
        }
    }

    /// Renders the Help & Documentation panel UI.
    pub fn show(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(RichText::new("❓ Help & Documentation").color(theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Offline Clean-Room Reference (Zero AI)")
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                });
            });
            ui.separator();

            // Search Bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(14.0));
                let search_edit = TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search functions (e.g. log2, sum, inv, plot)...")
                    .desired_width(ui.available_width() - 80.0);
                ui.add(search_edit);

                if !self.search_query.is_empty() && ui.button("Clear").clicked() {
                    self.search_query.clear();
                }
            });

            ui.add_space(4.0);

            // Category Filter Bar
            let categories = [
                "All Categories",
                "Math & Elementary",
                "Statistics & Data",
                "Linear Algebra",
                "Plotting & Graphics",
                "File & I/O",
                "Programming & Language",
                "Simulink & Systems",
            ];

            egui::ComboBox::from_label("Category")
                .selected_text(&self.selected_category)
                .show_ui(ui, |ui| {
                    for cat in categories {
                        ui.selectable_value(&mut self.selected_category, cat.to_string(), cat);
                    }
                });

            ui.separator();

            // Split: Left = Matching Functions List, Right = Detailed Documentation View
            let query = self.search_query.trim().to_lowercase();
            let cat = &self.selected_category;

            let filtered_entries: Vec<&'static DocEntry> = DOC_CATALOG
                .iter()
                .filter(|e| {
                    let matches_cat = cat == "All Categories" || e.category == cat.as_str();
                    let matches_query = query.is_empty()
                        || e.name.to_lowercase().contains(&query)
                        || e.summary.to_lowercase().contains(&query);
                    matches_cat && matches_query
                })
                .collect();

            ui.columns(2, |columns| {
                // Left Column: Function List
                columns[0].vertical(|ui| {
                    ui.label(
                        RichText::new(format!("Functions ({})", filtered_entries.len()))
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ScrollArea::vertical()
                        .id_salt("help_fn_list")
                        .max_height(ui.available_height() - 10.0)
                        .show(ui, |ui| {
                            for entry in &filtered_entries {
                                let is_selected = self
                                    .selected_entry
                                    .map(|curr| curr.name == entry.name)
                                    .unwrap_or(false);

                                let label = format!("{}  - {}", entry.name, entry.summary);
                                if ui.selectable_label(is_selected, label).clicked() {
                                    self.selected_entry = Some(entry);
                                }
                            }
                        });
                });

                // Right Column: Detailed Documentation Card
                columns[1].vertical(|ui| {
                    if let Some(entry) = self.selected_entry {
                        ScrollArea::vertical()
                            .id_salt("help_fn_detail")
                            .max_height(ui.available_height() - 10.0)
                            .show(ui, |ui| {
                                ui.heading(
                                    RichText::new(entry.name).color(theme.accent).size(20.0),
                                );
                                ui.label(
                                    RichText::new(format!("Category: {}", entry.category))
                                        .size(11.0)
                                        .color(theme.text_secondary),
                                );
                                ui.add_space(6.0);

                                ui.label(
                                    RichText::new(entry.summary)
                                        .strong()
                                        .color(theme.text_primary),
                                );
                                ui.add_space(8.0);

                                // Syntax block
                                ui.label(RichText::new("Syntax:").strong());
                                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(entry.syntax)
                                                .monospace()
                                                .color(Color32::from_rgb(180, 220, 255)),
                                        )
                                        .selectable(true),
                                    );
                                });
                                ui.add_space(8.0);

                                // Description
                                ui.label(RichText::new("Description:").strong());
                                ui.label(
                                    RichText::new(entry.description).color(theme.text_primary),
                                );
                                ui.add_space(8.0);

                                // Example
                                ui.label(RichText::new("Example:").strong());
                                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(entry.example)
                                                .monospace()
                                                .color(Color32::from_rgb(160, 240, 180)),
                                        )
                                        .selectable(true),
                                    );
                                });
                            });
                    } else {
                        ui.label(
                            RichText::new("Select a function from the list to view documentation.")
                                .italics()
                                .color(theme.text_secondary),
                        );
                    }
                });
            });
        });
    }
}
