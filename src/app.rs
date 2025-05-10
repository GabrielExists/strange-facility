// #![cfg(target_arch = "wasm32")]

use yew::prelude::*;
use crate::jobs::*;
use gloo::timers::callback::Timeout;
use crate::view_logic::*;

pub struct App {
    pub state: State,

    pub view_cache: ViewCache,
    pub programmer_error: Option<String>,
}

pub struct State {
    pub history: Vec<HistoryStep>,
    pub redo_queue: Vec<HistoryStep>,
    pub discovered_jobs: Vec<Job>,

    // State for the view
    pub last_combination: CombinationResult,
    pub selected_resource: Option<Resource>,
    pub animation_resources: Option<(Timeout, Vec<Resource>)>,
    pub displayed_job: Option<Job>,
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
        let state = State {
            history: vec![],
            redo_queue: vec![],
            discovered_jobs: vec![],
            last_combination: CombinationResult::Nothing,
            selected_resource: None,
            animation_resources: None,
            displayed_job: None,
        };
        let result = App::create_view_cache(&state);
        match result {
            Ok(view_cache) => {
                Self {
                    state,
                    view_cache,
                    programmer_error: None,
                }
            }
            Err(error) => {
                Self {
                    state,
                    view_cache: ViewCache {
                        job_rows: Vec::new(),
                        current_resources: vec![],
                        total_days: 0,
                        user_error: None,
                        game_state: GameState::Playing,
                        resource_headings: vec![],
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
                self.state.animation_resources = Some((handle, job.combination_resources.clone()));
                self.add_job(job);
                self.state.last_combination = CombinationResult::Nothing;
                true
            }
            AppMessage::AddOne(index) => {
                self.state.history.push(HistoryStep::AddOne(index));
                self.state.redo_queue.clear();
                self.refresh_view_cache();
                true
            }
            AppMessage::RemoveOne(index) => {
                self.state.history.push(HistoryStep::RemoveOne(index));
                self.state.redo_queue.clear();
                self.refresh_view_cache();
                true
            }
            AppMessage::RemoveCluster(index) => {
                self.state.history.push(HistoryStep::RemoveCluster(index));
                self.state.redo_queue.clear();
                self.refresh_view_cache();
                true
            }
            AppMessage::SelectResource(resource) => {
                match self.state.selected_resource {
                    None => {
                        self.state.selected_resource = Some(resource);
                        self.refresh_view_cache();
                        true
                    }
                    Some(selected_resource) => {
                        if selected_resource != resource {
                            self.apply_combination(&selected_resource, &resource);
                            self.state.selected_resource = None;
                            self.refresh_view_cache();
                            true
                        } else {
                            // Deselect
                            self.state.selected_resource = None;
                            self.refresh_view_cache();
                            true
                        }
                    }
                }
            }
            AppMessage::AnimationResourceBlinkEnd() => {
                self.state.animation_resources = None;
                self.refresh_view_cache();
                true
            }
            AppMessage::Undo() => {
                match self.state.history.pop() {
                    None => {}
                    Some(step) => {
                        self.state.redo_queue.push(step);
                    }
                }
                self.refresh_view_cache();
                true
            }
            AppMessage::Redo() => {
                match self.state.redo_queue.pop() {
                    None => {}
                    Some(step) => {
                        self.state.history.push(step);
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

