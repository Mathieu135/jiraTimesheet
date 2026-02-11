mod config;
mod jira;
mod timer;

use config::{ConfigState, get_config, save_config};
use jira::{JiraClient, JiraProject, JiraTicketDetail, JiraTransition, TimesheetEntry};
use timer::{HistoryEntry, TimerState, get_history, get_timers, pause_timer, resume_timer, set_timer_elapsed, start_timer, stop_timer};

use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

fn build_client(config_state: &tauri::State<'_, ConfigState>) -> Result<JiraClient, String> {
    let config = config_state.config.lock().map_err(|e| e.to_string())?;
    if config.jira_url.is_empty() || config.email.is_empty() || config.api_token.is_empty() {
        return Err("Jira not configured. Please set URL, email and API token in Settings.".to_string());
    }
    Ok(JiraClient::new(&config.jira_url, &config.email, &config.api_token))
}

#[tauri::command]
async fn list_projects(
    config_state: tauri::State<'_, ConfigState>,
) -> Result<Vec<JiraProject>, String> {
    let client = build_client(&config_state)?;
    client.list_projects().await
}

#[tauri::command]
async fn search_tickets(
    config_state: tauri::State<'_, ConfigState>,
    project_key: String,
) -> Result<Vec<jira::JiraTicket>, String> {
    let client = build_client(&config_state)?;
    client.search_project_tickets(&project_key).await
}

#[tauri::command]
async fn get_issue_detail(
    config_state: tauri::State<'_, ConfigState>,
    issue_key: String,
) -> Result<JiraTicketDetail, String> {
    let client = build_client(&config_state)?;
    client.get_issue_detail(&issue_key).await
}

#[tauri::command]
async fn get_transitions(
    config_state: tauri::State<'_, ConfigState>,
    issue_key: String,
) -> Result<Vec<JiraTransition>, String> {
    let client = build_client(&config_state)?;
    client.get_transitions(&issue_key).await
}

#[tauri::command]
async fn transition_issue(
    config_state: tauri::State<'_, ConfigState>,
    issue_key: String,
    transition_id: String,
) -> Result<(), String> {
    let client = build_client(&config_state)?;
    client.transition_issue(&issue_key, &transition_id).await
}

#[tauri::command]
async fn get_my_worklogs(
    config_state: tauri::State<'_, ConfigState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<TimesheetEntry>, String> {
    let client = build_client(&config_state)?;
    client.get_my_worklogs(&start_date, &end_date).await
}

fn record_history(timer_state: &tauri::State<'_, TimerState>, timer: &timer::Timer, logged: bool) {
    if let Ok(mut history) = timer_state.history.lock() {
        history.push(HistoryEntry {
            issue_key: timer.issue_key.clone(),
            summary: timer.summary.clone(),
            elapsed_seconds: timer.elapsed_seconds,
            logged,
            stopped_at: chrono::Utc::now(),
        });
    }
}

#[tauri::command]
async fn discard_timer(
    timer_state: tauri::State<'_, TimerState>,
    timer_id: u32,
) -> Result<(), String> {
    let timer = stop_timer(timer_state.clone(), timer_id)?;
    record_history(&timer_state, &timer, false);
    Ok(())
}

#[tauri::command]
async fn stop_and_log(
    timer_state: tauri::State<'_, TimerState>,
    config_state: tauri::State<'_, ConfigState>,
    timer_id: u32,
) -> Result<u64, String> {
    let timer = stop_timer(timer_state.clone(), timer_id)?;

    if timer.elapsed_seconds < 60 {
        return Err("Worklog must be at least 1 minute".to_string());
    }

    let client = build_client(&config_state)?;

    client
        .log_worklog(&timer.issue_key, timer.elapsed_seconds)
        .await?;

    record_history(&timer_state, &timer, true);

    Ok(timer.elapsed_seconds)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .manage(TimerState::new())
        .manage(ConfigState::new())
        .invoke_handler(tauri::generate_handler![
            list_projects,
            search_tickets,
            get_issue_detail,
            get_transitions,
            transition_issue,
            get_my_worklogs,
            start_timer,
            pause_timer,
            resume_timer,
            stop_timer,
            set_timer_elapsed,
            discard_timer,
            get_timers,
            get_history,
            stop_and_log,
            get_config,
            save_config,
        ])
        .setup(|app| {
            // System tray
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Jira Timesheet")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Global shortcut Ctrl+Shift+T
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["ctrl+shift+t"])?
                        .with_handler(|app, shortcut, event| {
                            if event.state == ShortcutState::Pressed
                                && shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyT)
                            {
                                if let Some(window) = app.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        })
                        .build(),
                )?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
