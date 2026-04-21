/**
@module SPECIAL.CLI.STATUS
Shared interactive status and progress reporting for CLI commands.
*/
// @fileimplements SPECIAL.CLI.STATUS
use std::io::{self, IsTerminal};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(super) struct StatusStep {
    pub phase: &'static str,
    pub weight: u32,
}

impl StatusStep {
    pub(super) const fn new(phase: &'static str, weight: u32) -> Self {
        Self { phase, weight }
    }
}

pub(super) struct CommandStatus {
    label: &'static str,
    enabled: bool,
    started_at: Instant,
    plan: Option<&'static [StatusStep]>,
}

impl CommandStatus {
    pub(super) fn with_plan(label: &'static str, plan: &'static [StatusStep]) -> Self {
        Self {
            label,
            enabled: io::stderr().is_terminal(),
            started_at: Instant::now(),
            plan: Some(plan),
        }
    }

    pub(super) fn phase(&self, phase: &str) {
        if self.enabled {
            if let Some(progress) = self.phase_progress(phase) {
                eprintln!(
                    "{}: {}... ({}{})",
                    self.label,
                    phase,
                    progress.percent,
                    progress
                        .eta
                        .map(|eta| format!(", est. {} remaining", format_duration(eta)))
                        .unwrap_or_default()
                );
            } else {
                eprintln!("{}: {}...", self.label, phase);
            }
        }
    }

    pub(super) fn note(&self, message: &str) {
        if self.enabled {
            eprintln!("{}: {}", self.label, message);
        }
    }

    pub(super) fn notifier(&self) -> impl Fn(&str) + 'static {
        let label = self.label;
        let enabled = self.enabled;
        move |message| {
            if enabled {
                eprintln!("{label}: {message}");
            }
        }
    }

    pub(super) fn finish(&self) {
        if self.enabled {
            eprintln!("{}: done in {:.1?}", self.label, self.started_at.elapsed());
        }
    }

    fn phase_progress(&self, phase: &str) -> Option<PhaseProgress> {
        let plan = self.plan?;
        let phase_index = plan.iter().position(|step| step.phase == phase)?;
        let total_weight: u32 = plan.iter().map(|step| step.weight).sum();
        if total_weight == 0 {
            return None;
        }

        let completed_weight: u32 = plan.iter().take(phase_index).map(|step| step.weight).sum();
        let percent = ((completed_weight * 100) / total_weight) as usize;
        let eta = if completed_weight == 0 {
            None
        } else {
            let remaining_weight = total_weight.saturating_sub(completed_weight);
            if remaining_weight == 0 {
                Some(Duration::ZERO)
            } else {
                Some(
                    self.started_at
                        .elapsed()
                        .mul_f64(remaining_weight as f64 / completed_weight as f64),
                )
            }
        };

        Some(PhaseProgress { percent, eta })
    }
}

struct PhaseProgress {
    percent: usize,
    eta: Option<Duration>,
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs_f64() >= 10.0 {
        format!("{:.0}s", duration.as_secs_f64())
    } else if duration.as_secs_f64() >= 1.0 {
        format!("{:.1}s", duration.as_secs_f64())
    } else {
        format!("{:.0}ms", duration.as_secs_f64() * 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandStatus, StatusStep};

    const PLAN: &[StatusStep] = &[
        StatusStep::new("first", 1),
        StatusStep::new("second", 3),
        StatusStep::new("third", 1),
    ];

    #[test]
    fn planned_phase_reports_percentages() {
        let status = CommandStatus::with_plan("special test", PLAN);
        let first = status
            .phase_progress("first")
            .expect("first phase progress");
        let second = status
            .phase_progress("second")
            .expect("second phase progress");
        let third = status
            .phase_progress("third")
            .expect("third phase progress");

        assert_eq!(first.percent, 0);
        assert_eq!(second.percent, 20);
        assert_eq!(third.percent, 80);
    }

    #[test]
    fn unknown_phase_has_no_progress() {
        let status = CommandStatus::with_plan("special test", PLAN);
        assert!(status.phase_progress("missing").is_none());
    }
}
