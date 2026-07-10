---
title: "ADR 0011: Service Direction Charter"
status: accepted
audience: all
updated: 2026-07-11
source_of_truth: docs/wiki/roadmaps/lut-video-service-master-plan.md
---

# ADR 0011: Service Direction Charter

## Context

The Phase 29-36 master plan introduces a service-capable pre-v1 track for foundation hardening, a float color chain, LUT workflows, evidence-gated RAW, optional local AI assistance, and LUT-first video support. The term "service-capable" must not change the product into SaaS, a hosted service, or a network-dependent application.

Task 29.0 is the documentation-only gate that decides whether to activate this track. The maintainer's explicit instruction to implement Task 29.0 is the acceptance required by that gate. Existing distribution and release-evidence work has independent gates that this track must preserve.

## Decision

SilicaRAW remains a local-first macOS desktop application. It is not SaaS, a hosted service, or a network-dependent product. The Phase 29-36 implementation route is active; distribution remains blocked.

Phases 29-32 are the required baseline for this track. SilicaRAW may make RAW product claims only after Phase 33 is complete and may make video product claims only after Phase 35 is complete. Phase 34 is optional; any Phase 34 work must be deterministic, local-only, and free of network inference.

The product identity remains `SilicaRAW`, and the bundle identifier remains `dev.silicaraw.desktop`. This track does not create a fork, rename, or rebrand.

Rust Core continues to own catalog state, the edit graph, permissions, storage commands, render requests, and export orchestration. Existing crate boundaries remain authoritative. "Service-capable" introduces no service runtime, background server, hosted control plane, or network listener.

Task 29.9 must replace the disabled CSP configuration with a strict Content Security Policy and complete the associated capability audit.

Q6.3 and Q6.4 remain open. Task 27.2 remains blocked, and Phase 28 has not started. Q6.3, Q6.4, Task 27.2, and every applicable Phase 28 gate remain independent release conditions. Phase 29-36 work cannot satisfy, weaken, duplicate, or bypass those conditions. Unsigned DMGs remain developer-preview-only and do not satisfy public beta, v1.0, or signed-release gates. This ADR makes no public distribution readiness claim.

Tasks 29.1-29.12 are unblocked only as their dependencies in the Phase 29 DAG are satisfied. Phase 30 cannot start until every Phase 29 task is complete.

This decision and Task 29.0 change documentation only. They introduce no code, schema, dependency, bundle identity, runtime behavior, or release-evidence change.

## Consequences

- The LUT and Video Service Master Plan becomes the active product-development route for Phase 29 and later work.
- Task 29.1 is the default next product-development task; other Phase 29 branches may proceed only when the DAG permits them.
- RAW, video, and optional local AI scope remain separated by explicit phase and evidence gates.
- Distribution and release-evidence work continues independently with Q6.3 next and Q6.4 after it; Task 27.2 remains blocked, and Phase 28 has not started because its existing entry conditions are unmet.
- No application behavior changes as a result of this ADR.

## Alternatives Considered

- **Keep the track in draft indefinitely:** rejected because the maintainer has explicitly accepted the local service-capable direction and its gates.
- **Move SilicaRAW to SaaS, hosting, or network-required operation:** rejected because it conflicts with the local-first product boundary and offline workflow.
- **Fork, rename, or rebrand for this track:** rejected because the current product and bundle identity remain authoritative.
- **Treat Phase 34 as required or allow network inference:** rejected because AI assistance is optional, deterministic, and local-only in this track.
- **Use Phase 29-36 work to bypass inherited release gates:** rejected because product capability work and release evidence are independent.

## Links

- [LUT and Video Service Master Plan](../roadmaps/lut-video-service-master-plan.md)
- [Task 29.0: Service Direction Design Gate](../tasks/29.0-service-direction-design-gate.md)
- [Current LLM Route](../llm/current-route.md)
- [Local Alpha Quality Closure Plan](../roadmaps/local-alpha-quality-closure-plan.md)
- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
