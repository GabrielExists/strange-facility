use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use crate::core::amount::Amount;
use crate::core::job::Job;

pub struct ResourceAttributes {
    pub upkeep: Vec<Vec<(Resource, Amount)>>,
    pub visible: bool,
    // Display as a name on each row intead of reserving a column
    pub display_as_name: bool,
    pub row: usize,
}

pub type ResourceSet = BTreeMap<Resource, i64>;
pub type ResourceList = Vec<(Resource, i64)>;
pub type AttributeMappings = BTreeMap<Resource, ResourceAttributes>;

#[derive(Clone, Debug)]
pub enum CombinationResult {
    Job(Job, Option<String>),
    Text(String),
    Nothing,
}

pub struct DeltaOutput {
    pub status: DeltaOutputStatus,
    pub changed_resources: Vec<Resource>,
    pub resources_after: ResourceSet,
}


pub enum DeltaOutputStatus {
    // All requirements were fulfilled for some set
    Success {
        delta_index: usize,
    },
    // All requirements were fulfilled, and there was a change that cared about X.
    // Included is the value that was used for X
    SuccessX {
        delta_index: usize,
        x: i64,
    },
    // The requirements failed, so the entire task failed
    Failure {
        errors: Vec<String>,
        failing_resources: Vec<Resource>,
    },
}

impl DeltaOutput {
    pub fn user_messages(&self) -> Option<&Vec<String>> {
        match &self.status {
            DeltaOutputStatus::Success { .. } => None,
            DeltaOutputStatus::SuccessX { .. } => None,
            DeltaOutputStatus::Failure { errors, .. } => Some(errors),
        }
    }
    pub fn is_ok(&self) -> bool {
        self.status.is_ok()
    }
    pub fn delta_index(&self) -> usize {
        self.status.delta_index()
    }
    pub fn failing_resources(&self) -> Vec<Resource> {
        self.status.failing_resources()
    }
    pub fn is_mergeable(&self, other: &DeltaOutput) -> bool {
        match (&self.status, &other.status) {
            (
                DeltaOutputStatus::Success { delta_index: first_index },
                DeltaOutputStatus::Success { delta_index: second_index }
            ) => {
                if first_index != second_index {
                    return false;
                }
            }
            (
                DeltaOutputStatus::SuccessX { delta_index: first_index, .. },
                DeltaOutputStatus::SuccessX { delta_index: second_index, .. }
            ) => {
                if first_index != second_index {
                    return false;
                }
            }
            (DeltaOutputStatus::Failure { .. }, DeltaOutputStatus::Failure { .. }) => {}
            (_, _) => {
                return false;
            }
        }
        true
    }
}

impl DeltaOutputStatus {
    pub fn is_ok(&self) -> bool {
        match self {
            DeltaOutputStatus::Success { .. } => true,
            DeltaOutputStatus::SuccessX { .. } => true,
            DeltaOutputStatus::Failure { .. } => false,
        }
    }
    // Get which delta index
    pub fn delta_index(&self) -> usize {
        match self {
            DeltaOutputStatus::Success { delta_index } => delta_index.clone(),
            DeltaOutputStatus::SuccessX { delta_index, .. } => delta_index.clone(),
            DeltaOutputStatus::Failure { .. } => { 0 }
        }
    }
    pub fn failing_resources(&self) -> Vec<Resource> {
        match self {
            DeltaOutputStatus::Success { .. } => { Vec::new() }
            DeltaOutputStatus::SuccessX { .. } => { Vec::new() }
            DeltaOutputStatus::Failure { failing_resources, .. } => {
                failing_resources.clone()
            }
        }
    }
}

// impl Job {
//     pub fn new(button_text: &'static str, deltas: Vec<Vec<(Resource, Amount)>>, combination_resources: Vec<Resource>, id: &mut usize) -> Job {
//         let this_id = id.clone();
//         *id += 1;
//         Job {
//             long_text: button_text,
//             deltas,
//             removable: true,
//             saved: true,
//             instances: 1,
//             id: this_id,
//             combination_resources,
//         }
//     }
//     pub fn new_unsaved(button_text: &'static str, deltas: Vec<Vec<(Resource, Amount)>>, combination_resources: Vec<Resource>, id: &mut usize) -> Job {
//         let this_id = id.clone();
//         *id += 1;
//         Job {
//             long_text: button_text,
//             deltas,
//             removable: true,
//             saved: false,
//             instances: 1,
//             id: this_id,
//             combination_resources,
//         }
//     }
// }

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Resource {
    Coin,
    Fragment,
    DayDream,
    GlassBottle,
    Dream,
    SoothingMemory,
    ComfortDream,
    ScaryFragment,
    Nightmare,
}

pub fn attributes() -> AttributeMappings {
    BTreeMap::from([
    ])
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::Coin => f.write_str("Coin"),
            Resource::Fragment => f.write_str("Fragment"),
            Resource::DayDream => f.write_str("Day Dream"),
            Resource::GlassBottle => f.write_str("Glass bottle"),
            Resource::Dream => f.write_str("Dream"),
            Resource::SoothingMemory => f.write_str("Soothing memory"),
            Resource::ComfortDream => f.write_str("Comfort dream"),
            Resource::ScaryFragment => f.write_str("Scary fragment"),
            Resource::Nightmare => f.write_str("Nightmare"),
        }
    }
}

