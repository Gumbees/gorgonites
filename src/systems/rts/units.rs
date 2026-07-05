//! Shared unit vocabulary for the RTS layer.
//!
//! Concrete unit stats, era-scaled names, and the live unit struct now live
//! in `crate::game::entities`; this module keeps the small shared enums.

/// High-level behavioural state a unit can be in (used for display/AI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitState {
    Idle,
    Moving,
    Attacking,
    Gathering,
    Building,
    Dead,
}
