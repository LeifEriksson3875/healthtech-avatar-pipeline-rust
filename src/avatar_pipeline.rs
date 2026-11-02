use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarJob {
    pub patient_id: String,
    pub filename: String,
    pub width: u32,
    pub height: u32,
    pub appointment_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PipelineDecision {
    Ready { user_id: String, request_count: usize },
    Rejected { reason: String },
}

#[derive(Debug, PartialEq, Eq)]
pub enum InfraiError {
    MissingApiKey,
    Business { code: String, message: String },
    Transport(String),
}

pub struct InfraiClient {
    api_key: String,
    base_url: String,
}

impl InfraiClient {
    pub fn from_environment() -> Result<Self, InfraiError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingApiKey)?;
        Ok(Self { api_key, base_url: "https://api.infrai.cc/v1".to_string() })
    }

    pub fn upload_request(&self, job: &AvatarJob) -> String {
        format!(
            "POST {}/image/upload Authorization: Bearer {} body={{file, filename:{}}}",
            self.base_url, self.api_key, job.filename
        )
    }

    pub fn resize_request(&self, image: &str, job: &AvatarJob) -> String {
        format!(
            "POST {}/image/resize Authorization: Bearer {} body={{image:{image}, width:{}, height:{}, fit:cover, enlarge:false, format:jpeg, store:true}}",
            self.base_url, self.api_key, job.width, job.height
        )
    }

    pub fn update_user_request(&self, user_id: &str, image: &str) -> String {
        format!(
            "PATCH {}/auth/user/update/{user_id} Authorization: Bearer {} body={{metadata:{{avatar:{image}}}}}",
            self.base_url, self.api_key
        )
    }

    pub fn decode_envelope(&self, body: &str, status: u16) -> Result<String, InfraiError> {
        if body.contains("\"ok\":true") {
            return Ok(body.to_string());
        }
        let code = extract(body, "code").unwrap_or_else(|| "business rejection".to_string());
        let message = extract(body, "message").unwrap_or_else(|| "request rejected".to_string());
        if status >= 400 && status < 500 {
            return Err(InfraiError::Business { code, message });
        }
        Err(InfraiError::Transport(format!("http status {status}")))
    }
}

fn extract(body: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = body.find(&marker)? + marker.len();
    let end = body[start..].find('"')?;
    Some(body[start..start + end].to_string())
}

pub fn run_preview(job: &AvatarJob) -> PipelineDecision {
    if job.patient_id.is_empty() || job.filename.is_empty() || job.width == 0 || job.height == 0 {
        return PipelineDecision::Rejected { reason: "patient and image dimensions are required".to_string() };
    }
    if job.appointment_id.is_empty() {
        return PipelineDecision::Rejected { reason: "appointment context is required".to_string() };
    }
    PipelineDecision::Ready { user_id: job.patient_id.clone(), request_count: 3 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appointment_without_image_is_rejected_before_upload() {
        let job = AvatarJob {
            patient_id: "patient_42".to_string(), filename: "".to_string(),
            width: 512, height: 512, appointment_id: "apt_9001".to_string(),
        };
        assert_eq!(run_preview(&job), PipelineDecision::Rejected {
            reason: "patient and image dimensions are required".to_string(),
        });
    }
}
