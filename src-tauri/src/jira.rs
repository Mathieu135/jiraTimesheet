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
    #[serde(default, rename = "originalEstimateSeconds")]
    time_estimate_seconds: u64,
    #[serde(default, rename = "remainingEstimateSeconds")]
    time_remaining_seconds: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicketDetail {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub description: String,
    pub priority: String,
    pub assignee: String,
    pub reporter: String,
    pub issue_type: String,
    pub labels: Vec<String>,
    pub created: String,
    pub updated: String,
    pub time_spent_seconds: u64,
    pub time_estimate_seconds: u64,
    pub time_remaining_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTransition {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct TransitionsResponse {
    transitions: Vec<TransitionValue>,
}

#[derive(Debug, Deserialize)]
struct TransitionValue {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct IssueDetailResponse {
    key: String,
    fields: IssueDetailFields,
}

#[derive(Debug, Deserialize)]
struct IssueDetailFields {
    summary: String,
    status: StatusField,
    description: Option<serde_json::Value>,
    priority: Option<PriorityField>,
    assignee: Option<UserField>,
    reporter: Option<UserField>,
    issuetype: Option<IssueTypeField>,
    #[serde(default)]
    labels: Vec<String>,
    created: Option<String>,
    updated: Option<String>,
    #[serde(default)]
    timetracking: Option<TimeTracking>,
}

#[derive(Debug, Deserialize)]
struct PriorityField {
    name: String,
}

#[derive(Debug, Deserialize)]
struct UserField {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct IssueTypeField {
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

    pub async fn get_transitions(&self, issue_key: &str) -> Result<Vec<JiraTransition>, String> {
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            self.base_url, issue_key
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

        let result: TransitionsResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let transitions = result
            .transitions
            .into_iter()
            .map(|t| JiraTransition {
                id: t.id,
                name: t.name,
            })
            .collect();

        Ok(transitions)
    }

    pub async fn transition_issue(&self, issue_key: &str, transition_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            self.base_url, issue_key
        );

        let body = serde_json::json!({
            "transition": { "id": transition_id }
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
            return Err(format!("Transition error {}: {}", status, body));
        }

        Ok(())
    }

    pub async fn get_issue_detail(&self, issue_key: &str) -> Result<JiraTicketDetail, String> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary,status,description,priority,assignee,reporter,issuetype,labels,created,updated,timetracking",
            self.base_url, issue_key
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

        let issue: IssueDetailResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let description = issue
            .fields
            .description
            .map(|d| extract_adf_text(&d))
            .unwrap_or_default();

        let time_tracking = issue.fields.timetracking.as_ref();

        Ok(JiraTicketDetail {
            key: issue.key,
            summary: issue.fields.summary,
            status: issue.fields.status.name,
            description,
            priority: issue.fields.priority.map(|p| p.name).unwrap_or_default(),
            assignee: issue.fields.assignee.map(|a| a.display_name).unwrap_or_default(),
            reporter: issue.fields.reporter.map(|r| r.display_name).unwrap_or_default(),
            issue_type: issue.fields.issuetype.map(|t| t.name).unwrap_or_default(),
            labels: issue.fields.labels,
            created: issue.fields.created.unwrap_or_default(),
            updated: issue.fields.updated.unwrap_or_default(),
            time_spent_seconds: time_tracking.map(|t| t.time_spent_seconds).unwrap_or(0),
            time_estimate_seconds: time_tracking.map(|t| t.time_estimate_seconds).unwrap_or(0),
            time_remaining_seconds: time_tracking.map(|t| t.time_remaining_seconds).unwrap_or(0),
        })
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

/// Extract plain text from Jira's Atlassian Document Format (ADF) JSON.
fn extract_adf_text(value: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    collect_adf_text(value, &mut parts);
    parts.join("")
}

fn collect_adf_text(value: &serde_json::Value, parts: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            let node_type = map.get("type").and_then(|v| v.as_str()).unwrap_or("");

            // Add newlines between block-level nodes
            if matches!(node_type, "paragraph" | "heading" | "bulletList" | "orderedList" | "listItem" | "codeBlock" | "blockquote") {
                if !parts.is_empty() {
                    let last = parts.last().map(|s| s.as_str()).unwrap_or("");
                    if !last.is_empty() && !last.ends_with('\n') {
                        parts.push("\n".to_string());
                    }
                }
            }

            if node_type == "listItem" {
                parts.push("- ".to_string());
            }

            // Text node
            if node_type == "text" {
                if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                    parts.push(text.to_string());
                }
            }

            // Recurse into content
            if let Some(content) = map.get("content") {
                collect_adf_text(content, parts);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_adf_text(item, parts);
            }
        }
        _ => {}
    }
}
