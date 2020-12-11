use std::collections::{HashMap, HashSet};

use crate::types::AircraftState;

mod ffac;
mod flta;
mod mode_1;
mod mode_2;
mod mode_3;
mod mode_4;
mod mode_5;
mod pda;

pub use ffac::*;
pub use flta::*;
pub use mode_1::*;
pub use mode_2::*;
pub use mode_3::*;
pub use mode_4::*;
pub use mode_5::*;
pub use pda::*;

/// Available alerts from the TAWS.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
#[cfg_attr(feature = "wasi", derive(serde::Serialize))]
#[derive(strum::EnumString)]
pub enum Alert {
    /// Forward Lookig Terrain Avoidance
    FLTA,

    /// Five Hundred foot altitude Callout
    FFAC,

    /// Premature Descent Alerting
    PDA,

    /// Excessive Rate of Descent
    Mode1,

    /// Excessive ClosureRate to Terrain
    Mode2,

    /// Negative Climb Rate or Altitude Loss after Take-off or Go Around
    Mode3,

    /// Flight Near Terrain when Not in Landing Configuration
    Mode4,

    /// Excessive Downward Deviation from an ILS Glideslope or LPV/GLS Glidepath
    Mode5,
    // TODO add more
}
impl Eq for Alert {}

/// Importance level of an alert
///
/// Orderd by high priority to low priority in ascending order
#[derive(Clone, Copy, Debug, PartialEq, Hash, strum::EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum AlertLevel {
    /// The level or category of alert for conditions that require immediate flight crew awareness
    /// and immediate flight crew response.
    Warning,

    /// The level or category of alert for conditions that require immediate flight crew awareness
    /// and a less urgent subsequent flight crew response than a warning alert.  
    Caution,
}
impl Eq for AlertLevel {}

/// Collection of a all alerts which are currently present in the TAWS
#[derive(Debug, Default, PartialEq)]
pub struct AlertState {
    /// Alerts which are to be displayed to the crew
    pub alerts: HashMap<Alert, AlertLevel>,

    /// Alerts which are not to be disclosed to the crew to avoid nuisance
    pub nuisance_alerts: HashMap<Alert, AlertLevel>,
}

impl AlertState {
    pub fn alerts_total_count(&self) -> usize {
        self.alerts.len() + self.nuisance_alerts.len()
    }

    pub fn alerts_count(&self, level: AlertLevel) -> usize {
        self.alerts.values().filter(|v| **v == level).count()
    }

    pub fn mode_alert_level(&self, alert_system: Alert) -> Option<AlertLevel> {
        self.alerts.get(&alert_system).cloned()
    }

    /// updates internal alerts with new alerts, removing all old alerts. Prioritizes as well.
    pub(crate) fn insert(&mut self, alert: Alert, alert_level: AlertLevel) {
        if let Some(old_alert_level) = self.alerts.get_mut(&alert) {
            if (alert_level as isize) < (*old_alert_level as isize) {
                self.nuisance_alerts.insert(alert, *old_alert_level);
                *old_alert_level = alert_level;
            }
        } else {
            self.alerts.insert(alert, alert_level);
        }
    }
}

/// Trait which is to be fulfilled by all functionalities
pub trait AlertSystem: std::fmt::Debug + Send {
    /// Returns whether this alarm is armed.
    ///
    /// Arming refers to the automatic switching on of a function by
    /// the Equipment (DO-367 Chapter 1.9).
    fn is_armed(&self) -> bool;

    /// Dismiss this alert
    fn inhibit(&mut self);

    /// Enable this alert
    fn uninhibit(&mut self);

    /// Returns whether this alarm is inhibited
    fn is_inhibited(&self) -> bool;

    /// Process a new AircraftState, emit alerts if appropiate
    fn process(&mut self, state: &AircraftState) -> Option<AlertLevel>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn strum_test() {
        let mut key = String::from("Mode 1");
        key.retain(|c| !c.is_whitespace());

        let _: Alert = key
            .parse()
            .expect(&format!("Unable to parse {} as Alert ", key));
    }

    #[test]
    pub fn alert_state_propagates_alerts() {
        let mut alert_state = AlertState::default();
        let test_alerts = vec![(Alert::Mode3, AlertLevel::Caution)];
        for (new_alert, new_alert_level) in test_alerts.iter() {
            alert_state.insert(*new_alert, *new_alert_level);
        }

        assert_eq!(test_alerts.len(), alert_state.alerts_total_count())
    }

    #[test]
    pub fn alert_state_usage() {
        let alts = AlertState::default();

        // using hashset contains
        if alts.alerts.get(&Alert::Mode1) == Some(&AlertLevel::Caution) {
            // do important stuff
        }

        // using hashset any
        if alts.alerts.iter().any(|(k, v)| *v == AlertLevel::Caution) {
            // do important stuff
        }

        // using match
        for x in &alts.alerts {
            match x {
                (Alert::Mode1, AlertLevel::Caution) if 1 > 0 => {}

                (Alert::Mode1, AlertLevel::Caution) => {}
                (Alert::Mode1, AlertLevel::Warning) => {}
                _ => {}
            }
        }

        // using match lazily
        for x in &alts.alerts {
            match x {
                (_, AlertLevel::Caution) if 1 > 0 => {}
                _ => {}
            }
        }
    }
}
