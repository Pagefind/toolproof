use console::style;

use crate::{ToolproofTestStep, ToolproofTestStepState};

pub fn log_step_runs(steps: &Vec<ToolproofTestStep>, indent: usize) {
    for step in steps {
        use ToolproofTestStepState::*;
        let prefix = if indent > 0 {
            format!("{: <1$}↳ ", "", indent)
        } else {
            "".to_string()
        };

        println!(
            "{prefix}{}",
            match step.state() {
                Dormant => style(format!("⦸ {step}")).dim(),
                Skipped => style(format!("⊝ {step}")).dim(),
                Failed => style(format!("✘ {step}")).red(),
                Passed => style(format!("✓ {step}")).green(),
            }
        );
        match step {
            ToolproofTestStep::Ref {
                hydrated_steps: Some(inner_steps),
                ..
            } => {
                log_step_runs(inner_steps, indent + 2);
            }
            _ => {}
        }
    }
}
