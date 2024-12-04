use std::collections::BTreeMap;
use crate::app::*;
use crate::game::*;
use crate::jobs::*;

pub struct ViewCache {
    pub job_rows: Vec<JobRow>,
    pub seen_resources: Vec<Resource>,
    pub current_resources: Vec<CurrentResource>,
    pub total_days: usize,
    pub user_error: Option<String>,
    pub max_row: usize,
    pub game_state: GameState,
}

pub struct JobRow {
    pub job: Job,
    pub output: JobOutput,
    pub resource_list: Vec<(Resource, i64)>,
    pub resource_tool_list: Vec<ResourceTool>,
    pub index: usize,
}

pub struct ResourceTool {
    pub resource_pair: (Resource, i64),
    pub status: ResourceToolStatus,
}

pub enum ResourceToolStatus {
    Standard,
    Changed,
    Removed,
}

pub struct CurrentResource {
    pub resource: Resource,
    pub amount: i64,
    pub row: usize,
}

pub enum HistoryStep {
    Job(Job),
    AddOne(usize),
    RemoveOne(usize),
    RemoveCluster(usize),
}

pub enum GameState {
    Playing,
    Won {
        spent_days: usize,
    },
}

pub fn history_from_combinations(combinations: Vec<(Resource, Resource)>) -> Vec<HistoryStep> {
    let mut history = Vec::new();
    for (first, second) in combinations {
        let result = first.combine(&second);
        match result {
            CombinationResult::Job(job, _) => {
                history.push(HistoryStep::Job(job));
            }
            CombinationResult::Text(_) => {}
            CombinationResult::Nothing => {}
        }
    }
    history
}

impl App {

    // fn norm(resource_sets: &mut Vec<BTreeMap<Resource, u64>>, resources: &mut Resources) {
    //     let attributes = attributes();
    //     for resource in resources.iter() {
    //         let visible = attributes.get(resource.0).map(|atts| atts.visible).unwrap_or(true);
    //         for set in resource_sets.iter_mut() {
    //             if !visible {
    //                 set.remove(resource.0);
    //             } else {
    //                 set.entry(resource.0.clone()).or_insert(0);
    //             }
    //         }
    //     }
    //     for (resource, attribute) in attributes.iter() {
    //         if !attribute.visible {
    //             resources.remove(resource);
    //         }
    //     }
    // }
    //

    // fn normalize_list(resource_sets: &Vec<ResourceSet>, seen_resources: &Vec<Resource>) -> Vec<Vec<(Resource, i64)>> {
    //     resource_sets.iter().map(|set| {
    //         Self::normalize(set, seen_resources)
    //     }).collect()
    // }
    pub fn add_job(&mut self, job: Job) {
        self.history.push(HistoryStep::Job(job));
        self.redo_queue.clear();
        self.refresh_view_cache();
    }

    fn normalize(resource_set: &ResourceSet, seen_resources: &Vec<Resource>) -> Vec<(Resource, i64)> {
        let mut result = Vec::new();
        for resource in seen_resources.iter() {
            result.push((resource.clone(), resource_set.get(resource).map(|number| number.clone()).unwrap_or(0)));
        }
        result
    }

    fn remove_invisible(resources: &mut Vec<Resource>) {
        let attributes = attributes();
        resources.retain(|item| {
            attributes.get(item).map(|attribute| attribute.visible && !attribute.display_as_name).unwrap_or(true)
        });
    }
    pub fn apply_combination(&mut self, first_resource: &Resource, second_resource: &Resource) {
        let combination_result = first_resource.combine(second_resource);
        match &combination_result {
            CombinationResult::Job(new_job, _) => {
                if new_job.saved {
                    if self.discovered_jobs.iter().find(|job| job.id == new_job.id).is_none() {
                        self.discovered_jobs.push(new_job.clone());
                    }
                }
                self.add_job(new_job.clone())
            }
            CombinationResult::Text(_text) => {}
            CombinationResult::Nothing => {}
        }
        self.last_combination = combination_result;
    }

    fn create_resource_tool_list(resources: &ResourceSet, changed: Option<&Vec<Resource>>) -> Vec<ResourceTool> {
        let attributes = attributes();
        let mut resource_tool_list = Vec::new();
        for (resource, amount) in resources.iter() {
            let (display_as_name, visible) = attributes.get(resource)
                .map(|attribute| (attribute.display_as_name, attribute.visible))
                .unwrap_or((false, false));
            let changed = changed.map(|changed| changed.contains(resource)).unwrap_or(false);
            if display_as_name && visible {
                if changed {
                    if *amount > 0 {
                        resource_tool_list.push(ResourceTool {
                            resource_pair: (resource.clone(), *amount),
                            status: ResourceToolStatus::Changed,
                        });
                    } else {
                        resource_tool_list.push(ResourceTool {
                            resource_pair: (resource.clone(), *amount),
                            status: ResourceToolStatus::Removed,
                        });
                    }
                } else {
                    if *amount > 0 {
                        resource_tool_list.push(ResourceTool {
                            resource_pair: (resource.clone(), *amount),
                            status: ResourceToolStatus::Standard,
                        });
                    }
                }
            }
        }
        resource_tool_list
    }

