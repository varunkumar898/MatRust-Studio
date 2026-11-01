% GENERATE_SAMPLE_MODELS.M
% Generates clean-room synthetic Simulink test models at runtime using standard
% Simulink scripting APIs (new_system, add_block, add_line, set_param, save_system).
% No proprietary MathWorks model binaries are committed to the repository.

function generate_sample_models(output_dir)
    if nargin < 1
        output_dir = fullfile(fileparts(mfilename('fullpath')), 'generated');
    end
    if ~exist(output_dir, 'dir')
        mkdir(output_dir);
    end

    %% 1. Simple Feedback Oscillator / First-Order Model
    model1 = 'simple_feedback';
    model1_path = fullfile(output_dir, [model1, '.slx']);
    
    bdclose('all');
    new_system(model1);
    open_system(model1);
    
    % Add core blocks
    add_block('simulink/Sources/Sine Wave', [model1, '/SineWave'], 'Position', [50, 100, 80, 130]);
    add_block('simulink/Math Operations/Sum', [model1, '/Sum'], 'Position', [140, 100, 170, 130], 'Inputs', '+-');
    add_block('simulink/Math Operations/Gain', [model1, '/Gain'], 'Position', [220, 100, 250, 130], 'Gain', '2.0');
    add_block('simulink/Continuous/Integrator', [model1, '/Integrator'], 'Position', [300, 100, 330, 130], 'InitialCondition', '0.0');
    add_block('simulink/Sinks/Out1', [model1, '/Out1'], 'Position', [380, 105, 410, 125]);
    
    % Add lines
    add_line(model1, 'SineWave/1', 'Sum/1');
    add_line(model1, 'Sum/1', 'Gain/1');
    add_line(model1, 'Gain/1', 'Integrator/1');
    add_line(model1, 'Integrator/1', 'Out1/1');
    % Feedback loop: Integrator output back to Sum port 2
    add_line(model1, 'Integrator/1', 'Sum/2');
    
    % Solver settings: Fixed-step Euler
    set_param(model1, 'SolverType', 'Fixed-step');
    set_param(model1, 'Solver', 'FixedStepDiscrete');
    set_param(model1, 'FixedStep', '0.01');
    set_param(model1, 'StopTime', '10.0');
    
    save_system(model1, model1_path);
    close_system(model1);
    
    fprintf('Generated model: %s\n', model1_path);
end
