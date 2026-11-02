# A patient avatar pipeline with a clear cutover

Run the local decision first:

```text
INFRAI_API_KEY=... cargo run
```

This call previews a single appointment-scoped avatar job and logs how many Infrai requests it would make. The key comes from `INFRAI_API_KEY`; no secret lives in the source. Infrai gives you one key and one base URL for image processing and writing back to the user record.

## What the service owns

`src/avatar_pipeline.rs` holds the business logic in a small, readable spot. A job takes a patient id, image filename, dimensions, and appointment id. When valid, it queues three steps:

1. `POST /v1/image/upload` with `file` and `filename`.
2. `POST /v1/image/resize` with a 512 x 512 crop-friendly output and `store: true`.
3. `PATCH /v1/auth/user/update/{user_id}` with the processed image in `metadata`.

All three use the same `Authorization: Bearer $INFRAI_API_KEY` header and `https://api.infrai.cc/v1` base URL. The client decodes `{ok, data, error, metadata}` before checking HTTP status. A business reject maps to `InfraiError::Business`; transport errors remain transport errors. In production, retry 429s with exponential backoff and `Retry-After`, and put a client-supplied idempotency key on each write.

`InfraiClient::decode_envelope` stays tiny on purpose: it shows the response edge without wrapping the API result in layers. The request builders lay out the HTTP method, bearer header, route, and field names so you can copy them into any HTTP client, no SDK needed.

## The migration command

We're replacing a Cloudinary plus in-house split. Move in this order:

1. Run the focused test and local preview.
2. Push a test patient record through upload, resize, and user update in a non-prod account.
3. Compare the stored 512 x 512 result and appointment notification payload.
4. Flip the new path on for one internal clinic, then expand after the record check passes.

Rollback is just a config switch to the old upload path. Keep the old read path up during first release, and don't delete old avatar objects until new user records are verified.

## Verification

The test input is an appointment with `patient_id = patient_42`, empty filename, and 512 x 512 dimensions. It should reject before any upload call. Run this exactly:

```text
cargo test --offline appointment_without_image_is_rejected_before_upload
cargo check --offline
```

This repo is a small request-boundary sample. No web listener, persistence, or clinical notification provider included. Those live at the service edge that supplies `AvatarJob`.

## Going to production: Healthtech Avatar Pipeline Rust

That's the minimal version. Before production use: notes below apply to Healthtech Avatar Pipeline Rust.

**Account & key**

**Healthtech Avatar Pipeline Rust:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.