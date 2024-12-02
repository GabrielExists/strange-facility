// #![cfg(target_arch = "wasm32")]

use yew::prelude::*;
use std::fmt::{Display, Formatter};
use crate::jobs::*;
// use serde::{Deserialize, Serialize};
// use serde_wasm_bindgen::to_value;
// use wasm_bindgen::prelude::*;
// use wasm_bindgen_futures::spawn_local;

pub struct App {
    user_error: Option<String>,
    applied_buttons: Vec<Job>,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    AddJob (Job),
    RemoveHistory(usize),
}


impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            user_error: None,
            applied_buttons: Vec::new()
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.user_error = None;
        match msg {
            AppMessage::AddJob(job) => {
                self.applied_buttons.push(job);
                // let new_resources = apply_job(self.resources.clone(), job);
                // match new_resources {
                //     Ok(new_resources) => {
                //        self.resources = new_resources;
                //     }
                //     Err(error_message) => {
                //         self.user_error = Some(error_message);
                //     }
                // }
                true
            }
            AppMessage::RemoveHistory(index) => {
                if index < self.applied_buttons.len() {
                    self.applied_buttons.remove(index);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <div class="flex flex-row">
            <div class="w-4/5 h-screen border border-slate-800 bg-blue-100 flex-col">
                <div class="flex flex-row gap-y-2">
                    { for Job::list_jobs().into_iter().map(|job| {
                        let callback_job = job.clone();
                        html! {
                            <button class="border border-slate-900 background-slate-100 p-2" onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::AddJob(callback_job.clone()))}>
                                {job.button_text}
                            </button>
                        }
                    })}
                    <div class="border border-slate-900 background-slate-100 p-2 ml-4">
                        {format!("Total jobs ran: {}", self.applied_buttons.len())}
                    </div>
                </div>
                <div class="flex flex-row gap-y-2">
                    { for (apply_jobs_partial(starting_resources(), self.applied_buttons.clone())).iter().map(|resource| {
                        html! {
                            <div class="border border-slate-900 p-2">
                                {format!("{}: {}", resource.0.to_string(), resource.1)}
                            </div>
                        }
                    })}

                </div>
                <div class="flex flex-row gap-y-2">
                    {&self.user_error}
                </div>
                <div class="flex flex-col gap-y-2">
                    { for get_job_success_zip(starting_resources(), self.applied_buttons.clone()).into_iter().enumerate().map(|(index, (job, is_ok))| {
                        html! {
                            <button class={
                                "border border-slate-900 background-slate-100 p-2 flex flex-row items-center justify-center"
                            } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::RemoveHistory(index))}>
                                <div class={if is_ok {"bg-red-600 w-1 h-6 p-1 mr-2 collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
                                {job.button_text}
                            </button>
                        }
                    })}
                </div>
            </div>
            <div class="w-1/5 h-screen border border-slate-800 bg-blue-300">
            </div>
        </div>
        }
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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
        }
    }
}
