// #![cfg(target_arch = "wasm32")]

use std::collections::BTreeMap;
use yew::prelude::*;
use std::fmt::{Display, Formatter};
use std::iter::{zip, Zip};
use std::slice::Iter;
use crate::jobs::*;

pub struct App {
    pub user_error: Option<String>,
    pub applied_jobs: Vec<Job>,
    pub view_cache: ViewCache,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    AddJob(Job),
    RemoveHistory(usize),
}

pub struct ViewCache {
    pub job_and_ok: Vec<(Job, bool)>,
    pub resource_lists: Vec<Vec<(Resource, i64)>>,
    pub seen_resources: Vec<Resource>,
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let applied_jobs = vec![Job::starting_resources()];
        let (view_cache, user_error) = App::create_view_cache(&applied_jobs);
        Self {
            user_error,
            applied_jobs,
            view_cache,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.user_error = None;
        match msg {
            AppMessage::AddJob(job) => {
                self.applied_jobs.push(job);
                self.refresh_view_cache();
                true
            }
            AppMessage::RemoveHistory(index) => {
                if index < self.applied_jobs.len() {
                    self.applied_jobs.remove(index);
                }
                self.refresh_view_cache();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <div class="flex flex-row">
            <div class="h-screen border border-slate-800 bg-blue-100 flex-col">
                <div class="p-4 te">
                    {"This game is very WIP, but there's some stuff to play around with. Actions marked in red failed due to lack of resources. Try to successfully depart in the fewest days possible."}
                </div>
                // Current error
                { if self.user_error.is_some() {
                    html!{ <div class="flex flex-row gap-y-2 p-2 border border-red-600"> {&self.user_error} </div> }
                } else {
                    html!{ <div></div> }
                }
                }
                // List available jobs
                <div class="flex flex-row gap-y-2">
                    { for Job::list_jobs().into_iter().map(|job| {
                        let callback_job = job.clone();
                        html! {
                            <button class="border border-slate-900 background-slate-100 p-2" onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::AddJob(callback_job.clone()))}>
                                {job.button_text}
                            </button>
                        }
                    })}
                    // Total jobs
                    <div class="border border-slate-900 background-slate-100 p-2 ml-4">
                        {format!("Total days spent: {}", self.applied_jobs.len())}
                    </div>
                </div>

                // History of jobs
            // <div class="flex flex-col gap-y-2">
            // { for self.view_cache.job_and_ok.iter().enumerate().map(|(index, (job, is_ok))| {
            //     html! {
            //         <button class={
            //             "border border-slate-900 background-slate-100 p-2 flex flex-row items-center justify-center"
            //         } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::RemoveHistory(index))}>
            //             <div class={if *is_ok {"bg-red-600 w-1 h-6 p-1 mr-2 collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
            //             {job.button_text}
            //         </button>
            //     }
            // })}
                // </div>
                                // "border border-slate-900 background-slate-100 p-2"
                // Table of resource steps and jobs
                <table class="table-auto">
                    <thead>
                        <tr>
                            <td class="border border-slate-900 background-slate-100 p-2">
                                {"Task for today"}
                            </td>
                    { for self.view_cache.seen_resources.iter().map(|resource| {
                        html! {
                            <td class="border border-slate-900 background-slate-100 p-2">
                                {resource.to_string()}
                            </td>
                        }
                    })}
                        </tr>
                    </thead>
                    <tbody>
                    { for self.get_table_zip().enumerate().rev().map(|(index, ((job, is_ok), resources))| {
                        html! {
                        <tr>
                            <td class="border border-slate-900 background-slate-100">
                            { if job.button {
                                html! {
                                <button class={
                                    "border border-slate-900 background-slate-100 p-2 flex flex-row items-center justify-center"
                                } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::RemoveHistory(index))}>
                                    <div class={if *is_ok {"bg-red-600 w-1 h-6 p-1 mr-2 collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
                                    {job.button_text}
                                </button>
                                }
                            } else {
                                html! {
                                <div class={ "background-slate-100 p-2 flex flex-row items-center justify-center" }>
                                    <div class={if *is_ok {"bg-red-600 w-1 h-6 p-1 mr-2 collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
                                    {job.button_text}
                                </div>
                                }
                            } }
                            </td>
                            { for resources.iter().map(|(_resource, amount)| {
                                html! {
                            <td class="border border-slate-900 background-slate-100 p-2">
                                    <div class="flex">
                                <div class={if *amount >= 0 {"collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
                                {amount.to_string()}
                                    </div>
                            </td>
                                }
                            })}
                        </tr>
                        }
                    })}
                    </tbody>
                </table>
            </div>
            // <div class="w-1/5 h-screen border border-slate-800 bg-blue-300">
            // </div>
        </div>
        }
    }
}

impl App {
    pub fn refresh_view_cache(&mut self) {
        let (view_cache, user_error) = Self::create_view_cache(&self.applied_jobs);
        self.user_error = user_error;
        self.view_cache = view_cache;
    }

    pub fn create_view_cache(applied_jobs: &Vec<Job>) -> (ViewCache, Option<String>) {
        let mut job_and_ok = Vec::new();
        let mut resource_sets = Vec::new();
        let mut user_error = None;
        let mut seen_resources = Vec::new();
        let mut resources = BTreeMap::new();
        for job in applied_jobs.iter() {
            let (new_resources, result) = apply_job(resources.clone(), &job);
            for resource in new_resources.keys() {
                if !seen_resources.contains(resource) {
                    seen_resources.push(resource.clone());
                }
            }
            match result {
                Ok(()) => {
                    job_and_ok.push((job.clone(), true));
                    resource_sets.push(new_resources.clone());
                    resources = new_resources;
                    user_error = None;
                }
                Err(error_message) => {
                    job_and_ok.push((job.clone(), false));
                    resource_sets.push(new_resources.clone());
                    user_error = Some(error_message);
                }
            }
        }
        // seen_resources.append(&mut resources);
//     for index in 0..jobs.len() {
//         let previous_resources = apply_jobs_partial(resources.clone(), jobs[0..index].to_vec());
//         let job = jobs.get(index).expect("This index should be valid.").clone();
//         let success = apply_job(previous_resources, job.clone()).is_ok();
//         results.push((job, success));
//     }

        Self::prune(&mut seen_resources);
        let resource_lists = Self::normalize_list(&resource_sets, &seen_resources);

        let view_cache = ViewCache {
            job_and_ok,
            resource_lists,
            seen_resources,
        };
        (view_cache, user_error)
    }

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

    fn normalize_list(resource_sets: &Vec<ResourceSet>, seen_resources: &Vec<Resource>) -> Vec<Vec<(Resource, i64)>>{
        resource_sets.iter().map(|set|{
            Self::normalize(set, seen_resources)
        }).collect()
    }
    fn normalize(resource_set: &ResourceSet, seen_resources: &Vec<Resource>) -> Vec<(Resource, i64)> {
        let mut result = Vec::new();
        for resource in seen_resources.iter() {
            result.push((resource.clone(), resource_set.get(resource).map(|number| number.clone()).unwrap_or(0)));
        }
        result
    }

    fn prune(resources: &mut Vec<Resource>) {
        let attributes = attributes();
        resources.retain(|item|{
            attributes.get(item).map(|attribute| attribute.visible).unwrap_or(true)
        });
    }

    fn get_table_zip(&self) -> Zip<Iter<(Job, bool)>, Iter<ResourceList>> {
        zip(self.view_cache.job_and_ok.iter(), self.view_cache.resource_lists.iter())
    }
    // fn get_starting_resources(&self) -> Resources {
    //     let mut resources = starting_resources();
    //     Self::prune_and_normalize(&mut resources, &self.view_cache.resources);
    //     resources
    // }
    // fn get_visible_resources(&self) -> Resources {
    //     let attributes = attributes();
    //     apply_jobs_partial(starting_resources(), self.applied_jobs.clone())
    //         .into_iter()
    //         .filter(|(resource, _num)| {
    //             attributes.get(resource).map(|att| att.visible).unwrap_or(true)
    //         })
    //         .collect()
    // }
    //
    // fn get_visible_resources_first_n(&self, n: usize) -> Resources {
    //     let attributes = attributes();
    //     apply_jobs_partial(starting_resources(), self.applied_jobs.clone())
    //         .into_iter()
    //         .take(n)
    //         .filter(|(resource, _num)| {
    //             attributes.get(resource).map(|att| att.visible).unwrap_or(true)
    //         })
    //         .collect()
    // }
}

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
            Resource::AirTank => {
                f.write_str("Air tank")
            }
            Resource::AirTankPressurizer => {
                f.write_str("Air tank pressurizer")
            }
        }
    }
}
