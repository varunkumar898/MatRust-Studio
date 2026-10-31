use rustlab_eval::Interpreter;
use rustlab_stdlib::register_all;

#[test]
fn test_supported_functions_doc_parity_and_coverage() {
    let mut interp = Interpreter::new();
    register_all(&mut interp);

    let names = interp.builtin_names();
    assert!(
        names.len() >= 80,
        "Expected at least 80 registered builtins, found {}",
        names.len()
    );

    // Read docs/SUPPORTED_FUNCTIONS.md
    let candidate_paths = [
        "docs/SUPPORTED_FUNCTIONS.md",
        "../../docs/SUPPORTED_FUNCTIONS.md",
        "../docs/SUPPORTED_FUNCTIONS.md",
    ];
    let mut doc_text = None;
    for path in candidate_paths {
        if let Ok(text) = std::fs::read_to_string(path) {
            doc_text = Some((path.to_string(), text));
            break;
        }
    }
    let (doc_path, text) = doc_text.expect("docs/SUPPORTED_FUNCTIONS.md must exist");

    // Verify key functions are documented
    for key_fn in &[
        "zeros", "ones", "eye", "size", "length", "numel", "abs", "sqrt", "sin", "cos", "inv",
        "det", "mldivide", "mrdivide", "disp", "sprintf", "plot",
    ] {
        assert!(
            text.contains(key_fn),
            "docs/SUPPORTED_FUNCTIONS.md is missing key function: {key_fn}"
        );
    }

    // If UPDATE_DOCS environment variable is set, regenerate table of all registered functions
    if std::env::var("UPDATE_DOCS").as_deref() == Ok("1") {
        let mut generated = String::from("# Supported MATLAB Functions Catalog: `mlapp-rs`\n\n");
        generated.push_str("This document details the built-in function library available in the native runtime, error diagnostic behavior for unsupported functions, and the optional GNU Octave bridge.\n\n---\n\n");
        generated.push_str("## 1. Core Built-In Functions\n\n");
        generated.push_str(&format!(
            "Total registered functions: **{}**.\n\n",
            names.len()
        ));
        generated.push_str("| # | Function Name | Category | Engine |\n");
        generated.push_str("| :--- | :--- | :--- | :--- |\n");

        for (idx, name) in names.iter().enumerate() {
            let category = match name.as_str() {
                "zeros" | "ones" | "eye" | "rand" | "randn" | "linspace" | "logspace" | "magic"
                | "true" | "false" => "Generators",
                "size" | "length" | "numel" | "ndims" | "isempty" | "isscalar" | "isvector"
                | "ismatrix" | "squeeze" | "diag" => "Inspection & Shape",
                "abs" | "sign" | "sqrt" | "exp" | "log" | "log10" | "log2" | "sin" | "cos"
                | "tan" | "asin" | "acos" | "atan" | "atan2" | "round" | "floor" | "ceil"
                | "fix" | "mod" | "rem" => "Elementary Math & Trig",
                "sum" | "prod" | "cumsum" | "cumprod" | "mean" | "median" | "mode" | "std"
                | "var" | "min" | "max" | "prctile" | "quantile" | "cov" | "corrcoef"
                | "histcounts" | "movmean" | "movmedian" | "cummax" | "cummin" | "zscore" => {
                    "Statistics & Reductions"
                }
                "inv" | "det" | "mldivide" | "mrdivide" | "lu" | "qr" | "svd" | "eig" | "rank"
                | "cond" | "null" | "orth" | "pinv" | "expm" | "kron" => "Linear Algebra",
                "fft" | "ifft" | "fft2" | "ifft2" | "filter" | "filtfilt" | "hamming" | "hann"
                | "blackman" | "xcorr" => "Signal Processing",
                "rgb2gray" | "imresize" | "imfilter" => "Image Processing",
                "fitlm" | "predict" | "kmeans" | "confusionmat" => "Machine Learning",
                "rotx" | "roty" | "rotz" | "trotx" | "troty" | "trotz" | "transl"
                | "fkine_2link" | "ikine_2link" | "jtraj" => "Robotics",
                "sort" | "sortrows" | "unique" | "find" | "any" | "all" => "Sorting & Searching",
                "strcat" | "sprintf" | "fprintf" | "disp" | "num2str" | "mat2str"
                | "str2double" | "strcmp" | "strcmpi" | "strncmp" | "strncmpi" | "strfind"
                | "strrep" | "lower" | "upper" | "strtrim" | "split" | "join" | "regexp"
                | "regexprep" => "Strings & Text",
                "readmatrix" | "writematrix" | "readtable" | "writetable" | "head" | "tail"
                | "summary" => "Tables & CSV I/O",
                "uialert" | "uiconfirm" | "drawnow" | "pause" | "plot" | "scatter" | "bar"
                | "xlabel" | "ylabel" | "title" | "grid" | "hold" => "App Designer UI & Graphics",
                _ => "General Utility",
            };
            generated.push_str(&format!(
                "| {} | `{}` | {} | Native Clean-Room |\n",
                idx + 1,
                name,
                category
            ));
        }

        generated
            .push_str("\n---\n\n## 2. App Designer UI & Graphics Built-Ins (`mlapp-runtime`)\n\n");
        generated.push_str("The App Designer runtime environment provides dedicated built-in functions for interactive UI figure control and Cartesian 2D plotting:\n\n");
        generated.push_str("| Built-In | Description | Dispatched UI Mutation |\n");
        generated.push_str("| :--- | :--- | :--- |\n");
        generated.push_str(
            "| `plot` | 2D Cartesian line plotting on `UIAxes` | `UiMutation::PlotCall` |\n",
        );
        generated.push_str(
            "| `hold` | Toggles plot hold mode ('on'/'off') | `UiMutation::PlotCall` |\n",
        );
        generated
            .push_str("| `uialert` | Modal diagnostic dialog prompt | `UiMutation::Alert` |\n");
        generated.push_str(
            "| `drawnow` | Flushes pending UI mutation queue | Synchronous event flush |\n",
        );
        generated.push_str("| `pause` | Cooperatively yields execution with cancellation checks | Yield / Sleep |\n\n");
        generated.push_str("---\n\n## 3. Unsupported Function Policy\n\n");
        generated.push_str("Unsupported functions **never fail silently**. Calling an unrecognized function immediately raises a loud diagnostic:\n\n");
        generated.push_str("```\nRuntimeError: Unsupported MATLAB function '<name>' (not implemented in native runtime).\n```\n\n---\n\n");
        generated.push_str("## 4. GNU Octave Bridge (Tier 4) — NOT IMPLEMENTED\n\n");
        generated.push_str("The GNU Octave bridge is currently **NOT IMPLEMENTED** (documented stub). In current versions, unsupported functions fail loudly with a runtime error diagnostic.\n");

        std::fs::write(&doc_path, generated).expect("write updated docs/SUPPORTED_FUNCTIONS.md");
    }
}
