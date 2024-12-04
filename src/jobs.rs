#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::vec;
use crate::game::attributes;


#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Amount {
    Gain(i64),
    Spend(i64),
    Catalyst(i64),
    GainX(i64),
    SpendX(i64),
    CatalystX(i64),
    Set(i64),
}

impl Amount {
    pub fn multiply(self, factor: i64) -> Self {
        match self {
            Amount::Gain(delta) => Amount::Gain(delta * factor),
            Amount::Spend(delta) => Amount::Spend(delta * factor),
            Amount::Catalyst(delta) => Amount::Catalyst(delta * factor),
            Amount::GainX(delta) => Amount::GainX(delta * factor),
            Amount::SpendX(delta) => Amount::SpendX(delta * factor),
            Amount::CatalystX(delta) => Amount::CatalystX(delta * factor),
            Amount::Set(target) => Amount::Set(target * factor),
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Resource {
    Player,
    Fish,
    FoodRation,
    Scrap,
    SpareParts,
    Submarine,
    Net,
    NetUpgraded,
    Claw,
    ClawUpgraded,
    Martha,
    MarthaAtWork,
    Forge,
}

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
pub struct Job {
    // Text on the button
    pub button_text: &'static str,
    // How resources change when this is applied
    pub deltas: Vec<Vec<(Resource, Amount)>>,
    // Can this be removed by the user?
    pub removable: bool,
    // Does this show up in the quick list of jobs?
    pub saved: bool,
    // How many sets of jobs are collapsed into this item
    pub instances: usize,
    // Other jobs with the same id are considered to be the exact same
    pub id: usize,
    pub combination_resources: Vec<Resource>,
}

#[derive(Clone, Debug)]
pub enum CombinationResult {
    Job(Job, Option<String>),
    Text(String),
    Nothing,
}

impl Job {
    pub fn new(button_text: &'static str, deltas: Vec<Vec<(Resource, Amount)>>, combination_resources: Vec<Resource>, id: &mut usize) -> Job {
        let this_id = id.clone();
        *id += 1;
        Job {
            button_text,
            deltas,
            removable: true,
            saved: true,
            instances: 1,
            id: this_id,
            combination_resources,
        }
    }
    pub fn new_unsaved(button_text: &'static str, deltas: Vec<Vec<(Resource, Amount)>>, combination_resources: Vec<Resource>, id: &mut usize) -> Job {
        let this_id = id.clone();
        *id += 1;
        Job {
            button_text,
            deltas,
            removable: true,
            saved: false,
            instances: 1,
            id: this_id,
            combination_resources,
        }
    }
    pub fn starting_resources() -> Job {
        Job {
            button_text: "Explore the facility",
            deltas: vec![vec![
                (Resource::Player, Amount::Gain(1)),
                (Resource::FoodRation, Amount::Gain(7)),
                (Resource::Martha, Amount::Gain(1)),
                (Resource::Forge, Amount::Gain(1)),
                (Resource::Net, Amount::Gain(1)),
                (Resource::Submarine, Amount::Gain(1)),
                (Resource::Claw, Amount::Gain(1)),
            ]],
            removable: false,
            saved: false,
            instances: 1,
            id: 0,
            combination_resources: vec![],
        }
    }
}

pub struct JobOutput {
    pub main_output: DeltaOutput,
    pub upkeep_outputs: Vec<(Resource, DeltaOutput)>,
    pub resources_after: ResourceSet,
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

impl JobOutput {
    pub fn user_message(&self) -> Option<&str> {
        self.main_output.user_messages().map(|list| list.first()).unwrap_or(None).map(|string| string.as_str())
    }
    pub fn is_ok(&self) -> bool {
        if !self.main_output.is_ok() {
            return false;
        }
        for (_resource, output) in self.upkeep_outputs.iter() {
            if !output.status.is_ok() {
                return false;
            }
        }
        true
    }
    pub fn delta_index(&self) -> usize {
        self.main_output.delta_index()
    }

    pub fn failing_resources(&self) -> Vec<Resource> {
        let mut failing_resources = self.main_output.failing_resources();
        for (_upkeep_resource, upkeep_output) in self.upkeep_outputs.iter() {
            failing_resources.append(&mut upkeep_output.failing_resources());
        }
        failing_resources
    }
    pub fn is_mergeable(&self, other: &Self) -> bool {
        return (self.is_ok() && other.is_ok()) || (!self.is_ok() && !other.is_ok());
        // if !self.main_output.is_mergeable(&other.main_output) {
        //     return false;
        // }
        // if self.upkeep_outputs.len() != other.upkeep_outputs.len() {
        //     return false;
        // }
        // for (
        //     (first_resource, first_output),
        //     (second_resource, second_output)
        // ) in zip(self.upkeep_outputs.iter(), other.upkeep_outputs.iter()) {
        //     if first_resource != second_resource {
        //         return false;
        //     }
        //     if !first_output.is_mergeable(&second_output) {
        //         return false;
        //     }
        // }
        // true
    }
    pub fn get_changed_resources(&self) -> Vec<Resource> {
        let mut total = Vec::new();
        total.extend(self.main_output.changed_resources.iter());
        for (_resource, output) in self.upkeep_outputs.iter() {
            total.extend(output.changed_resources.iter());
        }
        total
    }
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
    pub(crate) fn is_mergeable(&self, other: &DeltaOutput) -> bool {
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

pub fn apply_job(orig_resources: ResourceSet, job: &Job) -> Result<JobOutput, String> {
    let mut upkeep_outputs = Vec::new();
    let main_output = apply_deltas(orig_resources, &job.deltas, job.instances as i64)?;
    let mut resources = main_output.resources_after.clone();
    for (current_resource, attribute) in attributes() {
        if let Some(num) = resources.get(&current_resource) {
            let num = *num;
            if num > 0 {
                let delta_output = apply_deltas(resources, &attribute.upkeep, num * job.instances as i64)?;
                resources = delta_output.resources_after.clone();
                upkeep_outputs.push((current_resource, delta_output));
            }
        }
    }
    Ok(JobOutput {
        main_output,
        upkeep_outputs,
        resources_after: resources,
    })
}

pub fn apply_deltas(mut resources: ResourceSet, deltas: &Vec<Vec<(Resource, Amount)>>, multiplier: i64) -> Result<DeltaOutput, String> {
    if multiplier == 0 {
        return Err("Zero multiplier".to_string());
    }
    if multiplier < 0 {
        return Err("Negative multiplier".to_string());
    }
    let deltas = deltas.into_iter()
        .map(|delta| {
            delta.into_iter()
                .map(|(delta_resource, delta_amount)| {
                    (delta_resource.clone(), delta_amount.multiply((multiplier).clone()))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut x = None;
    let mut errors = Vec::new();
    let mut failing_resources = Vec::new();
    let mut changed_resources = Vec::new();
    let mut delta_index = 0;
    let mut delta = None;
    for (new_delta_index, current_delta) in deltas.iter().enumerate() {
        errors.clear();
        failing_resources.clear();
        delta_index = new_delta_index;
        delta = Some(current_delta);
        for (resource, amount) in current_delta.iter() {
            let current_amount = resources.entry(resource.clone()).or_insert(0);
            match amount {
                Amount::Gain(_) => {}
                Amount::GainX(_) => {}
                Amount::Spend(delta) |
                Amount::Catalyst(delta) => {
                    if delta > current_amount {
                        errors.push(format!("Not enough {}", resource));
                        failing_resources.push(resource.clone())
                    }
                }
                Amount::SpendX(delta) |
                Amount::CatalystX(delta) => {
                    if *current_amount < *delta {
                        failing_resources.push(resource.clone());
                    }
                    if let Some(new_x) = current_amount.checked_div(delta.clone()) {
                        match &x {
                            Some(current_x) => {
                                if new_x < *current_x {
                                    x = Some(new_x);
                                }
                            }
                            None => {
                                x = Some(new_x);
                            }
                        }
                    } else {
                        if delta < current_amount {
                            errors.push(format!("This button is configured incorrectly, with a zero quantity for resource {:?}.", current_amount));
                        }
                    }
                }
                Amount::Set(_) => {}
            }
        }
        if errors.is_empty() {
            break;
        }
    }
    if let Some(delta) = delta {
        for (resource, amount) in delta.into_iter() {
            let current_resource = resources.entry(resource.clone()).or_insert(0);
            match amount {
                Amount::Gain(delta) => {
                    if *delta != 0 {
                        changed_resources.push(resource.clone());
                    }
                    *current_resource += delta;
                }
                Amount::Spend(delta) => {
                    if *delta != 0 {
                        changed_resources.push(resource.clone());
                    }
                    *current_resource -= delta;
                }
                Amount::Catalyst(_) => {}
                Amount::GainX(delta_per) => {
                    if *delta_per != 0 {
                        changed_resources.push(resource.clone());
                    }
                    *current_resource += delta_per * x.unwrap_or(0);
                }
                Amount::SpendX(delta_per) => {
                    if *delta_per != 0 {
                        changed_resources.push(resource.clone());
                    }
                    *current_resource -= delta_per * x.unwrap_or(0);
                }
                Amount::CatalystX(_delta_per) => {}
                Amount::Set(target) => {
                    changed_resources.push(resource.clone());
                    *current_resource -= target;
                }
            }
        }
    }
    let status = if errors.is_empty() {
        match x {
            Some(x) => {
                DeltaOutputStatus::SuccessX {
                    delta_index,
                    x,
                }
            }
            None => {
                DeltaOutputStatus::Success {
                    delta_index,
                }
            }
        }
    } else {
        DeltaOutputStatus::Failure {
            errors,
            failing_resources,
        }
    };
    Ok(DeltaOutput {
        status,
        changed_resources,
        resources_after: resources,
    })
    // if delta.is_negative() {
    //     if *current_resource > delta as u64 {
    //         *current_resource += delta as u64;
    //     } else {
    //         self.user_error = Some(format!("Not enough {}, {}/{}", resource, current_resource, -delta));
    //     }
    // } else {
    //     *current_resource += delta as u64;
    // }
}

// pub fn apply_jobs(mut resources: Resources, jobs: Vec<Job>) -> Result<Resources, String> {
//     for job in jobs {
//         resources = apply_job(resources, job)?;
//     }
//     Ok(resources)
// }
//
//
// pub fn apply_jobs_partial(mut resources: Resources, jobs: Vec<Job>) -> Resources {
//     for job in jobs {
//         match apply_job(resources.clone(), job) {
//             Ok(new_resources) => {
//                 resources = new_resources;
//             }
//             Err(_) => {}
//         }
//     }
//     resources
// }
//
// pub fn get_job_success_zip(resources: Resources, jobs: Vec<Job>) -> Vec<(Job, bool)> {
//     let mut results = Vec::new();
//     for index in 0..jobs.len() {
//         let previous_resources = apply_jobs_partial(resources.clone(), jobs[0..index].to_vec());
//         let job = jobs.get(index).expect("This index should be valid.").clone();
//         let success = apply_job(previous_resources, job.clone()).is_ok();
//         results.push((job, success));
//     }
//     results
// }

