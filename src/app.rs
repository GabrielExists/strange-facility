// #![cfg(target_arch = "wasm32")]

use std::cmp::max;
use std::collections::BTreeMap;
use yew::prelude::*;
use crate::jobs::*;
use gloo::timers::callback::{Timeout};

pub struct App {
    pub history: Vec<HistoryStep>,
    pub redo_queue: Vec<HistoryStep>,
    pub last_combination: CombinationResult,
    pub selected_resource: Option<Resource>,
    pub discovered_jobs: Vec<Job>,

    pub animation_resources: Option<(Timeout, Vec<Resource>)>,

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
    job: Job,
    output: JobOutput,
    resource_list: Vec<(Resource, i64)>,
    resource_tool_list: Vec<ResourceTool>,
    index: usize,
}

pub struct ResourceTool {
    resource_pair: (Resource, i64),
    status: ResourceToolStatus,
}

pub enum ResourceToolStatus {
    Standard,
    Changed,
    Removed,
}

pub struct CurrentResource {
    resource: Resource,
    amount: i64,
    row: usize,
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

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let _history = history_from_combinations(vec![
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Martha, Resource::Fish),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Scrap, Resource::Forge),
            (Resource::SpareParts, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::SpareParts, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Martha, Resource::Fish),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Net),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Submarine, Resource::Claw),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Scrap, Resource::Forge),
            (Resource::Submarine, Resource::SpareParts),
        ]);
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
        html! {
        <div class="flex flex-row">
            <div class="p-2 border border-slate-800 bg-blue-100 flex-col gap-y-2">
                <div class="p-4 te">
                    {
                        match self.view_cache.game_state {
                            GameState::Playing => {
                                "This game is very WIP, but there's some stuff to play around with. Actions marked in red failed due to lack of resources. Try to successfully depart in the fewest days possible.".to_string()
                            }
                            GameState::Won{ spent_days } => {
                                format!("You successfully escaped! It took you {} days!", spent_days)
                            }
                        }
                    }
                </div>
                // List resources
                <div class="border border-slate-900 p-2">
                    <div>{"Combine two resources!"}</div>
                { for (0..=self.view_cache.max_row).rev().map(|current_row| {
                    html!{
                    <div class="flex gap-1 my-1">
                { for self
                    .view_cache
                    .current_resources
                    .iter()
                    .filter(|current_resource|{ current_resource.row == current_row })
                    .map(|current_resource|
                {
                    let resource = current_resource.resource.clone();
                    let flashing = self.animation_resources.as_ref().map(|(_interval, flashing_resources)| {flashing_resources.contains(&resource)}).unwrap_or(false);
                    let selected = self.selected_resource.map(|selected| selected == resource).unwrap_or(false);
                    if selected || flashing {
                    html! {
                        <button
                            onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::SelectResource(resource.clone()))}
                            class="bg-blue-500 text-slate-100 border border-slate-900 rounded-md p-2"
                        >
                            {format!("{}: {}", resource.long_name(), current_resource.amount)}
                        </button>
                    }
                    } else {
                    html! {
                        <button
                            onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::SelectResource(resource.clone()))}
                            class="border border-slate-900 rounded-md p-2 active:bg-blue-500 active:text-slate-100"
                        >
                            {format!("{}: {}", resource.long_name(), current_resource.amount)}
                        </button>
                        }
                    }
                })}
                    </div>
                    }
                })}
                </div>
                // List available jobs
                <div class="flex flex-row flex-wrap gap-y-2">
                { for self.discovered_jobs.iter().map(|job| {
                    let callback_job = job.clone();
                    html! {
                    <button class="border border-slate-900 background-slate-100 p-2 rounded-md mr-1 mt-2" onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::AddJob(callback_job.clone()))}>
                        {job.button_text}
                    </button>
                    }
                })}
                </div>
                // Current error
                {
                    if self.programmer_error.is_some() {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-purple-600 my-2"> {&self.programmer_error} </div> }
                    } else if let CombinationResult::Text(text) = &self.last_combination {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-blue-500 my-2"> {text} </div> }
                    } else if let CombinationResult::Job(_, Some(text)) = &self.last_combination {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-blue-500 my-2"> {text} </div> }
                    } else if self.view_cache.user_error.is_some() {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-red-600 my-2"> {&self.view_cache.user_error} </div> }
                    } else {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-gray-900 my-2"> {"-"} </div> }
                    }
                }
                <div class="flex gap-x-2">
                    // Total jobs
                    <div class="border border-slate-900 background-slate-100 p-2">
                        {format!("Total days spent: {}", self.view_cache.total_days)}
                    </div>
                    <button
                        disabled={self.history.is_empty()}
                        class={if self.history.is_empty() {
                            "border background-slate-100 p-2 rounded-md border-slate-400 text-slate-400"
                        } else {
                            "border background-slate-100 p-2 rounded-md border-slate-900"
                        }}
                        onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::Undo())}>
                        {"Undo"}
                    </button>
                    <button
                        disabled={self.redo_queue.is_empty()}
                        class={if self.redo_queue.is_empty() {
                            "border border-slate-400 p-2 rounded-md text-slate-400 background-slate-100"
                        } else {
                            "border border-slate-900 p-2 rounded-md background-slate-100"
                        }}
                        onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::Redo())}>
                        {"Redo"}
                    </button>
                </div>
                // Table of resource steps and jobs
                <table class="table-auto m-2 mt-3">
                    <thead>
                        <tr>
                            <td class="border border-slate-900 background-slate-100 p-2">
                                {"Edit"}
                            </td>
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
                            <td class="border border-slate-900 background-slate-100 p-2">
                                {"Other"}
                            </td>
                        </tr>
                    </thead>
                    <tbody>
                    { for self.view_cache.job_rows.iter().rev().map(|job_row| {
                        let index = job_row.index;
                        let failing_resources = job_row.output.failing_resources();
                        let job = &job_row.job;
                        html! {
                        <tr>
                            <td class="border border-slate-900 background-slate-100 px-1">
                                <div class="flex">
                            { if job.removable {
                                html! {
                                    <button class={
                                        "border border-slate-900 background-slate-100 p-2 py-1 pl-1 flex flex-row items-center justify-center rounded-md"
                                    } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::AddOne(index))}>
                                        {"+1"}
                                    </button>
                                }
                            } else {
                                html!{<></>}
                            }}
                            { if job.removable && job.instances != 1 {
                                html! {
                                    <button class={
                                        "ml-2 border border-slate-900 background-slate-100 p-2 py-1 pl-1 flex flex-row items-center justify-center rounded-md"
                                    } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::RemoveOne(index))}>
                                        {"-1"}
                                    </button>
                                }
                            } else {
                                html!{<></>}
                            }}
                            { if job.removable {
                                html! {
                                    <button class={
                                        "ml-2 border border-slate-900 background-slate-100 px-2 py-1 flex flex-row items-center justify-center rounded-md"
                                    } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::RemoveCluster(index))}>
                                        {"X"}
                                    </button>
                                }
                            } else {
                                html!{<></>}
                            }}
                                </div>
                            </td>
                            <td class="border border-slate-900 background-slate-100">
                            // { if job.removable {
                            //     html! {
                            //     <button class={
                            //         "border border-slate-900 background-slate-100 p-2 flex flex-row items-center justify-center rounded-md"
                            //     } onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::RemoveOne(index))}>
                            //         <div class={if *is_ok {"bg-red-600 w-1 h-6 p-1 mr-2 collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
                            //         {if job.instances != 1 { format!("{}x ", job.instances)} else {String::new()}}
                            //         {job.button_text}
                            //     </button>
                            //     }
                            // } else {
                            //     html! {
                                <div class={ "background-slate-100 p-2 flex flex-row items-center justify-center" }>
                                    <div class={if job_row.output.is_ok() {"bg-red-600 w-1 h-6 p-1 mr-2 collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} ></div>
                                    <div class={if job_row.output.is_ok() {""} else {"line-through"}}>
                                        {if job.instances != 1 { format!("{}x ", job.instances)} else {String::new()}}
                                        {job.button_text}
                                    </div>

                                </div>
                            // }
                            // } }
                            </td>
                            { for job_row.resource_list.iter().map(|(resource, amount)| {
                                html! {
                            <td class="border border-slate-900 background-slate-100 p-2">
                                <div class="flex">
                                    <div class={if !failing_resources.contains(resource) {"collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} >
                                    </div>
                                    <div class={if job_row.output.get_changed_resources().contains(resource) {"underline"} else {""}}>
                                        {amount.to_string()}
                                    </div>
                                </div>
                            </td>
                                }
                            })}
                            <td class="border border-slate-900 background-slate-100 p-2">
                            { for job_row.resource_tool_list.iter().map(|tool| {
                                let (resource, amount) = tool.resource_pair;
                                let (text, class_string) = match tool.status {
                                    ResourceToolStatus::Standard => {
                                        let text = if amount == 1 {
                                            format!("{}", resource)
                                        } else {
                                            format!("{}: {}", resource, amount)
                                        };
                                        ( text, "" )
                                    }
                                    ResourceToolStatus::Changed => {
                                        let text = if amount == 1 {
                                            format!("{}", resource)
                                        } else {
                                            format!("{}: {}", resource, amount)
                                        };
                                        ( text, "underline" )
                                    },
                                    ResourceToolStatus::Removed => {
                                        let text = format!("{}", resource);
                                        ( text, "line-through" )
                                    }
                                };
                                html! {
                                <div class="flex">
                                    <div class={if !failing_resources.contains(&resource) {"collapse"} else {"bg-red-600 w-1 h-6 p-1 mr-2 visible"}} >
                                    </div>
                                    <div class={class_string}>
                                        {text}
                                    </div>
                                </div>
                                }
                            })}
                            </td>
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

fn history_from_combinations(combinations: Vec<(Resource, Resource)>) -> Vec<HistoryStep> {
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
    fn add_job(&mut self, job: Job) {
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
    fn apply_combination(&mut self, first_resource: &Resource, second_resource: &Resource) {
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
            let changed = changed.map(|changed | changed.contains(resource)).unwrap_or(false);
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
                max_row = max(max_row, row);
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
