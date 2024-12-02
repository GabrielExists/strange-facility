#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
// use std::iter::zip;
use std::vec;


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

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::Player => {
                f.write_str("Player")
            }
            Resource::Scrap => {
                f.write_str("Scrap")
            }
            Resource::SpareParts => {
                f.write_str("Spare parts")
            }
            Resource::Fish => {
                f.write_str("Fish")
            }
            Resource::FoodRation => {
                f.write_str("Food ration")
            }
            Resource::Submarine => {
                f.write_str("Sub")
            }
            Resource::Net => {
                f.write_str("Net")
            }
            Resource::NetUpgraded => {
                f.write_str("Net+")
            }
            Resource::Claw => {
                f.write_str("Claw")
            }
            Resource::ClawUpgraded => {
                f.write_str("Claw+")
            }
            Resource::Martha => {
                f.write_str("Martha")
            }
            Resource::MarthaAtWork => {
                f.write_str("WorkingMartha")
            }
            Resource::Forge => {
                f.write_str("Forge")
            }
            // Resource::ForgeGlowing => {
            //     f.write_str("Glowing forge")
            // }
        }
    }
}

macro_rules! both {
    ($A:ident, $B:ident) => {
        (Resource::$A, Resource::$B) | (Resource::$B, Resource::$A)
    };
}

pub const WIN_JOB_ID: usize = 1;

