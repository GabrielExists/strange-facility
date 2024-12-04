// #![cfg(target_arch = "wasm32")]

use std::cmp::max;
use std::collections::BTreeMap;
use yew::prelude::*;
use crate::jobs::*;
use gloo::timers::callback::Timeout;
use crate::game::{attributes, WIN_JOB_ID};
use crate::view_logic::*;

pub struct App {
    pub history: Vec<HistoryStep>,
    pub redo_queue: Vec<HistoryStep>,
    pub last_combination: CombinationResult,
    pub selected_resource: Option<Resource>,
    pub discovered_jobs: Vec<Job>,

    pub animation_resources: Option<(Timeout, Vec<Resource>)>,
    pub displayed_job: Option<Job>,

    pub view_cache: ViewCache,
    pub programmer_error: Option<String>,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    AddJob(Job),
    AddOne(usize),
    RemoveOne(usize),
    RemoveCluster(usize),
    SelectResource(Resource),
    AnimationResourceBlinkEnd(),
    Undo(),
    Redo(),
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let history = Vec::new();
        let result = App::create_view_cache(&history);
        match result {
            Ok(view_cache) => {
                Self {
                    history,
                    redo_queue: vec![],
                    last_combination: CombinationResult::Nothing,
                    selected_resource: None,
                    discovered_jobs: vec![],
                    animation_resources: None,
                    displayed_job: None,
                    view_cache,
                    programmer_error: None,
                }
            }
            Err(error) => {
                Self {
                    history: vec![],
                    redo_queue: vec![],
                    last_combination: CombinationResult::Nothing,
                    selected_resource: None,
                    discovered_jobs: vec![],
                    animation_resources: None,
                    displayed_job: None,
                    view_cache: ViewCache {
                        job_rows: Vec::new(),
                        seen_resources: vec![],
                        current_resources: vec![],
                        total_days: 0,
                        user_error: None,
                        max_row: 0,
                        game_state: GameState::Playing,
                    },
                    programmer_error: Some(error),
                }
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::AddJob(job) => {
                let handle = {
                    let link = ctx.link().clone();
                    Timeout::new(300, move || link.send_message(AppMessage::AnimationResourceBlinkEnd()))
                };
                self.animation_resources = Some((handle, job.combination_resources.clone()));
                self.add_job(job);
                self.last_combination = CombinationResult::Nothing;
                true
            }
            AppMessage::AddOne(index) => {
                self.history.push(HistoryStep::AddOne(index));
                self.redo_queue.clear();
                self.refresh_view_cache();
                true
            }
            AppMessage::RemoveOne(index) => {
                self.history.push(HistoryStep::RemoveOne(index));
                self.redo_queue.clear();
                self.refresh_view_cache();
                true
            }
            AppMessage::RemoveCluster(index) => {
                self.history.push(HistoryStep::RemoveCluster(index));
                self.redo_queue.clear();
                self.refresh_view_cache();
                true
            }
            AppMessage::SelectResource(resource) => {
                match self.selected_resource {
                    None => {
                        self.selected_resource = Some(resource);
                        true
                    }
                    Some(selected_resource) => {
                        if selected_resource != resource {
                            self.apply_combination(&selected_resource, &resource);
                            self.selected_resource = None;
                            true
                        } else {
                            false
                        }
                    }
                }
            }
            AppMessage::AnimationResourceBlinkEnd() => {
                self.animation_resources = None;
                true
            }
            AppMessage::Undo() => {
                match self.history.pop() {
                    None => {}
                    Some(step) => {
                        self.redo_queue.push(step);
                    }
                }
                self.refresh_view_cache();
                true
            }
            AppMessage::Redo() => {
                match self.redo_queue.pop() {
                    None => {}
                    Some(step) => {
                        self.history.push(step);
                    }
                }
                self.refresh_view_cache();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        crate::view::view(self, ctx)
    }
}

