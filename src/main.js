import {
  listProjects,
  searchTickets,
  startTimer,
  pauseTimer,
  resumeTimer,
  stopAndLog,
  discardTimer,
  getTimers,
  getConfig,
  saveConfig,
} from "./jira.js";

// DOM elements
const mainPanel = document.getElementById("main-panel");
const settingsPanel = document.getElementById("settings-panel");
const settingsBtn = document.getElementById("settings-btn");
const settingsCancel = document.getElementById("settings-cancel");
const settingsForm = document.getElementById("settings-form");
const projectsSection = document.getElementById("projects-section");
const projectsList = document.getElementById("projects-list");
const ticketsSection = document.getElementById("tickets-section");
const ticketsTitle = document.getElementById("tickets-title");
const ticketsList = document.getElementById("tickets-list");
const backBtn = document.getElementById("back-btn");
const projectsFilter = document.getElementById("projects-filter");
const ticketsFilter = document.getElementById("tickets-filter");
const timersList = document.getElementById("timers-list");

let refreshInterval = null;
let cachedProjects = [];
let cachedTickets = [];

// --- Init ---

async function init() {
  await loadConfig();
  await loadProjects();
  startRefreshLoop();
}

// --- Config / Settings ---

async function loadConfig() {
  try {
    const config = await getConfig();
    document.getElementById("jira-url").value = config.jira_url || "";
    document.getElementById("jira-email").value = config.email || "";
    document.getElementById("jira-token").value = config.api_token || "";
  } catch (_) {
    // Config not yet set
  }
}

function showSettings() {
  mainPanel.classList.add("hidden");
  settingsPanel.classList.remove("hidden");
}

function hideSettings() {
  settingsPanel.classList.add("hidden");
  mainPanel.classList.remove("hidden");
}

settingsBtn.addEventListener("click", showSettings);
settingsCancel.addEventListener("click", hideSettings);

settingsForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const jiraUrl = document.getElementById("jira-url").value.trim();
  const email = document.getElementById("jira-email").value.trim();
  const apiToken = document.getElementById("jira-token").value.trim();

  try {
    await saveConfig(jiraUrl, email, apiToken);
    showToast("Settings saved", "success");
    hideSettings();
    await loadProjects();
  } catch (err) {
    showToast(err, "error");
  }
});

// --- Projects ---

async function loadProjects() {
  projectsFilter.value = "";
  projectsList.innerHTML = '<div class="loading">Loading projects...</div>';
  try {
    cachedProjects = await listProjects();
    renderProjects(cachedProjects);
  } catch (err) {
    cachedProjects = [];
    projectsList.innerHTML =
      '<div class="empty-state">Configure Jira in Settings to see projects.</div>';
  }
}

function filterProjects() {
  const q = projectsFilter.value.toLowerCase();
  const filtered = cachedProjects.filter(
    (p) => p.key.toLowerCase().includes(q) || p.name.toLowerCase().includes(q)
  );
  renderProjects(filtered);
}

projectsFilter.addEventListener("input", filterProjects);

function renderProjects(projects) {
  if (projects.length === 0) {
    projectsList.innerHTML =
      '<div class="empty-state">No projects found.</div>';
    return;
  }

  projectsList.innerHTML = projects
    .map(
      (p) => `
    <button class="project-card" data-key="${escapeHtml(p.key)}">
      <span class="project-key">${escapeHtml(p.key)}</span>
      <span class="project-name">${escapeHtml(p.name)}</span>
    </button>
  `
    )
    .join("");
}

projectsList.addEventListener("click", (e) => {
  const card = e.target.closest(".project-card");
  if (!card) return;
  const key = card.dataset.key;
  const name = card.querySelector(".project-name").textContent;
  showTickets(key, name);
});

// --- Tickets ---

async function showTickets(projectKey, projectName) {
  projectsSection.classList.add("hidden");
  ticketsSection.classList.remove("hidden");
  ticketsFilter.value = "";
  ticketsTitle.textContent = `${projectKey} — ${projectName}`;
  ticketsList.innerHTML = '<div class="loading">Loading tickets...</div>';

  try {
    cachedTickets = await searchTickets(projectKey);
    renderTickets(cachedTickets);
  } catch (err) {
    cachedTickets = [];
    ticketsList.innerHTML = `<div class="empty-state">Error: ${escapeHtml(String(err))}</div>`;
  }
}

