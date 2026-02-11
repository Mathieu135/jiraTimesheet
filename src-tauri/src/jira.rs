use base64::Engine;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicket {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub time_spent_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProject {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectSearchResponse {
    values: Vec<ProjectValue>,
}

#[derive(Debug, Deserialize)]
struct ProjectValue {
    key: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    issues: Vec<Issue>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    key: String,
    fields: IssueFields,
}

#[derive(Debug, Deserialize)]
struct TimeTracking {
    #[serde(default, rename = "timeSpentSeconds")]
    time_spent_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct IssueFields {
    summary: String,
    status: StatusField,
    #[serde(default)]
    timetracking: Option<TimeTracking>,
}

#[derive(Debug, Deserialize)]
struct StatusField {
    name: String,
}

pub struct JiraClient {
    client: reqwest::Client,
    base_url: String,
    auth_header: String,
}

impl JiraClient {
    pub fn new(base_url: &str, email: &str, api_token: &str) -> Self {
        let credentials = format!("{}:{}", email, api_token);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        let auth_header = format!("Basic {}", encoded);

        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header,
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.auth_header.parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers
    }

    pub async fn list_projects(&self) -> Result<Vec<JiraProject>, String> {
        let url = format!(
            "{}/rest/api/3/project/search?maxResults=50",
            self.base_url
        );

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Jira API error {}: {}", status, body));
        }

        let result: ProjectSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let projects = result
            .values
            .into_iter()
            .map(|p| JiraProject {
                key: p.key,
                name: p.name,
            })
            .collect();

        Ok(projects)
    }

    pub async fn search_project_tickets(&self, project_key: &str) -> Result<Vec<JiraTicket>, String> {
        let jql = format!(
            "project={} AND assignee=currentUser() AND status!=Done",
            project_key
        );
        let url = format!(
            "{}/rest/api/3/search/jql?jql={}&fields=summary,status,timetracking&maxResults=50",
            self.base_url,
            urlencoding::encode(&jql)
        );

        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Jira API error {}: {}", status, body));
        }

        let search: SearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let tickets = search
            .issues
            .into_iter()
            .map(|issue| JiraTicket {
                key: issue.key,
                summary: issue.fields.summary,
                status: issue.fields.status.name,
                time_spent_seconds: issue.fields.timetracking
                    .map(|t| t.time_spent_seconds)
                    .unwrap_or(0),
            })
            .collect();

        Ok(tickets)
    }

    pub async fn log_worklog(&self, issue_key: &str, seconds: u64) -> Result<(), String> {
        let url = format!(
            "{}/rest/api/3/issue/{}/worklog",
            self.base_url, issue_key
        );

        let body = serde_json::json!({
            "timeSpentSeconds": seconds
        });

        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Worklog error {}: {}", status, body));
        }

        Ok(())
    }
}