impl Resource {
    pub fn long_name(&self) -> &'static str {
        match self {
            Resource::Player => { "Player" }
            Resource::Scrap => { "Scrap metal" }
            Resource::SpareParts => { "Spare Parts" }
            Resource::Fish => { "Fish" }
            Resource::FoodRation => { "Food Ration" }
            Resource::Submarine => { "Submarine" }
            Resource::Net => { "Fishing net" }
            Resource::NetUpgraded => { "Improved fishing net" }
            Resource::Martha => { "Martha, the canning machine" }
            Resource::Forge => { "An ancient forge" }
            Resource::Claw => { "A huge mechanical claw" }
            Resource::ClawUpgraded => { "An upgraded mechanical claw" }
            // Resource::ForgeGlowing => { "A glowing forge" }
            Resource::MarthaAtWork => { "Martha, currently canning" }
        }
    }
    // fn description(&self) -> &'static str {
    //     match self {
    //         Resource::Player => { "This is you, you have needs." }
    //         Resource::Scrap => { "Various metal scraps that could perhaps be forged into something." }
    //         _ => { "" }
    //     }
    // }

    pub fn combine(&self, other: &Self) -> CombinationResult {
        match (self, other) {
            both!(SpareParts, Submarine) => {
                // This has to stay in sync with the WIN_JOB_ID
                let mut id = 1;
                CombinationResult::Job(
                    Job::new("Make the submarine worthy of the deep seas, load it up with food, and get out of this place. (Win the game)", vec![vec![
                        (Resource::Submarine, Amount::Catalyst(1)),
                        (Resource::FoodRation, Amount::Spend(20)),
                        (Resource::SpareParts, Amount::Spend(20)),
                    ]], vec![self.clone(), other.clone()], &mut id),
                    Some("With some trepidation, you set off into the dark depths.".to_string()),
                )
            }
            both!(Submarine, Net) |
            both!(Submarine, NetUpgraded) => {
                let mut id = 2;
                CombinationResult::Job(
                    Job::new("Collect the ocean's bounty, by submarine and net", vec![
                        vec![
                            (Resource::NetUpgraded, Amount::Catalyst(1)),
                            (Resource::Fish, Amount::Gain(20)),
                        ],
                        vec![
                            (Resource::Net, Amount::Catalyst(1)),
                            (Resource::Fish, Amount::Gain(10)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("There are yet edible creatures to find in these desolate waters.".to_string()),
                )
            }
            both!(Fish, Martha) => {
                let mut id = 3;
                CombinationResult::Job(
                    Job::new("Operate Martha, the canning machine", vec![vec![
                        (Resource::Martha, Amount::Spend(1)),
                        (Resource::MarthaAtWork, Amount::Gain(1)),
                        (Resource::Fish, Amount::Catalyst(15)),
                    ]], vec![self.clone(), other.clone()], &mut id),
                    Some("Martha wakes up and gets to work, continuously cooking and canning.".to_string()),
                )
            }
            both!(Submarine, Claw) |
            both!(Submarine, ClawUpgraded) => {
                let mut id = 4;
                CombinationResult::Job(
                    Job::new("Gather metal off of derelict metal structures", vec![
                        vec![
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Gain(6)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Gain(3)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("You gingerly manoeuvre the submarine and let the claw rend the submerged shapes.".to_string()),
                )
            }
            both!(Claw, Forge) |
            both!(ClawUpgraded, Forge) => {
                let mut id = 5;
                CombinationResult::Job(
                    Job::new_unsaved("Melt the claw for parts", vec![
                        vec![
                            (Resource::ClawUpgraded, Amount::Spend(1)),
                            (Resource::SpareParts, Amount::Gain(3)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Spend(1)),
                            (Resource::SpareParts, Amount::Gain(2)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("The forge eagerly consumes the sacrifice.".to_string()),
                )
            }
            both!(Claw, Martha) |
            both!(Claw, MarthaAtWork) |
            both!(ClawUpgraded, Martha) |
            both!(ClawUpgraded, MarthaAtWork) => {
                let mut id = 6;
                CombinationResult::Job(
                    Job::new_unsaved("Viciously murder Martha", vec![
                        vec![
                            (Resource::MarthaAtWork, Amount::Spend(1)),
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Gain(15)),
                        ],
                        vec![
                            (Resource::Martha, Amount::Spend(1)),
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Gain(15)),
                        ],
                        vec![
                            (Resource::MarthaAtWork, Amount::Spend(1)),
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Gain(15)),
                        ],
                        vec![
                            (Resource::Martha, Amount::Spend(1)),
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Gain(15)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("You gut your loyal companion for parts.".to_string()),
                )
            }
            both!(Scrap, Forge) => {
                let mut id = 7;
                CombinationResult::Job(
                    Job::new("Toil in the forge", vec![vec![
                        (Resource::Forge, Amount::Catalyst(1)),
                        (Resource::Scrap, Amount::Spend(10)),
                        (Resource::SpareParts, Amount::Gain(2)),
                    ]], vec![self.clone(), other.clone()], &mut id),
                    Some("The age-old installation grumbles, but tolerates your presence.".to_string()),
                )
            }
            both!(SpareParts, Claw) => {
                let mut id = 8;
                CombinationResult::Job(
                    Job::new_unsaved("Improve the claw", vec![vec![
                        (Resource::Claw, Amount::Spend(1)),
                        (Resource::ClawUpgraded, Amount::Gain(1)),
                        (Resource::SpareParts, Amount::Spend(2)),
                    ]], vec![self.clone(), other.clone()], &mut id),
                    Some("The claw looks better than ever.".to_string()),
                )
            }
            both!(SpareParts, Net) => {
                let mut id = 9;
                CombinationResult::Job(
                    Job::new_unsaved("Improve the net", vec![vec![
                        (Resource::Net, Amount::Spend(1)),
                        (Resource::NetUpgraded, Amount::Gain(1)),
                        (Resource::SpareParts, Amount::Spend(5)),
                    ]], vec![self.clone(), other.clone()], &mut id),
                    Some("Some clever engineering later and the net boasts struts to expand its opening during fishing".to_string()),
                )
            }
            both!(Net, Forge) |
            both!(NetUpgraded, Forge) => {
                let mut id = 10;
                CombinationResult::Job(
                    Job::new_unsaved("Burn the net", vec![
                        vec![
                            (Resource::NetUpgraded, Amount::Spend(1)),
                            (Resource::Forge, Amount::Catalyst(1)),
                        ],
                        vec![
                            (Resource::Net, Amount::Spend(1)),
                            (Resource::Forge, Amount::Catalyst(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("The net burns fruitlessly.".to_string()),
                )
            }
            both!(Claw, Scrap) |
            both!(ClawUpgraded, Scrap) => {
                let mut id = 11;
                CombinationResult::Job(
                    Job::new_unsaved("Destroy scrap", vec![
                        vec![
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Spend(1)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::Scrap, Amount::Spend(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("The scrap goes CRUUNCH under the might of the claw.".to_string()),
                )
            }
            both!(Claw, Fish) |
            both!(ClawUpgraded, Fish) => {
                let mut id = 12;
                CombinationResult::Job(
                    Job::new_unsaved("Destroy fish", vec![
                        vec![
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::Fish, Amount::Spend(10)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::Fish, Amount::Spend(10)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("If you line them up just right, the claw can ruin ten fish at a time".to_string()),
                )
            }
            both!(Claw, FoodRation) |
            both!(ClawUpgraded, FoodRation) => {
                let mut id = 13;
                CombinationResult::Job(
                    Job::new_unsaved("Destroy ration", vec![
                        vec![
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::FoodRation, Amount::Spend(1)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::FoodRation, Amount::Spend(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("The can goes crunch in the most satisfying way, but you can sense that Martha would be disappointed.".to_string()),
                )
            }
            both!(SpareParts, Martha) => {
                let mut id = 14;
                CombinationResult::Job(
                    Job::new_unsaved("Maintain Martha", vec![
                        vec![
                            (Resource::Martha, Amount::Catalyst(1)),
                            (Resource::SpareParts, Amount::Spend(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("Martha and you spend a wonderful evening over some well-deserved maintenance. She doesn't need it but you can tell she enjoys it.".to_string()),
                )
            }
            both!(Scrap, FoodRation) => {
                let mut id = 15;
                CombinationResult::Job(
                    Job::new_unsaved("Lose a food can among the scrap", vec![
                        vec![
                            (Resource::Scrap, Amount::Catalyst(1)),
                            (Resource::FoodRation, Amount::Spend(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("You but the can among the gray shapes, and when you turn back you've lost it.".to_string()),
                )
            }
            both!(SpareParts, Forge) => {
                let mut id = 16;
                CombinationResult::Job(
                    Job::new_unsaved("Resmelt spare parts", vec![
                        vec![
                            (Resource::SpareParts, Amount::Catalyst(3)),
                            (Resource::Forge, Amount::Catalyst(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("You find no ways to improve the forge. Depositing the spare parts into the forge causes them to melted and manufactured again, yielding some very similar parts.".to_string()),
                )
            }
            both!(Forge, Fish) => {
                let mut id = 17;
                CombinationResult::Job(
                    Job::new_unsaved("Vaporize fish", vec![
                        vec![
                            (Resource::Fish, Amount::Spend(1)),
                            (Resource::Forge, Amount::Catalyst(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("You chuck a fish from across the room into the forge, and you're rewarded with a strong burned fishy smell that hangs around for several days. Nice throw!".to_string()),
                )
            }
            both!(Forge, FoodRation) => {
                let mut id = 18;
                CombinationResult::Job(
                    Job::new_unsaved("Vaporize ration", vec![
                        vec![
                            (Resource::FoodRation, Amount::Spend(1)),
                            (Resource::Forge, Amount::Catalyst(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("It's not enough metal for the forge to do something useful with, so it simply burns.".to_string()),
                )
            }
            both!(Claw, Net) |
            both!(ClawUpgraded, Net) |
            both!(Claw, NetUpgraded) |
            both!(ClawUpgraded, NetUpgraded) => {
                let mut id = 19;
                CombinationResult::Job(
                    Job::new_unsaved("Destroy net", vec![
                        vec![
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::NetUpgraded, Amount::Spend(1)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::NetUpgraded, Amount::Spend(1)),
                        ],
                        vec![
                            (Resource::ClawUpgraded, Amount::Catalyst(1)),
                            (Resource::Net, Amount::Spend(1)),
                        ],
                        vec![
                            (Resource::Claw, Amount::Catalyst(1)),
                            (Resource::Net, Amount::Spend(1)),
                        ],
                    ], vec![self.clone(), other.clone()], &mut id),
                    Some("The claw closes, the net rips, and you've ruined your only way to get food.".to_string()),
                )
            }
            both!(Submarine, FoodRation) => {
                CombinationResult::Text("The submarine is not feeling especially hungry at the moment.".to_string())
            }
            both!(Submarine, Fish) => {
                CombinationResult::Text("Slapping a fish against the submarine does not do much, but it sure is enjoyable.".to_string())
            }
            both!(Net, Martha) |
            both!(Net, MarthaAtWork) |
            both!(NetUpgraded, Martha) |
            both!(NetUpgraded, MarthaAtWork) => {
                CombinationResult::Text("You proudly display the oversized net to your only companion. The giant machine does not react.".to_string())
            }
            both!(Scrap, Martha) => {
                CombinationResult::Text("You proudly display the chunk of metal to your only companion. Martha keeps resting without noticing.".to_string())
            }
            both!(Scrap, MarthaAtWork) => {
                CombinationResult::Text("You proudly display the chunk of metal to your only companion. Martha keeps working without noticing.".to_string())
            }
            both!(Fish, MarthaAtWork) => {
                CombinationResult::Text("Calm down, she's working on it!".to_string())
            }
            both!(Fish, Net) |
            both!(Fish, NetUpgraded) => {
                CombinationResult::Text("Yes, is indeed a net intended for fish, but this fish has already been caught. What are you doing?.".to_string())
            }
            both!(FoodRation, Net) |
            both!(FoodRation, NetUpgraded) => {
                CombinationResult::Text("The nets add close to no extra insulation to the food.".to_string())
            }
            both!(Scrap, Net) |
            both!(Scrap, NetUpgraded) => {
                CombinationResult::Text("Perhaps you could drag scrap around the facility like that, but it would not save time.".to_string())
            }
            both!(SpareParts, NetUpgraded) => {
                CombinationResult::Text("Beyond this point, further improvements rapidly approach diminishing returns.".to_string())
            }
            both!(SpareParts, ClawUpgraded) => {
                CombinationResult::Text("Improving the claw beyond this point would be hubris.".to_string())
            }
            both!(SpareParts, Scrap) => {
                CombinationResult::Text("You put a spare part on some scrap and fashion a mascot. It's nothing compared to Martha though.".to_string())
            }
            both!(Scrap, Fish) => {
                CombinationResult::Text("The scrap was pulled out of the ocean, and smelled like sea before, but it's worse now.".to_string())
            }
            both!(Scrap, Submarine) => {
                CombinationResult::Text("You put scrap around the submarine like a decoration, until you realize you won't get out of the dock with the scrap in the way.".to_string())
            }
            both!(SpareParts, Fish) => {
                CombinationResult::Text("You briefly consider creating a cybernetic fish before thinking better of it.".to_string())
            }
            both!(SpareParts, FoodRation) => {
                CombinationResult::Text("You briefly consider creating a cybernetic can bot but you have many inhibitions.".to_string())
            }
            both!(FoodRation, Fish) => {
                CombinationResult::Text("So you want more fish in your fish? Nah, ask Martha for help instead.".to_string())
            }
            both!(SpareParts, MarthaAtWork) => {
                CombinationResult::Text("You'd maintain Martha, but she might cut your hand off if you do it while she's working.".to_string())
            }
            both!(FoodRation, Martha) |
            both!(FoodRation, MarthaAtWork) => {
                CombinationResult::Text("You reckon this is like giving a present back to the person you received it from.".to_string())
            }
            both!(Submarine, Martha) |
            both!(Submarine, MarthaAtWork) => {
                CombinationResult::Text("Look, you'd love to show the submarine to Martha, but Martha is bolted to the ground and you have no way of getting the massive submarine out of the water and through the doors.".to_string())
            }
            both!(Submarine, Forge) => {
                CombinationResult::Text("Even if you'd like to melt the submarine, you can't get the huge submarine in there.".to_string())
            }
            both!(Martha, Forge) |
            both!(MarthaAtWork, Forge) => {
                CombinationResult::Text("Getting Martha to that malevolent entity would be difficult, but if you're absolutely certain you want to hurt her perhaps the claw would work.".to_string())
            }
            both!(ClawUpgraded, Claw) |
            both!(Martha, MarthaAtWork) |
            both!(Net, NetUpgraded) |
            (Resource::Scrap, Resource::Scrap) |
            (Resource::SpareParts, Resource::SpareParts) |
            (Resource::Fish, Resource::Fish) |
            (Resource::FoodRation, Resource::FoodRation) |
            (Resource::Submarine, Resource::Submarine) |
            (Resource::Net, Resource::Net) |
            (Resource::NetUpgraded, Resource::NetUpgraded) |
            (Resource::Claw, Resource::Claw) |
            (Resource::ClawUpgraded, Resource::ClawUpgraded) |
            (Resource::Martha, Resource::Martha) |
            (Resource::MarthaAtWork, Resource::MarthaAtWork) |
            (Resource::Forge, Resource::Forge) |
            (Resource::Player, _) | (_, Resource::Player) => {
                CombinationResult::Text("Congrats, you broke the game. Let me know how!".to_string())
            }
            // _ => {
            //     CombinationResult::Nothing
            // }
        }
    }
}

pub fn attributes() -> AttributeMappings {
    BTreeMap::from([
        (Resource::Player, ResourceAttributes {
            upkeep: vec![Vec::from([(Resource::FoodRation, Amount::Spend(1))])],
            visible: false,
            display_as_name: true,
            row: 0,
        }),
        (Resource::Submarine, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Net, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::NetUpgraded, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Claw, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::ClawUpgraded, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Martha, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::MarthaAtWork, ResourceAttributes {
            upkeep: vec![
                vec![
                    (Resource::Fish, Amount::Spend(15)),
                    (Resource::FoodRation, Amount::Gain(5)),
                ],
                vec![
                    (Resource::Martha, Amount::Gain(1)),
                    (Resource::MarthaAtWork, Amount::Spend(1)),
                ],
            ],
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        (Resource::Forge, ResourceAttributes {
            upkeep: Vec::from([]),
            visible: true,
            display_as_name: true,
            row: 1,
        }),
        // (Resource::ForgeGlowing, ResourceAttributes {
        //     upkeep: Vec::from([]),
        //     visible: true,
        //     row: 1,
        // }),
    ])
}
