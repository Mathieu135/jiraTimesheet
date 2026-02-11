# Jira Timesheet

App desktop pour tracker le temps sur les tickets Jira et logger les worklogs automatiquement.

## Stack

- **Backend** : Rust + Tauri v2
- **Frontend** : Vanilla JS + Vite
- **UI** : Dark theme, decorations natives Windows, system tray

## Fonctionnalites

- Listing des projets Jira avec filtre de recherche instantane
- Tickets par projet (lazy load) avec temps deja logge affiche
- Changement de statut des tickets (transitions Jira)
- Timers : start / pause / resume / discard / log to Jira
- Confirmation avant discard d'un timer
- Sections redimensionnables (projets, tickets, timers)
- Raccourci global `Ctrl+Shift+T` pour afficher/masquer la fenetre
- System tray avec menu Show/Quit
- Config via `.env` (persistee entre les lancements)

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
│   ├── main.js                 # Logique frontend (navigation, filtres, timers)
│   ├── jira.js                 # Wrapper invoke() commandes Tauri
│   └── style.css               # Theme dark, CSS variables
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              # Commandes Tauri (orchestrateur)
│   │   ├── jira.rs             # Client HTTP Jira (projets, tickets, transitions, worklogs)
│   │   ├── timer.rs            # Gestion timers en memoire
│   │   └── config.rs           # Config via env vars
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docker-compose.yml
├── Dockerfile.dev
└── .env.example
```

## API Jira utilisees

| Endpoint | Methode | Usage |
|----------|---------|-------|
| `/rest/api/3/project/search` | GET | Liste des projets |
| `/rest/api/3/search/jql` | GET | Recherche tickets par projet (avec timetracking) |
| `/rest/api/3/issue/{key}/transitions` | GET | Transitions disponibles pour un ticket |
| `/rest/api/3/issue/{key}/transitions` | POST | Appliquer une transition (changer le statut) |
| `/rest/api/3/issue/{key}/worklog` | POST | Logger du temps |
