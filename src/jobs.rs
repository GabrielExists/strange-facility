#![allow(dead_code)]
use std::collections::BTreeMap;

pub type Resources = BTreeMap<Resource, u64>;
#[derive(Clone, Debug)]
pub struct Job {
    pub button_text: &'static str,
    pub delta: Vec<(Resource, Amount)>,
}

impl Job {
    pub fn list_jobs() -> Vec<Job> {
        vec![
            Job {
                button_text: "Deconstruct derelict machines",
                delta: vec![
                    (Resource::Scrap, Amount::Gain(3)),
                    (Resource::FoodRation, Amount::Spend(1)),
                ],
            },
            Job {
                button_text: "Toil in the forge",
                delta: vec![
                    (Resource::Scrap, Amount::Spend(5)),
                    (Resource::SpareParts, Amount::Gain(1)),
                    (Resource::FoodRation, Amount::Spend(1)),
                ],
            },
            Job {
                button_text: "Collect the ocean's bounty, by submarine and net",
                delta: vec![
                    (Resource::Fish, Amount::Gain(10)),
                    (Resource::FoodRation, Amount::Spend(1)),
                ],
            },
            Job {
                button_text: "Operate Martha, the canning machine",
                delta: vec![
                    (Resource::FoodRation, Amount::Spend(1)),
                    (Resource::Fish, Amount::SpendX(10)),
                    (Resource::FoodRation, Amount::GainX(3)),
                ],
            },
            Job {
                button_text: "Make submarine worthy of the deep seas and depart (winning the game)",
                delta: vec![
                    (Resource::SpareParts, Amount::Spend(10)),
                    (Resource::FoodRation, Amount::Spend(1)),
                ],
            },
        ]
    }
}
pub fn starting_resources() -> Resources {
    BTreeMap::from([
        (Resource::FoodRation, 5)
    ])
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Resource {
    Scrap,
    SpareParts,
    Fish,
    FoodRation,
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Amount {
    Gain(u64),
    Spend(u64),
    Catalyst(u64),
    GainX(u64),
    SpendX(u64),
    CatalystX(u64),
}

pub fn apply_job(mut resources: Resources, job: Job) -> Result<Resources, String> {
    let mut x = None;
    for (resource, amount) in job.delta.iter() {
        let current_resource = resources.entry(resource.clone()).or_insert(0);
        match amount {
            Amount::Gain(_) => {}
            Amount::GainX(_) => {}
            Amount::Spend(delta) |
            Amount::Catalyst(delta) => {
                if delta > current_resource {
                    return Err(format!(""));
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
                        return Err(format!("This button is configured incorrectly, with a zero quantity for resource {:?}.", current_resource))
                    }
                }
            }
        }
    }
    for (resource, amount) in job.delta.into_iter() {
        let current_resource = resources.entry(resource).or_insert(0);
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
            Amount::CatalystX(_delta_per) => {

            }
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
    Ok(resources)
}

pub fn apply_jobs(mut resources: Resources, jobs: Vec<Job>) -> Result<Resources, String> {
    for job in jobs {
        resources = apply_job(resources, job)?;
    }
    Ok(resources)
}


pub fn apply_jobs_partial(mut resources: Resources, jobs: Vec<Job>) -> Resources {
    for job in jobs {
        match apply_job(resources.clone(), job) {
            Ok(new_resources) => {
                resources = new_resources;
            }
            Err(_) => {}
        }
    }
    resources
}

pub fn get_job_success_zip(resources: Resources, jobs: Vec<Job>) -> Vec<(Job, bool)> {
    let mut results = Vec::new();
    for index in 0..jobs.len() {
        let previous_resources = apply_jobs_partial(resources.clone(), jobs[0..index].to_vec());
        let job = jobs.get(index).expect("This index should be valid.").clone();
        let success = apply_job(previous_resources, job.clone()).is_ok();
        results.push((job, success));
    }
    results
}