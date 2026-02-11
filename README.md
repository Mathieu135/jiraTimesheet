# Jira Timesheet

App desktop pour tracker le temps sur les tickets Jira et logger les worklogs automatiquement.

## Stack

- **Backend** : Rust + Tauri v2
- **Frontend** : Vanilla JS + Vite
- **UI** : Fenetre frameless, dark theme, system tray

## Fonctionnalites

- Listing des projets Jira avec filtre de recherche
- Tickets par projet (lazy load) avec temps deja logge
- Timers : start / pause / resume / discard / log to Jira
- Confirmation avant discard d'un timer
- Raccourci global `Ctrl+Shift+T` pour afficher/masquer la fenetre
- System tray avec menu Show/Quit

## Prerequis

- Docker + Docker Compose
- WSLg (pour l'affichage graphique sous WSL2)

## Installation

```bash
cp .env.example .env
# Editer .env avec vos identifiants Jira
```

### Variables d'environnement

| Variable | Description |
|----------|-------------|
| `JIRA_URL` | URL de votre instance Jira (ex: `https://xxx.atlassian.net`) |
| `JIRA_EMAIL` | Email du compte Jira |
| `JIRA_TOKEN` | [API token Jira](https://id.atlassian.com/manage-profile/security/api-tokens) |

## Lancement

```bash
docker compose build
docker compose up
```

Premier build long (compilation Rust), les suivants sont caches grace aux volumes Docker.

## Structure

```
.
├── index.html                  # HTML principal
├── src/
│   ├── main.js                 # Logique frontend (navigation, timers)
│   ├── jira.js                 # Wrapper invoke() commandes Tauri
│   └── style.css               # Theme dark, CSS variables
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # Commandes Tauri (orchestrateur)
│   │   ├── jira.rs             # Client HTTP Jira
│   │   ├── timer.rs            # Gestion timers en memoire
│   │   └── config.rs           # Config via env vars
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docker-compose.yml
├── Dockerfile.dev
└── .env.example
```

## API Jira utilisees

- `GET /rest/api/3/project/search` — Liste des projets
- `GET /rest/api/3/search/jql` — Recherche tickets par projet
- `POST /rest/api/3/issue/{key}/worklog` — Logger du temps
