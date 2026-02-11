const { invoke } = window.__TAURI__.core;

export async function listProjects() {
  return invoke("list_projects");
}

export async function searchTickets(projectKey) {
  return invoke("search_tickets", { projectKey });
}

export async function getIssueDetail(issueKey) {
  return invoke("get_issue_detail", { issueKey });
}

export async function getTransitions(issueKey) {
  return invoke("get_transitions", { issueKey });
}

export async function transitionIssue(issueKey, transitionId) {
  return invoke("transition_issue", { issueKey, transitionId });
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

export async function setTimerElapsed(timerId, elapsedSeconds) {
  return invoke("set_timer_elapsed", { timerId, elapsedSeconds });
}

export async function getHistory() {
  return invoke("get_history");
}

export async function getTimers() {
  return invoke("get_timers");
}

export async function getMyWorklogs(startDate, endDate) {
  return invoke("get_my_worklogs", { startDate, endDate });
}

export async function getConfig() {
  return invoke("get_config");
}

export async function saveConfig(jiraUrl, email, apiToken) {
  return invoke("save_config", { jiraUrl, email, apiToken });
}