    pub fn refresh_view_cache(&mut self) {
        let result = Self::create_view_cache(&self.history);
        match result {
            Ok(view_cache) => {
                self.view_cache = view_cache;
            }
            Err(programmer_error) => {
                self.programmer_error = Some(programmer_error);
            }
        }
    }


    pub fn create_view_cache(steps: &Vec<HistoryStep>) -> Result<ViewCache, String> {
        let mut user_error = None;
        let mut seen_resources = Vec::new();
        let mut jobs_to_execute = vec![Job::starting_resources()];
        // Apply history to create job application order
        for step in steps {
            match step {
                HistoryStep::Job(job) => {
                    jobs_to_execute.push(job.clone());
                }
                HistoryStep::AddOne(index) => {
                    let job = jobs_to_execute.get(*index);
                    if let Some(job) = job {
                        jobs_to_execute.insert(*index, job.clone())
                    }
                }
                HistoryStep::RemoveOne(index) => {
                    if *index < jobs_to_execute.len() {
                        jobs_to_execute.remove(*index);
                    }
                }
                HistoryStep::RemoveCluster(index) => {
                    if *index < jobs_to_execute.len() {
                        let first_job = jobs_to_execute.remove(*index);
                        // The latter elements have moved over now, so we can keep checking this
                        // slot to find all consecutive similar jobs
                        loop {
                            match jobs_to_execute.get(*index) {
                                Some(job) => {
                                    if first_job.id == job.id {
                                        jobs_to_execute.remove(*index);
                                    } else {
                                        break;
                                    }
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        // Apply jobs
        let mut resources = BTreeMap::new();
        let mut job_and_output = Vec::new();
        for job in jobs_to_execute.into_iter() {
            let job_output = apply_job(resources.clone(), &job)?;
            for resource in job_output.resources_after.keys() {
                let amount = job_output.resources_after.get(resource);
                if let Some(amount) = amount {
                    if !seen_resources.contains(resource) && *amount != 0 {
                        seen_resources.push(resource.clone());
                    }
                }
            }

            if job_output.is_ok() {
                resources = job_output.resources_after.clone();
            }
            match job_output.user_message() {
                None => {
                    user_error = None;
                }
                Some(error_message) => {
                    user_error = Some(error_message.to_string());
                }
            }
            job_and_output.push((job, job_output));
        };

        let mut total_days = 0;
        let mut game_state = GameState::Playing;
        for (job, output) in job_and_output.iter() {
            if output.is_ok() {
                total_days += job.instances;
                if job.id == WIN_JOB_ID {
                    game_state = GameState::Won {
                        spent_days: total_days,
                    }
                }
            }
        }

        // Prepare the complete list of resources that should be represented on each row of the table
        Self::remove_invisible(&mut seen_resources);
        seen_resources.sort();

        // Merge jobs
        let mut job_rows = Vec::new();
        for (index, (this_job, this_output)) in job_and_output.into_iter().enumerate() {
            let resource_list = Self::normalize(&this_output.resources_after, &seen_resources);
            match job_rows.last_mut() {
                None => {
                    let resource_by_name_list = Self::create_resource_tool_list(&this_output.resources_after, Some(&this_output.get_changed_resources()));
                    job_rows.push(JobRow {
                        job: this_job,
                        output: this_output,
                        resource_list,
                        resource_tool_list: resource_by_name_list,
                        index,
                    });
                }
                Some(last_row) => {
                    if this_job.id == last_row.job.id &&
                        this_output.is_mergeable(&last_row.output)
                    {
                        let changed = &this_output.get_changed_resources().into_iter().chain(last_row.output.get_changed_resources()).collect::<Vec<_>>();
                        let resource_tool_list = Self::create_resource_tool_list(&this_output.resources_after, Some(changed));
                        last_row.job.instances += this_job.instances;
                        last_row.resource_list = resource_list;
                        last_row.resource_tool_list = resource_tool_list;
                        last_row.output.main_output.changed_resources.extend(this_output.get_changed_resources().into_iter());
                    } else {
                        let resource_by_name_list = Self::create_resource_tool_list(&this_output.resources_after, Some(&this_output.get_changed_resources()));
                        job_rows.push(JobRow {
                            job: this_job,
                            output: this_output,
                            resource_list,
                            resource_tool_list: resource_by_name_list,
                            index,
                        });
                    }
                }
            }
        }

        // Process selectable resources for display
        let attributes = attributes();
        let mut max_row = 0;
        let resource_by_name_list = Self::create_resource_tool_list(&resources, None);
        let current_resources = Self::normalize(&resources, &seen_resources)
            .into_iter()
            .chain(resource_by_name_list.into_iter().map(|tool| tool.resource_pair))
            .filter_map(|(resource, amount)| {
                if amount == 0 {
                    return None;
                }
                let row = attributes.get(&resource).map(|attributes| {
                    attributes.row.clone()
                }).unwrap_or(0);
                max_row = std::cmp::max(max_row, row);
                Some(CurrentResource {
                    resource,
                    amount,
                    row,
                })
            }).collect::<Vec<_>>();

        Ok(ViewCache {
            job_rows,
            seen_resources,
            current_resources,
            total_days,
            user_error,
            max_row,
            game_state,
        })
    }
}
