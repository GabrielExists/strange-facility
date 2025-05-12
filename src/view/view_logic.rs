use std::collections::BTreeMap;
use yew::Classes;
use crate::app::*;
use crate::core::job::{Job, JobOutput, starting_resources};
use crate::core::resource::{attributes, Resource, ResourceAttributes, ResourceSet};
use crate::game::*;
use crate::jobs::*;
use crate::view::app::{App, State};
use crate::view::class_string;
use crate::view::view::class_string;

pub struct ViewCache {
    pub current_resources: Vec<Vec<CurrentResource>>,
    pub job_rows: Vec<JobRow>,
    pub resource_headings: Vec<Resource>,
    pub total_days: usize,
    pub user_error: Option<String>,
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
    #[allow(dead_code)]
    pub row: usize,
    pub classes: Classes,
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


impl App {

    pub fn add_job(&mut self, job: Job) {
        self.state.displayed_job = Some(job.clone());
        self.state.history.push(HistoryStep::Job(job));
        self.state.redo_queue.clear();
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

    pub fn create_resource_view(state: &State, newest_row_of_resources: ResourceSet) -> Vec<Vec<CurrentResource>> {
        let attributes = attributes();
        let mut max_row = 0;
        let current_resources = newest_row_of_resources.iter()
            .filter_map(|(resource, amount)| {
                if *amount > 0 {
                    if let Some(att) = attributes.get(resource) {
                        if att.row > max_row {
                            max_row = att.row;
                        }
                        Some((resource, *amount, Some(att)))
                    } else {
                        Some((resource, *amount, None))
                    }
                } else {
                    None
                }
            }).collect::<Vec<(&Resource, i64, Option<&ResourceAttributes>)>>();

        let mut current_resource_rows = Vec::new();
        for row_number in (0..=max_row).rev() {
            let row = current_resources.iter()
                .filter(|(_current_resource, _amount, att)| {
                    let (current_row, visible) = if let Some(att) = att {
                        (att.row, att.visible)
                    } else {
                        (0, true)
                    };
                    current_row == row_number && visible
                })
                .map(|(current_resource, amount, att)| {
                    let resource = (*current_resource).clone();
                    // let selected = state.selected_resource.map(|selected| selected == resource).unwrap_or(false);
                    let selected = false;
                    let show_blue_background =  selected;
                    let show_blue_border = false;
                    let row = if let Some(att) = att {
                        att.row
                    } else {
                        0
                    };
                    let class = if show_blue_border && show_blue_background {
                        class_string("bg-blue-500 text-slate-100 border-blue-500")
                    } else if show_blue_background {
                        class_string("bg-blue-500 text-slate-100 border-slate-900")
                    } else if show_blue_border {
                        class_string("border-blue-500 active:bg-blue-500 active:text-slate-100")
                    } else {
                        class_string("border-slate-900 active:bg-blue-500 active:text-slate-100")
                    };
                    CurrentResource {
                        resource: resource.clone(),
                        amount: *amount,
                        row,
                        classes: class,
                    }
                })
                .collect::<Vec<_>>();
            current_resource_rows.push(row);
        }
        current_resource_rows
    }

    pub fn refresh_view_cache(&mut self) {
        let result = Self::create_view_cache(&self.state);
        match result {
            Ok(view_cache) => {
                self.view_cache = view_cache;
            }
            Err(programmer_error) => {
                self.programmer_error = Some(programmer_error);
            }
        }
    }


    pub fn create_view_cache(state: &State) -> Result<ViewCache, String> {
        let mut user_error = None;
        let mut seen_resources = Vec::new();
        let mut jobs_to_execute = vec![starting_resources()];
        // Apply history to create job application order
        for step in state.history.iter() {
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
        // // Apply jobs
        // let mut resources = BTreeMap::new();
        // let mut job_and_output = Vec::new();
        // for job in jobs_to_execute.into_iter() {
        //     let job_output = apply_job(resources.clone(), &job)?;
        //     for resource in job_output.resources_after.keys() {
        //         let amount = job_output.resources_after.get(resource);
        //         if let Some(amount) = amount {
        //             if !seen_resources.contains(resource) && *amount != 0 {
        //                 seen_resources.push(resource.clone());
        //             }
        //         }
        //     }
        //
        //     if job_output.is_ok() {
        //         resources = job_output.resources_after.clone();
        //     }
        //     match job_output.user_message() {
        //         None => {
        //             user_error = None;
        //         }
        //         Some(error_message) => {
        //             user_error = Some(error_message.to_string());
        //         }
        //     }
        //     job_and_output.push((job, job_output));
        // };
        //
        // let mut total_days = 0;
        // let mut game_state = GameState::Playing;
        // for (job, output) in job_and_output.iter() {
        //     if output.is_ok() {
        //         // total_days += job.instances;
        //         if job.id == WIN_JOB_ID {
        //             game_state = GameState::Won {
        //                 spent_days: total_days,
        //             }
        //         }
        //     }
        // }
        //
        // // Prepare the complete list of resources that should be represented on each row of the table
        // Self::remove_invisible(&mut seen_resources);
        // seen_resources.sort();
        //
        // // Merge jobs
        // let mut job_rows = Vec::new();
        // for (index, (this_job, this_output)) in job_and_output.into_iter().enumerate() {
        //     let resource_list = Self::normalize(&this_output.resources_after, &seen_resources);
        //     match job_rows.last_mut() {
        //         None => {
        //             let resource_by_name_list = Self::create_resource_tool_list(&this_output.resources_after, Some(&this_output.get_changed_resources()));
        //             job_rows.push(JobRow {
        //                 job: this_job,
        //                 output: this_output,
        //                 resource_list,
        //                 resource_tool_list: resource_by_name_list,
        //                 index,
        //             });
        //         }
        //         Some(last_row) => {
        //             if this_job.id == last_row.job.id &&
        //                 this_output.is_mergeable(&last_row.output)
        //             {
        //                 let changed = &this_output.get_changed_resources().into_iter().chain(last_row.output.get_changed_resources()).collect::<Vec<_>>();
        //                 let resource_tool_list = Self::create_resource_tool_list(&this_output.resources_after, Some(changed));
        //                 last_row.job.instances += this_job.instances;
        //                 last_row.resource_list = resource_list;
        //                 last_row.resource_tool_list = resource_tool_list;
        //                 last_row.output.main_output.changed_resources.extend(this_output.get_changed_resources().into_iter());
        //             } else {
        //                 let resource_by_name_list = Self::create_resource_tool_list(&this_output.resources_after, Some(&this_output.get_changed_resources()));
        //                 job_rows.push(JobRow {
        //                     job: this_job,
        //                     output: this_output,
        //                     resource_list,
        //                     resource_tool_list: resource_by_name_list,
        //                     index,
        //                 });
        //             }
        //         }
        //     }
        // }

        // Process selectable resources for display
        // let _attributes = attributes();
        // let current_resources = App::create_resource_view(state, resources);

        Ok(ViewCache {
            current_resources: Vec::new(),
            job_rows: Vec::new(),
            resource_headings: seen_resources,
            total_days: 0,
            user_error,
            game_state: GameState::Playing,
        })
    }
}