function filterTickets() {
  const q = ticketsFilter.value.toLowerCase();
  const filtered = cachedTickets.filter(
    (t) =>
      t.key.toLowerCase().includes(q) ||
      t.summary.toLowerCase().includes(q) ||
      t.status.toLowerCase().includes(q)
  );
  renderTickets(filtered);
}

ticketsFilter.addEventListener("input", filterTickets);

function renderTickets(tickets) {
  if (tickets.length === 0) {
    ticketsList.innerHTML =
      '<div class="empty-state">No tickets assigned to you in this project.</div>';
    return;
  }

  ticketsList.innerHTML = tickets
    .map(
      (t) => `
    <div class="ticket-row">
      <div class="ticket-info">
        <span class="ticket-key">${escapeHtml(t.key)}</span>
        <span class="ticket-summary">${escapeHtml(t.summary)}</span>
      </div>
      ${t.time_spent_seconds > 0 ? `<span class="ticket-logged-time" title="Time logged">${formatTime(t.time_spent_seconds)}</span>` : ""}
      <span class="ticket-status">${escapeHtml(t.status)}</span>
      <button class="btn btn-primary btn-sm" data-action="start-ticket" data-key="${escapeHtml(t.key)}" data-summary="${escapeHtml(t.summary)}">Start</button>
    </div>
  `
    )
    .join("");
}

ticketsList.addEventListener("click", async (e) => {
  const btn = e.target.closest("[data-action='start-ticket']");
  if (!btn) return;

  const key = btn.dataset.key;
  const summary = btn.dataset.summary;

  try {
    await startTimer(key, summary);
    showToast(`Timer started for ${key}`, "success");
    await refreshTimers();
  } catch (err) {
    showToast(err, "error");
  }
});

// --- Back button ---

backBtn.addEventListener("click", () => {
  ticketsSection.classList.add("hidden");
  projectsSection.classList.remove("hidden");
});

// --- Timers ---

function startRefreshLoop() {
  refreshTimers();
  refreshInterval = setInterval(refreshTimers, 1000);
}

async function refreshTimers() {
  try {
    const timers = await getTimers();
    renderTimers(timers);
  } catch (_) {
    // Ignore transient errors
  }
}

function formatTime(totalSeconds) {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  const pad = (n) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
}

function renderTimers(timers) {
  if (timers.length === 0) {
    timersList.innerHTML =
      '<div class="empty-state">No active timers.<br>Pick a project and start a ticket.</div>';
    return;
  }

  timersList.innerHTML = timers
    .map(
      (t) => `
    <div class="timer-card ${t.paused ? "timer-paused" : ""}" data-id="${t.id}">
      <div class="timer-info">
        <div class="timer-key">${escapeHtml(t.issue_key)}</div>
        <div class="timer-summary">${escapeHtml(t.summary)}</div>
      </div>
      <div class="timer-time">${formatTime(t.elapsed_seconds)}</div>
      <div class="timer-actions">
        ${
          t.paused
            ? `<button class="timer-btn resume" title="Resume" data-action="resume" data-id="${t.id}">&#9654;</button>`
            : `<button class="timer-btn pause" title="Pause" data-action="pause" data-id="${t.id}">&#9208;</button>`
        }
        <button class="timer-btn stop" title="Stop (discard)" data-action="discard" data-id="${t.id}">&#9632;</button>
        <button class="timer-btn log" title="Stop & log to Jira" data-action="log" data-id="${t.id}">&#10003;</button>
      </div>
    </div>
  `
    )
    .join("");
}

timersList.addEventListener("click", async (e) => {
  const btn = e.target.closest("[data-action]");
  if (!btn) return;

  const action = btn.dataset.action;
  if (action === "start-ticket") return;
  const id = parseInt(btn.dataset.id, 10);

  try {
    switch (action) {
      case "pause":
        await pauseTimer(id);
        break;
      case "resume":
        await resumeTimer(id);
        break;
      case "discard":
        if (!confirm("Discard this timer without logging?")) return;
        await discardTimer(id);
        showToast("Timer stopped (not logged)", "success");
        break;
      case "log": {
        const seconds = await stopAndLog(id);
        showToast(`Logged ${formatTime(seconds)} to Jira`, "success");

        break;
      }
    }
    await refreshTimers();
  } catch (err) {
    showToast(err, "error");
  }
});

// --- Utils ---

function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

function showToast(message, type = "success") {
  const existing = document.querySelector(".toast");
  if (existing) existing.remove();

  const toast = document.createElement("div");
  toast.className = `toast ${type}`;
  toast.textContent = message;
  document.body.appendChild(toast);

  setTimeout(() => toast.remove(), 3000);
}

// --- Start ---
init();
