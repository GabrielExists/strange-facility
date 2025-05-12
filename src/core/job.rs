use strange_facility::jobs::{DeltaOutput, DeltaOutputStatus, ResourceSet};
use crate::core::amount::Amount;
use crate::core::resource::{attributes, DeltaOutput, DeltaOutputStatus, Resource, ResourceSet};

#[derive(Clone, Debug)]
pub struct Job {
    // Text shown on the button
    pub short_text: &'static str,
    // Description
    pub long_text: &'static str,
    // How resources change at the start of this job
    pub start_deltas: Vec<Vec<(Resource, Amount)>>,
    // How resources change at the end of this job
    pub end_deltas: Vec<Vec<(Resource, Amount)>>,
    // Does this show up in the quick list of jobs?
    pub saved: bool,
    // How many time slots do you need to do to finish this job?
    pub total_time_slots: usize,
    // Other jobs with the same id are considered to be the exact same
    pub id: JobId,
}

pub struct JobOutput {
    pub main_output: DeltaOutput,
    pub upkeep_outputs: Vec<(Resource, DeltaOutput)>,
    pub resources_after: ResourceSet,
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

pub fn apply_job(orig_resources: ResourceSet, job: &Job) -> Result<JobOutput, String> {
    let mut upkeep_outputs = Vec::new();
    let main_output = apply_deltas(orig_resources, &job.end_deltas, 1)?;
    let mut resources = main_output.resources_after.clone();
    for (current_resource, attribute) in attributes() {
        if let Some(num) = resources.get(&current_resource) {
            let num = *num;
            if num > 0 {
                let delta_output = apply_deltas(resources, &attribute.upkeep, num * 1)?;
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
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum JobId {
    FragmentCatch,
    DayDreamCraft,
    DayDreamSell,
    BottleBuy,
    DreamCraft,
    DreamSell,
    DreamUse,
    ComfortDreamCraft,
    ComfortDreamSell,
    NightmareCraft,
    NightmareSell,
    Retire,
}

pub fn starting_resources() -> Job{
    Job {
        short_text: "Starting resources",
        long_text: "",
        start_deltas: vec![],
        end_deltas: vec![],
        saved: false,
        total_time_slots: 0,
        id: JobId::FragmentCatch,
    }
}

pub fn create_job(job_id: JobId) -> Job {
    Job {
        short_text: "",
        long_text: "",
        start_deltas: vec![],
        end_deltas: vec![],
        saved: false,
        total_time_slots: 0,
        id: JobId::FragmentCatch,
    }
    // match job_id {
    //     JobId::FragmentCatch => {Job {
    //         short_text: "",
    //         long_text: "",
    //         start_deltas: vec![],
    //         end_deltas: vec![],
    //         saved: false,
    //         total_time_slots: 0,
    //         id: JobId::FragmentCatch,
    //     };}
    //     JobId::DayDreamCraft => {}
    //     JobId::DayDreamSell => {}
    //     JobId::BottleBuy => {}
    //     JobId::DreamCraft => {}
    //     JobId::DreamSell => {}
    //     JobId::DreamUse => {}
    //     JobId::ComfortDreamCraft => {}
    //     JobId::ComfortDreamSell => {}
    //     JobId::NightmareCraft => {}
    //     JobId::NightmareSell => {}
    //     JobId::Retire => {}
    // }
}

pub const WIN_JOB_ID: JobId = JobId::Retire;
