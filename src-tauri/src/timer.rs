use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub id: u32,
    pub issue_key: String,
    pub summary: String,
    pub started_at: DateTime<Utc>,
    pub elapsed_seconds: u64,
    pub paused: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_start: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub issue_key: String,
    pub summary: String,
    pub elapsed_seconds: u64,
    pub logged: bool,
    pub stopped_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct TimerState {
    pub timers: Mutex<Vec<Timer>>,
    pub history: Mutex<Vec<HistoryEntry>>,
    pub next_id: Mutex<u32>,
}

impl TimerState {
    pub fn new() -> Self {
        Self {
            timers: Mutex::new(Vec::new()),
            history: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

#[tauri::command]
pub fn start_timer(
    state: tauri::State<'_, TimerState>,
    issue_key: String,
    summary: String,
) -> Result<Timer, String> {
    let mut timers = state.timers.lock().map_err(|e| e.to_string())?;
    let mut next_id = state.next_id.lock().map_err(|e| e.to_string())?;

    // Don't start duplicate timer for same ticket
    if timers.iter().any(|t| t.issue_key == issue_key) {
        return Err(format!("Timer already running for {}", issue_key));
    }

    let timer = Timer {
        id: *next_id,
        issue_key,
        summary,
        started_at: Utc::now(),
        elapsed_seconds: 0,
        paused: false,
        pause_start: None,
    };

    *next_id += 1;
    timers.push(timer.clone());
    Ok(timer)
}

#[tauri::command]
pub fn pause_timer(
    state: tauri::State<'_, TimerState>,
    timer_id: u32,
) -> Result<(), String> {
    let mut timers = state.timers.lock().map_err(|e| e.to_string())?;

    let timer = timers
        .iter_mut()
        .find(|t| t.id == timer_id)
        .ok_or("Timer not found")?;

    if timer.paused {
        return Err("Timer already paused".to_string());
    }

    // Accumulate elapsed time before pausing
    let now = Utc::now();
    let running_since = timer.pause_start.unwrap_or(timer.started_at);
    let additional = (now - running_since).num_seconds().max(0) as u64;
    timer.elapsed_seconds += additional;
    timer.paused = true;
    timer.pause_start = Some(now);

    Ok(())
}

#[tauri::command]
pub fn resume_timer(
    state: tauri::State<'_, TimerState>,
    timer_id: u32,
) -> Result<(), String> {
    let mut timers = state.timers.lock().map_err(|e| e.to_string())?;

    let timer = timers
        .iter_mut()
        .find(|t| t.id == timer_id)
        .ok_or("Timer not found")?;

    if !timer.paused {
        return Err("Timer is not paused".to_string());
    }

    timer.paused = false;
    timer.pause_start = Some(Utc::now()); // Mark resume time

    Ok(())
}

#[tauri::command]
pub fn stop_timer(
    state: tauri::State<'_, TimerState>,
    timer_id: u32,
) -> Result<Timer, String> {
    let mut timers = state.timers.lock().map_err(|e| e.to_string())?;

    let pos = timers
        .iter()
        .position(|t| t.id == timer_id)
        .ok_or("Timer not found")?;

    let mut timer = timers.remove(pos);

    // Calculate final elapsed time
    if !timer.paused {
        let now = Utc::now();
        let running_since = timer.pause_start.unwrap_or(timer.started_at);
        let additional = (now - running_since).num_seconds().max(0) as u64;
        timer.elapsed_seconds += additional;
    }

    Ok(timer)
}

#[tauri::command]
pub fn get_timers(state: tauri::State<'_, TimerState>) -> Result<Vec<Timer>, String> {
    let timers = state.timers.lock().map_err(|e| e.to_string())?;

    let now = Utc::now();
    let result: Vec<Timer> = timers
        .iter()
        .map(|t| {
            let mut timer = t.clone();
            if !timer.paused {
                let running_since = timer.pause_start.unwrap_or(timer.started_at);
                let additional = (now - running_since).num_seconds().max(0) as u64;
                timer.elapsed_seconds += additional;
            }
            timer
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub fn get_history(state: tauri::State<'_, TimerState>) -> Result<Vec<HistoryEntry>, String> {
    let history = state.history.lock().map_err(|e| e.to_string())?;
    Ok(history.iter().rev().cloned().collect())
}
