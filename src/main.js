import {
  listProjects,
  searchTickets,
  getIssueDetail,
  getTransitions,
  transitionIssue,
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
const ticketDetailSection = document.getElementById("ticket-detail-section");
const detailTitle = document.getElementById("detail-title");
const detailContent = document.getElementById("ticket-detail-content");
const detailBackBtn = document.getElementById("detail-back-btn");
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
      <div class="ticket-info" data-action="show-detail" data-key="${escapeHtml(t.key)}">
        <span class="ticket-key">${escapeHtml(t.key)}</span>
        <span class="ticket-summary">${escapeHtml(t.summary)}</span>
      </div>
      ${t.time_spent_seconds > 0 ? `<span class="ticket-logged-time" title="Time logged">${formatTime(t.time_spent_seconds)}</span>` : ""}
      <button class="ticket-status" data-action="change-status" data-key="${escapeHtml(t.key)}" title="Change status">${escapeHtml(t.status)}</button>
      <button class="btn btn-primary btn-sm" data-action="start-ticket" data-key="${escapeHtml(t.key)}" data-summary="${escapeHtml(t.summary)}">Start</button>
    </div>
  `
    )
    .join("");
}

ticketsList.addEventListener("click", async (e) => {
  const startBtn = e.target.closest("[data-action='start-ticket']");
  if (startBtn) {
    try {
      await startTimer(startBtn.dataset.key, startBtn.dataset.summary);
      showToast(`Timer started for ${startBtn.dataset.key}`, "success");
      await refreshTimers();
    } catch (err) {
      showToast(err, "error");
    }
    return;
  }

  const statusBtn = e.target.closest("[data-action='change-status']");
  if (statusBtn) {
    await showTransitionMenu(statusBtn);
    return;
  }

  const infoEl = e.target.closest("[data-action='show-detail']");
  if (infoEl) {
    await showTicketDetail(infoEl.dataset.key);
  }
});

async function showTransitionMenu(anchor) {
  closeTransitionMenu();
  const issueKey = anchor.dataset.key;

  try {
    const transitions = await getTransitions(issueKey);
    if (transitions.length === 0) {
      showToast("No transitions available", "error");
      return;
    }

    const menu = document.createElement("div");
    menu.className = "transition-menu";
    menu.innerHTML = transitions
      .map(
        (t) =>
          `<button class="transition-option" data-tid="${escapeHtml(t.id)}">${escapeHtml(t.name)}</button>`
      )
      .join("");

    const rect = anchor.getBoundingClientRect();
    menu.style.top = `${rect.bottom + 4}px`;
    menu.style.left = `${rect.left}px`;

    menu.addEventListener("click", async (e) => {
      const opt = e.target.closest(".transition-option");
      if (!opt) return;
      closeTransitionMenu();
      try {
        await transitionIssue(issueKey, opt.dataset.tid);
        showToast(`Status updated`, "success");
        const ticket = cachedTickets.find((t) => t.key === issueKey);
        if (ticket) {
          ticket.status = opt.textContent;
        }
        renderTickets(
          ticketsFilter.value
            ? cachedTickets.filter((t) => {
                const q = ticketsFilter.value.toLowerCase();
                return (
                  t.key.toLowerCase().includes(q) ||
                  t.summary.toLowerCase().includes(q) ||
                  t.status.toLowerCase().includes(q)
                );
              })
            : cachedTickets
        );
      } catch (err) {
        showToast(err, "error");
      }
    });

    document.body.appendChild(menu);
    setTimeout(() => {
      document.addEventListener("click", closeTransitionMenu, { once: true });
    }, 0);
  } catch (err) {
    showToast(err, "error");
  }
}

function closeTransitionMenu() {
  const existing = document.querySelector(".transition-menu");
  if (existing) existing.remove();
}

// --- Ticket detail ---

async function showTicketDetail(issueKey) {
  ticketsSection.classList.add("hidden");
  ticketDetailSection.classList.remove("hidden");
  detailTitle.textContent = issueKey;
  detailContent.innerHTML = '<div class="loading">Loading details...</div>';

  try {
    const detail = await getIssueDetail(issueKey);
    renderTicketDetail(detail);
  } catch (err) {
    detailContent.innerHTML = `<div class="empty-state">Error: ${escapeHtml(String(err))}</div>`;
  }
}

function formatDate(isoString) {
  if (!isoString) return "—";
  const d = new Date(isoString);
  return d.toLocaleDateString("fr-FR", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function renderTicketDetail(detail) {
  const hasTimeTracking =
    detail.time_spent_seconds > 0 ||
    detail.time_estimate_seconds > 0 ||
    detail.time_remaining_seconds > 0;

  detailContent.innerHTML = `
    <div class="detail-meta">
      <div class="detail-meta-row">
        <span class="detail-label">Type</span>
        <span class="detail-value">${escapeHtml(detail.issue_type || "—")}</span>
      </div>
      <div class="detail-meta-row">
        <span class="detail-label">Status</span>
        <span class="detail-value detail-status">${escapeHtml(detail.status)}</span>
      </div>
      <div class="detail-meta-row">
        <span class="detail-label">Priority</span>
        <span class="detail-value">${escapeHtml(detail.priority || "—")}</span>
      </div>
      <div class="detail-meta-row">
        <span class="detail-label">Assignee</span>
        <span class="detail-value">${escapeHtml(detail.assignee || "—")}</span>
      </div>
      <div class="detail-meta-row">
        <span class="detail-label">Reporter</span>
        <span class="detail-value">${escapeHtml(detail.reporter || "—")}</span>
      </div>
      ${detail.labels.length > 0 ? `
      <div class="detail-meta-row">
        <span class="detail-label">Labels</span>
        <span class="detail-value">${detail.labels.map((l) => `<span class="detail-label-tag">${escapeHtml(l)}</span>`).join(" ")}</span>
      </div>` : ""}
      <div class="detail-meta-row">
        <span class="detail-label">Created</span>
        <span class="detail-value">${formatDate(detail.created)}</span>
      </div>
      <div class="detail-meta-row">
        <span class="detail-label">Updated</span>
        <span class="detail-value">${formatDate(detail.updated)}</span>
      </div>
      ${hasTimeTracking ? `
      <div class="detail-meta-row">
        <span class="detail-label">Time</span>
        <span class="detail-value detail-time">
          ${detail.time_spent_seconds > 0 ? `<span title="Logged">${formatTime(detail.time_spent_seconds)} logged</span>` : ""}
          ${detail.time_estimate_seconds > 0 ? `<span title="Estimate">${formatTime(detail.time_estimate_seconds)} estimated</span>` : ""}
          ${detail.time_remaining_seconds > 0 ? `<span title="Remaining">${formatTime(detail.time_remaining_seconds)} remaining</span>` : ""}
        </span>
      </div>` : ""}
    </div>
    <div class="detail-summary">${escapeHtml(detail.summary)}</div>
    ${detail.description ? `<div class="detail-description">${escapeHtml(detail.description)}</div>` : ""}
    <div class="detail-actions">
      <button class="btn btn-primary btn-sm" id="detail-start-btn" data-key="${escapeHtml(detail.key)}" data-summary="${escapeHtml(detail.summary)}">Start Timer</button>
    </div>
  `;

  const startBtn = document.getElementById("detail-start-btn");
  startBtn.addEventListener("click", async () => {
    try {
      await startTimer(startBtn.dataset.key, startBtn.dataset.summary);
      showToast(`Timer started for ${startBtn.dataset.key}`, "success");
      await refreshTimers();
    } catch (err) {
      showToast(err, "error");
    }
  });
}

detailBackBtn.addEventListener("click", () => {
  ticketDetailSection.classList.add("hidden");
  ticketsSection.classList.remove("hidden");
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
