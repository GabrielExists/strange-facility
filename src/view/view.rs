use yew::prelude::*;
use crate::app::*;
use crate::jobs::*;

pub fn class_string(text: &'static str) -> Classes{
    let mut split = text.split(" ");
    if let Some(first) = split.next() {
        split.into_iter().fold(classes!(first), |mut class, substring| {
            class.extend(classes!(substring));
            class
        })
    } else {
        Classes::new()
    }
}

#[allow(dead_code)]
pub fn class_merge(base: &'static str, additional: Classes) -> Classes {
    let mut base = class_string(base);
    base.extend(additional);
    base
}

pub fn view(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="flex flex-row">
            <div class="p-2 border border-slate-800 bg-blue-100 flex-col gap-y-2">
                <div class="md:flex md:flex-row">
                    // List available jobs
                    <div class="flex flex-row flex-wrap gap-y-2 md:w-3/5">
                    { for app.state.discovered_jobs.iter().map(|job| {
                        let callback_job = job.clone();
                        html! {
                        <button class="border border-slate-900 background-slate-100 p-2 rounded-md mr-1 mt-2" onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::AddJob(callback_job.clone()))}>
                            {job.short_text}
                        </button>
                        }
                    })}
                    </div>
                { if let Some(job) = &app.state.displayed_job {
                    html! {
                    <div class="p-1 border-2 border-slate-900 mt-2 md:w-2/5">
                        <div class="flex flex-row gap-2">
                            <div class="p-1 border border-slate-900">
                                {"Icon"}
                            </div>
                            <div class="py-1 text-xl">
                                {job.short_text}
                            </div>
                        </div>
                        <div class="p-1">
                            {"Subtitle"}
                        </div>
                    { for job.end_deltas.iter().map(|delta_row|{
                        html! {
                        <div class="flex flex-row flex-wrap gap-2">
                        { for delta_row.iter().map(|(resource, amount)| {
                            html! {
                            <div class="p-1 px-2 border border-slate-900">
                                {format!("{}: {}", resource, amount)}
                            </div>
                            }
                        })}
                        </div>
                        }
                    })}
                    </div>
                    }
                } else {
                    html! {
                        <></>
                    }
                }}
                </div>
                // Current error
                {
                    if app.programmer_error.is_some() {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-purple-600 my-2"> {&app.programmer_error} </div> }
                    // } else if let CombinationResult::Text(text) = &app.state.last_combination {
                    //     html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-blue-500 my-2"> {text} </div> }
                    // } else if let CombinationResult::Job(_, Some(text)) = &app.state.last_combination {
                    //     html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-blue-500 my-2"> {text} </div> }
                    } else if app.view_cache.user_error.is_some() {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-red-600 my-2"> {&app.view_cache.user_error} </div> }
                    } else {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-gray-900 my-2"> {"-"} </div> }
                    }
                }
                <div class="flex gap-x-2">
                    // Total jobs
                    <div class="border border-slate-900 background-slate-100 p-2">
                        {format!("Total days spent: {}", app.view_cache.total_days)}
                    </div>
                    <button
                        disabled={app.state.history.is_empty()}
                        class={if app.state.history.is_empty() {
                            "border background-slate-100 p-2 rounded-md border-slate-400 text-slate-400"
                        } else {
                            "border background-slate-100 p-2 rounded-md border-slate-900"
                        }}
                        onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::Undo())}>
                        {"Undo"}
                    </button>
                    <button
                        disabled={app.state.redo_queue.is_empty()}
                        class={if app.state.redo_queue.is_empty() {
                            "border border-slate-400 p-2 rounded-md text-slate-400 background-slate-100"
                        } else {
                            "border border-slate-900 p-2 rounded-md background-slate-100"
                        }}
        //http://127.0.0.1:1420/strange-facility-4/
                        onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::Redo())}>
                        {"Redo"}
                    </button>
                </div>
                <div class="border grid grid-flow-row grid-cols-[repeat(20_,20px)] grid-rows-4 gap-[2px]">
                    <div class="border border-slate-400 p-1">{"1"}</div>
                    <div class="border border-slate-400 p-1 col-span-3">{"5"}</div>
                </div>
            </div>
        </div>
    }
}
