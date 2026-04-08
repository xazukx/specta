---
id: TASK-001
title: Make zod type infers optional
status: Done
assignee: []
created_date: '2026-04-04 20:01'
labels: []
dependencies: []
---

Currently, zod outputs, always have statements to export inferred types like this: `z.infer` (`specta-zod\src\primitives.rs`).
Make that optional by putting a boolean setting in the `Zod` struct (`specta-zod\src\zod.rs`).
It should be default true.
