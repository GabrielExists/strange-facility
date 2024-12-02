#![allow(dead_code)]

use std::collections::BTreeMap;


#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Amount {
    Gain(i64),
    Spend(i64),
    Catalyst(i64),
    GainX(i64),
    SpendX(i64),
    CatalystX(i64),
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
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Resource {
    Player,
    Scrap,
    SpareParts,
    Fish,
    FoodRation,
    AirTank,
    AirTankPressurizer,
}

pub struct ResourceAttributes {
    pub upkeep: Vec<(Resource, Amount)>,
    pub visible: bool,
}

pub type ResourceSet = BTreeMap<Resource, i64>;
pub type ResourceList = Vec<(Resource, i64)>;
pub type AttributeMappings = BTreeMap<Resource, ResourceAttributes>;

#[derive(Clone, Debug)]
pub struct Job {
    pub button_text: &'static str,
    pub delta: Vec<(Resource, Amount)>,
    pub button: bool,
}

pub fn attributes() -> AttributeMappings {
    BTreeMap::from([
        (Resource::Player, ResourceAttributes {
            upkeep: Vec::from([(Resource::FoodRation, Amount::Spend(1))]),
            visible: false,
        }),
        (Resource::AirTankPressurizer, ResourceAttributes {
            upkeep: Vec::from([(Resource::AirTank, Amount::Gain(1))]),
            visible: true,
        }),
    ])
}

impl Job {
    pub fn starting_resources() -> Job {
        Job {
            button_text: "Starting resources",
            delta: vec![
                (Resource::Player, Amount::Gain(1)),
                (Resource::FoodRation, Amount::Gain(5)),
            ],
            button: false,
        }
    }

    pub fn list_jobs() -> Vec<Job> {
        vec![
            Job {
                button_text: "Deconstruct derelict machines",
                delta: vec![
                    (Resource::Scrap, Amount::Gain(3)),
                ],
                button: true,
            },
            Job {
                button_text: "Toil in the forge",
                delta: vec![
                    (Resource::Scrap, Amount::Spend(5)),
                    (Resource::SpareParts, Amount::Gain(1)),
                ],
                button: true,
            },
            Job {
                button_text: "Collect the ocean's bounty, by submarine and net",
                delta: vec![
                    (Resource::Fish, Amount::Gain(10)),
                ],
                button: true,
            },
            Job {
                button_text: "Operate Martha, the canning machine",
                delta: vec![
                    (Resource::Fish, Amount::SpendX(10)),
                    (Resource::FoodRation, Amount::GainX(3)),
                ],
                button: true,
            },
            Job {
                button_text: "Jury rig an air pressurizer",
                delta: vec![
                    (Resource::SpareParts, Amount::Spend(2)),
                    (Resource::AirTankPressurizer, Amount::Gain(1)),
                ],
                button: true,
            },
            Job {
                button_text: "Make submarine worthy of the deep seas and depart (winning the game)",
                delta: vec![
                    (Resource::SpareParts, Amount::Spend(10)),
                    (Resource::AirTank, Amount::Spend(60)),
                ],
                button: true,
            },
        ]
    }
// pub fn list_visible_jobs() -> Vec<Job> {
//     let attributes = attributes()
//     Self::list_jobs().into_iter().map(|item| {
//         attributes.get()
//     })
// }
}

pub fn apply_job(orig_resources: ResourceSet, job: &Job) -> (ResourceSet, Result<(), String>) {
    let mut resources = orig_resources.clone();
    for (current_resource, attribute) in attributes() {
        if let Some(num) = resources.get(&current_resource) {
            let delta = attribute.upkeep
                .into_iter()
                .map(|(delta_resource, delta_amount)| {
                    (delta_resource, delta_amount.multiply((*num).clone()))
                })
                .collect();

            let (new_resources, result) = apply_delta(resources, &delta);
            if let Err(error) = result {
                return (new_resources, Err(error));
            }
            resources = new_resources;
        }
    }
    apply_delta(resources, &job.delta)
}

pub fn apply_delta(mut resources: ResourceSet, delta: &Vec<(Resource, Amount)>) -> (ResourceSet, Result<(), String>) {
    let mut x = None;
    let mut error = None;
    for (resource, amount) in delta.iter() {
        let current_resource = resources.entry(resource.clone()).or_insert(0);
        match amount {
            Amount::Gain(_) => {}
            Amount::GainX(_) => {}
            Amount::Spend(delta) |
            Amount::Catalyst(delta) => {
                if delta > current_resource {
                    error = Some(format!("Not enough {}", resource));
                }
            }
            Amount::SpendX(delta) |
            Amount::CatalystX(delta) => {
                if let Some(new_x) = current_resource.checked_div(delta.clone()) {
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
                    if delta < current_resource {
                        error = Some(format!("This button is configured incorrectly, with a zero quantity for resource {:?}.", current_resource));
                    }
                }
            }
        }
    }
    for (resource, amount) in delta.iter() {
        let current_resource = resources.entry(resource.clone()).or_insert(0);
        match amount {
            Amount::Gain(delta) => {
                *current_resource += delta;
            }
            Amount::Spend(delta) => {
                *current_resource -= delta;
            }
            Amount::Catalyst(_) => {}
            Amount::GainX(delta_per) => {
                *current_resource += delta_per * x.unwrap_or(0);
            }
            Amount::SpendX(delta_per) => {
                *current_resource -= delta_per * x.unwrap_or(0);
            }
            Amount::CatalystX(_delta_per) => {}
        }
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

    if let Some(error) = error {
        (resources, Err(error))
    } else {
        (resources, Ok(()))
    }
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