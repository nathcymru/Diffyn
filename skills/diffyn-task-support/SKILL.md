---
name: diffyn-task-support
description: Clarify a task, find a feasible first action or resume from supplied progress. Use for task initiation, explicit instructions, dependency review and returning after interruption.
license: MIT
---

# Support a task

Help with the requested task at the person's chosen pace. Accept rough notes and changing priorities. Do not infer physical ability, available resources or completed actions.

## Workflow

Establish the goal, known constraints and what completion means. Distinguish source instructions from user decisions and proposed actions. Ask only about missing information that changes useful work. For an ordinary task, a concrete first action may be sufficient; add a fuller plan when useful.

If Diffyn is available, inspect its schemas. Extract supplied sequences with the checklist tool, validate a supplied dependency graph with validate_action_plan, or construct build_resume_card from explicit state. A structural pass is not a safety or completeness certificate. Respect source revisions when carrying checklist progress to another call. Without tools, retain the same distinctions in plain text.

For resumption, reuse the supplied record. Show the goal, confirmed progress, blocker and recorded next action. Do not infer that an earlier suggestion was performed. Do not claim to remember or save state; provide a card the user or host can carry forward.

## Example

Given “Outline finished; figures unchecked; next check table 2”, a resume card can say “Outline complete. Next recorded action: check table 2. Figures remain unchecked.” It must not claim the report is ready to submit.

## Failure and stopping conditions

If a procedure is underspecified, state the missing prerequisite and avoid inventing safety-critical steps. A display window is not a safe stopping point. A blocked dependency stays blocked until supported information changes it. Respect pause, deferral or cancellation without nagging, streaks or unsolicited timers. Preparing a plan does not authorize external actions.
