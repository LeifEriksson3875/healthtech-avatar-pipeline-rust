mod avatar_pipeline;

use avatar_pipeline::{AvatarJob, PipelineDecision};

fn main() {
    let job = AvatarJob {
        patient_id: "patient_42".to_string(),
        filename: "portrait.jpg".to_string(),
        width: 512,
        height: 512,
        appointment_id: "apt_9001".to_string(),
    };

    match avatar_pipeline::run_preview(&job) {
        PipelineDecision::Ready { user_id, request_count } => {
            println!("avatar pipeline ready for {user_id}; {request_count} Infrai requests queued");
        }
        PipelineDecision::Rejected { reason } => println!("avatar pipeline rejected: {reason}"),
    }
}
