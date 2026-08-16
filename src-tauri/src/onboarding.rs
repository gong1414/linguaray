//! First-launch onboarding step reducer.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    Welcome,
    Accessibility,
    Provider,
    History,
    Shortcuts,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingEvent {
    Start,
    Continue,
    Skip,
    Complete,
}

pub fn next_step(current: OnboardingStep, event: OnboardingEvent) -> OnboardingStep {
    use OnboardingEvent::*;
    use OnboardingStep::*;
    match (current, event) {
        (Welcome, Start | Continue) => Accessibility,
        (Accessibility, Continue | Skip) => Provider,
        (Provider, Continue | Skip) => History,
        (History, Continue | Skip) => Shortcuts,
        (Shortcuts, Continue | Skip | Complete) => Done,
        (Done, _) => Done,
        (step, Complete) => {
            if step == Welcome {
                Accessibility
            } else {
                Done
            }
        }
        (step, _) => step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_welcome_to_done() {
        let mut step = OnboardingStep::Welcome;
        step = next_step(step, OnboardingEvent::Start);
        assert_eq!(step, OnboardingStep::Accessibility);
        step = next_step(step, OnboardingEvent::Skip);
        assert_eq!(step, OnboardingStep::Provider);
        step = next_step(step, OnboardingEvent::Continue);
        assert_eq!(step, OnboardingStep::History);
        step = next_step(step, OnboardingEvent::Continue);
        assert_eq!(step, OnboardingStep::Shortcuts);
        step = next_step(step, OnboardingEvent::Complete);
        assert_eq!(step, OnboardingStep::Done);
        assert_eq!(next_step(step, OnboardingEvent::Start), OnboardingStep::Done);
    }
}
