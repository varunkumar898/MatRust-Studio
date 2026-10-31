classdef SampleApp < matlab.apps.AppBase
    properties
        Button
        Label
    end
    methods
        function startup(app)
            app.Label.Text = 'Initialized';
        end
        function ButtonPushed(app, event)
            app.Label.Text = 'Clicked';
        end
    end
end
