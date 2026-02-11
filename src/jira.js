const { invoke } = window.__TAURI__.core;

export async function listProjects() {
  return invoke("list_projects");
}

export async function searchTickets(projectKey) {
  return invoke("search_tickets", { projectKey });
}

export async function startTimer(issueKey, summary) {
  return invoke("start_timer", { issueKey, summary });
}

export async function pauseTimer(timerId) {
  return invoke("pause_timer", { timerId });
}

export async function resumeTimer(timerId) {
  return invoke("resume_timer", { timerId });
}

export async function stopAndLog(timerId) {
  return invoke("stop_and_log", { timerId });
}

export async function stopTimer(timerId) {
  return invoke("stop_timer", { timerId });
}

export async function discardTimer(timerId) {
  return invoke("discard_timer", { timerId });
}

export async function getHistory() {
  return invoke("get_history");
}

export async function getTimers() {
  return invoke("get_timers");
}

export async function getConfig() {
  return invoke("get_config");
}

export async function saveConfig(jiraUrl, email, apiToken) {
  return invoke("save_config", { jiraUrl, email, apiToken });
}
