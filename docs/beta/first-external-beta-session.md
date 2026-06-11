# First External Beta Session Runbook

**Version:** v2.14.0
**Purpose:** How to conduct the first real external beta session through Orqestra's evidence chain.

---

## Overview

This runbook defines how to run the first real external beta session, collect consented evidence, and review it honestly. The session must involve at least one person who is not the maintainer/developer running the internal release process.

**Key rule:** This session is testing the beta path, not the participant. A failed, blocked, or incomplete session is still useful evidence if it is consented, redacted, structured, and reviewable.

---

## 1. Who Qualifies as an External Beta Participant

An external beta participant is someone who:

- Is not the project maintainer or core developer
- Has not authored or reviewed the Orqestra source code
- Has not run internal smoke checks, CI, or release processes
- Will use the documented self-serve path without insider knowledge

Do not count:

- Internal smoke checks
- Maintainer rehearsals
- CI fixtures or synthetic bundles
- Local-only developer walkthroughs
- Scripted demo sessions

---

## 2. How to Invite the Participant

1. Identify a willing participant who meets the criteria above
2. Share the participant instructions document (`external-beta-participant-instructions.md`)
3. Provide the download link or package for Orqestra
4. Set a reasonable time window (suggest 30–60 minutes)
5. Be available for questions but do not guide step-by-step

---

## 3. What to Provide

- Orqestra installer or binary for the participant's platform
- Link to the participant instructions document
- Link to the beta quickstart guide (`docs/beta-quickstart.md`)
- Contact method for questions (GitHub issue, email, or chat)

Do not provide:

- Internal test accounts or credentials
- Pre-configured settings files
- Access to internal documentation not in the public repo
- Your own API keys or service tokens

---

## 4. What the Participant Should Attempt

The participant should follow the participant instructions document, which asks them to:

1. Install or open Orqestra
2. Open a test repository or a repository they are comfortable using for beta testing
3. Complete the onboarding/readiness flow as far as they can
4. Export Beta Evidence only if they consent
5. Review the exported folder locally before sharing
6. Share the folder only if they are comfortable

---

## 5. What Evidence to Expect

If the participant completes the export, the bundle should contain:

- `beta-evidence-manifest.json` — consent and collection metadata
- `beta-session-outcome.json` — session result and steps
- `beta-failure-taxonomy.json` — failure codes if applicable
- `beta-evidence-feedback.json` — structured feedback (optional)
- `diagnostics-summary.json` — system diagnostics

If the participant cannot complete the export, that is still useful evidence. Record what happened and classify the session outcome honestly.

---

## 6. How the Participant Shares the Bundle

The participant shares the bundle manually through one of:

- Attached to a GitHub issue
- Sent via the beta feedback channel
- Shared through agreed-upon beta communication channel

No automatic upload exists. No telemetry collects bundles.

---

## 7. How to Avoid Collecting Private Data

- Tell the participant to use a test or non-sensitive repository
- Tell the participant to review the exported folder before sharing
- Tell the participant to remove anything they consider private
- Do not ask for or accept secrets, API keys, or credentials
- Do not ask for or accept screenshots containing private data

---

## 8. How to Stop the Session

If at any point:

- The participant is uncomfortable
- Privacy risk appears
- The participant wants to stop

Stop immediately. A partial or abandoned session is still valid evidence if the participant consents to sharing what occurred.

---

## 9. After the Session

1. Receive the bundle (or record that none was received)
2. Follow the review process in `external-beta-evidence-review.md`
3. Verify consent, check redaction, classify outcome
4. Update `external-beta-review.json` honestly
5. Write the session summary (aggregate only, no participant identity)
6. If accepted, update the review artifact with aggregate counts
7. If not accepted, record the honest state

---

## 10. Honest None-State

If no real external beta session has occurred by release time, the review artifact must remain:

```json
{
  "status": "none",
  "external_beta_user_data": false,
  "reviewed_bundle_count": 0,
  "accepted_bundle_count": 0
}
```

Do not fabricate, simulate, or approximate a session. The honest none-state is more valuable than a fake present-state.

---

## v2.14.0 Status

No real external beta session evidence has been reviewed yet. This release prepares the real-session runbook and participant instructions while preserving the honest none-state evidence boundary.
