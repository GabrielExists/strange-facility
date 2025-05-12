// #![cfg(target_arch = "wasm32")]

use yew::prelude::*;
use crate::jobs::*;
use gloo::timers::callback::Timeout;
use crate::core::job::Job;
use crate::game::Resource;
use crate::view::view_logic::{GameState, HistoryStep, ViewCache};
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
    pub displayed_job: Option<Job>,
}

#[derive(Clone, Debug)]
pub enum AppMessage {
    AddJob(Job),
    AddOne(usize),
    RemoveOne(usize),
    RemoveCluster(usize),
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
                // let handle = {
                //     let link = ctx.link().clone();
                //     Timeout::new(300, move || link.send_message(AppMessage::AnimationResourceBlinkEnd()))
                // };
                self.add_job(job);
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

