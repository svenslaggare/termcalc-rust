use tempfile::NamedTempFile;

use crate::calculator::{CalculationError, CalculationResult};
use crate::execution::engine::{ExecutionEngine, ExecutionError};
use crate::model::ResultValue;
use crate::number_helpers::float_range;

pub struct GnuPlotter<'a> {
    function: &'a str,
    start: ResultValue,
    end: ResultValue,
    step: Option<ResultValue>
}

impl<'a> GnuPlotter<'a> {
    pub fn new(
        function: &'a str,
        start: ResultValue,
        end: ResultValue,
        step: Option<ResultValue>
    ) -> GnuPlotter<'a> {
        GnuPlotter {
            function,
            start,
            end,
            step,
        }
    }

    pub fn plot(&self, execution: &ExecutionEngine) -> CalculationResult<String> {
        let function = execution.environment().get_function(&self.function)
            .ok_or_else(|| ExecutionError::FunctionNotFound(self.function.to_owned()))?;

        let mut plot_data_file = NamedTempFile::with_suffix(".csv").map_err(|_| CalculationError::InternalError)?;

        match (&self.start, &self.end) {
            (ResultValue::Float(start), ResultValue::Float(end)) => {
                let step = self.step.as_ref().map(|x| x.as_f64()).flatten().unwrap_or((end - start) / 100.0);
                for (x, y) in execution.execute_function(function, float_range(*start, *end, step).map(|x| [ResultValue::Float(x)]))? {
                    plot_data_file.write_all(format!("{};{}\n", x[0], y).as_bytes()).map_err(|_| CalculationError::InternalError)?;
                }
            }
            (ResultValue::Int(start), ResultValue::Int(end)) => {
                let step = self.step.as_ref().map(|x| x.as_i64()).flatten().unwrap_or(1);
                for (x, y) in execution.execute_function(function, (*start..*end + step).map(|x| [ResultValue::Int(x)]))? {
                    plot_data_file.write_all(format!("{};{}\n", x[0], y).as_bytes()).map_err(|_| CalculationError::InternalError)?;
                }
            }
            _ => {
                Err(ExecutionError::MixedTypes)?;
            }
        }

        let plot_definition = r#"
        set datafile separator ";"
        set xlabel "X"
        set ylabel "Y"
        set terminal wxt size 800,600
        plot ""#.to_owned() + &plot_data_file.path().display().to_string() + r#"" using 1:2 with linespoints title "Plot"
        "#;

        use std::io::Write;

        let mut plot_file = NamedTempFile::new().map_err(|_| CalculationError::InternalError)?;
        plot_file.write_all(plot_definition.as_bytes()).map_err(|_| CalculationError::InternalError)?;

        let mut command = std::process::Command::new("gnuplot")
            .arg(plot_file.path())
            .arg("-p")
            .spawn()
            .map_err(|_| CalculationError::GnuPlotNotFound)?;
        command.wait().map_err(|_| CalculationError::InternalError)?;

        Ok(String::new())
    }
}
