use yew::prelude::*;
use crate::app::*;
use crate::jobs::*;
use crate::view_logic::*;

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

pub fn merge(base: &'static str, additional: Classes) -> Classes {
    let mut base = class_string(base);
    base.extend(additional);
    base
}

pub fn view(app: &App, ctx: &Context<App>) -> Html {
    html! {
        <div class="flex flex-row">
            <div class="p-2 border border-slate-800 bg-blue-100 flex-col gap-y-2">
                <div class="p-4 te">
                    {
                        match app.view_cache.game_state {
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
                { for app.view_cache.current_resources.iter().map(|row| {
                    html!{
                    <div class="flex gap-1 my-1">
                        { for row.iter().map(|current_resource| {
                            // let resource = current_resource.resource.clone();
                            let resource = current_resource.resource.clone();
                            html! {
                        <button
                            onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::SelectResource(resource))}
                            class={merge("border rounded-md p-2", current_resource.classes.clone())}
                        >
                            {format!("{}: {}", current_resource.resource.long_name(), current_resource.amount)}
                        </button>
                            }
                        })}
                    </div>
                    }
                })}
                // { for (0..=app.view_cache.max_row).rev().map(|current_row| {
                //     html!{
                //     <div class="flex gap-1 my-1">
                // { for app
                //     .view_cache
                //     .current_resources
                //     .iter()
                //     .filter(|current_resource|{ current_resource.row == current_row })
                //     .map(|current_resource|
                // {
                //     let resource = current_resource.resource.clone();
                //     let flashing = app.animation_resources.as_ref().map(|(_interval, flashing_resources)| {flashing_resources.contains(&resource)}).unwrap_or(false);
                //     let selected = app.selected_resource.map(|selected| selected == resource).unwrap_or(false);
                //     if selected || flashing {
                //     html! {
                //         <button
                //             onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::SelectResource(resource.clone()))}
                //             class="bg-blue-500 text-slate-100 border border-slate-900 rounded-md p-2"
                //         >
                //             {format!("{}: {}", resource.long_name(), current_resource.amount)}
                //         </button>
                //     }
                //     } else {
                //     html! {
                //         <button
                //             onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::SelectResource(resource.clone()))}
                //             class="border border-slate-900 rounded-md p-2 active:bg-blue-500 active:text-slate-100"
                //         >
                //             {format!("{}: {}", resource.long_name(), current_resource.amount)}
                //         </button>
                //         }
                //     }
                // })}
                //     </div>
                //     }
                // })}
                </div>
                <div class="md:flex md:flex-row">
                    // List available jobs
                    <div class="flex flex-row flex-wrap gap-y-2 md:w-3/5">
                    { for app.state.discovered_jobs.iter().map(|job| {
                        let callback_job = job.clone();
                        html! {
                        <button class="border border-slate-900 background-slate-100 p-2 rounded-md mr-1 mt-2" onclick={ctx.link().callback(move |_event: MouseEvent| AppMessage::AddJob(callback_job.clone()))}>
                            {job.button_text}
                        </button>
                        }
                    })}
                    </div>
                    <div class="p-1 border-2 border-slate-900 mt-2 md:w-2/5">
                        <div class="flex flex-row gap-2">
                            <div class="p-1 border border-slate-900">
                                {"Icon"}
                            </div>
                            <div class="py-1 text-xl">
                                {"Title"}
                            </div>
                        </div>
                        <div class="p-1">
                            {"Subtitle"}
                        </div>
                        <div class="flex flex-row flex-wrap gap-2">
                            <div class="p-1 px-2 border border-slate-900">
                                {"Resource: 4"}
                            </div>
                            <div class="p-1 px-2 border border-slate-900">
                                {"Resource: 5"}
                            </div>
                            <div class="p-1 px-2 border border-slate-900">
                                {"Resource: 6"}
                            </div>
                        </div>
                    </div>
                </div>
                // Current error
                {
                    if app.programmer_error.is_some() {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-purple-600 my-2"> {&app.programmer_error} </div> }
                    } else if let CombinationResult::Text(text) = &app.state.last_combination {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-blue-500 my-2"> {text} </div> }
                    } else if let CombinationResult::Job(_, Some(text)) = &app.state.last_combination {
                        html!{ <div class="flex flex-row gap-y-2 p-2 border-2 border-blue-500 my-2"> {text} </div> }
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
                    { for app.view_cache.resource_headings.iter().map(|resource| {
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
                    { for app.view_cache.job_rows.iter().rev().map(|job_row| {
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
                            { if job.removable {
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
                            { if job.removable && job.instances != 1 {
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
