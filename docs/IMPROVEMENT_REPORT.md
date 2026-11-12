# Neuron-Wire — Comprehensive Repository Improvement Report

**Generated:** 2026-08-26
**Current HEAD:** `8e77eb6` (docs: update GitHub URLs from cianmag to zylvon)
**Branch:** master → zylvon/neuron-wire
**Tracked files:** 487
**Lines of Rust:** ~21,000 (src/) + tests

---

## 1. Repository Overview

Neuron-Wire (NWP) is an MIT-licensed Rust framework for coordinator-free, peer-to-peer decentralized learning. It provides:

- **Latency-weighted Kademlia DHT** for peer discovery (256 buckets, K=20)
- **Custom reliable UDP transport** with 3 reliability tiers (BestEffort/Data/Consensus), ACK bitfield, gradient aging
- **Hebbian/STDP learning** with neurogenesis, apoptosis, sparse gossip
- **Ed25519 identity + XChaCha20-Poly1305 AEAD** encryption (optional, off by default)
- **Trust-based rate limiting** for Sybil resistance
- **Single-threaded non-blocking engine** (~400 KHz–1 MHz ticks, no async runtime)
- **Deterministic simulator** with fixed seeds, known-good validation
- **E1–E9 experiment matrix** covering churn, partition, malicious routing, gradient decay, trust ablation, neurogenesis

**Validation status:** 342 tests pass on Linux CI, 4 fuzz targets, 19 benchmarks, network emulation, real UDP multi-process clusters.

---

## 2. What's Working Well

### Strengths
1. **Evidence-driven development** — Every claim is pinned to a commit and CI run. 7 pipeline-caught bugs documented as negative results. Honest about limitations.
2. **Reproducibility infrastructure** — One-command reproduction, known-good fixtures, deterministic seeds, CI-gated validation.
3. **Comprehensive documentation** — 39 root-level docs, formal model (1760 lines), algorithm spec, wire protocol spec, architecture diagrams, developer guide.
4. **Grant readiness** — Complete grant package (PDF + technical summary + 8 submission drafts), tracker with eligibility fields, evidence release with 5 assets.
5. **CI/CD maturity** — 7 GitHub Actions workflows, pre-commit hooks, cargo fmt/clippy gates, cross-platform (Linux/macOS/Windows/Docker).
6. **Security posture** — Ed25519 signatures, XChaCha20-Poly1305, trust system, audit log, gitleaks, cargo-audit, cargo-deny, CodeQL.
7. **Research rigor** — 8 research questions with falsifiable hypotheses, 10-paper series, formal proofs (convergence, Sybil bounds, Big-O/Ω/Θ).

---

## 3. Areas for Improvement

### 3.1 Code Quality & Maintainability

#### A. Engine Loop Monolith (`src/engine_loop.rs` — 1,858 lines)
- **Issue:** Single file concentrates all runtime logic: ingress, DHT, forward pass, Hebbian, neurogenesis, apoptosis, ML, observability.
- **Impact:** Hard to test, review, and modify individual phases independently.
- **Recommendation:** Split into phase modules (`engine/ingress.rs`, `engine/forward.rs`, `engine/hebbian.rs`, etc.) behind a trait or enum-dispatch. Each phase becomes independently testable.
- **Effort:** Medium (2-3 weeks). **Priority:** Funded-phase.

#### B. ML Subsystem Scope Creep (14+ modules)
- **Issue:** `src/` contains 14 ML/training modules (adaptive_lr, attention, continual_learning, curiosity, distillation, dynamic_activation, forgetting_bench, gradient_compression, meta_learning, ml, neurogenesis, sparse_tensor, uncertainty, distributed_replay) that expand beyond the core research claim (P2P gradient exchange + Hebbian learning).
- **Impact:** Dilutes focus, increases maintenance burden, makes the project harder to evaluate.
- **Recommendation:** Separate into an optional `neuron-wire-ml` crate or feature-gate behind `ml` feature flag. Core crate should be transport + DHT + Hebbian only.
- **Effort:** Medium. **Priority:** Funded-phase (post-grant).

#### C. `src/io.rs` — Legacy TCP Framing (169 lines)
- **Issue:** TCP framing helpers exist but are not the production transport. They add confusion about protocol identity.
- **Current state:** Already documented as "legacy/tooling only" but still compiled and exported.
- **Recommendation:** Move to `examples/` or `tests/` helpers, or gate behind a `legacy-tcp` feature flag. Remove from default `lib.rs` re-exports.
- **Effort:** Low. **Priority:** High (clarity).

#### D. `src/health.rs` — HTTP Observability (TCP-based)
- **Issue:** HTTP health/metrics endpoint uses TCP, which conflicts with the "UDP-only production transport" narrative.
- **Recommendation:** Either convert to UDP-based metrics or clearly document as "observability subsystem, not peer transport." Already partially done in ARCHITECTURE.md.
- **Effort:** Low. **Priority:** Medium.

#### E. `src/observability/` — Dashboard/Metrics (TCP-based)
- **Issue:** Same as health.rs — HTTP/SSE observability uses TCP.
- **Recommendation:** Same documentation approach. Consider QUIC or HTTP/3 for future.
- **Effort:** Low-Medium. **Priority:** Low (funded-phase).

#### F. Dependency Versions
- **Issue:** Some dependencies may be outdated. `toml = "0.7"` is old (latest 0.8). `rand = "0.8"` (latest 0.9). `proptest = "1"` (fine). `criterion = "0.5"` (fine).
- **Recommendation:** Audit with `cargo update` and `cargo audit`. Update toml to 0.8, rand to 0.9 if compatible.
- **Effort:** Low-Medium. **Priority:** Medium.

#### G. `#![deny(missing_docs)]` vs `#![warn(missing_docs)]`
- **Issue:** `src/adaptive_lr.rs` uses `#![deny(missing_docs)]` while `src/lib.rs` uses `#![warn(missing_docs)]`. Inconsistent.
- **Recommendation:** Standardize on `warn` for all modules, or `deny` for all. `deny` is stricter but may block compilation of new code.
- **Effort:** Trivial. **Priority:** Low.

### 3.2 Testing Gaps

#### A. No Combined Encrypted+Learning E2E Test
- **Issue:** Security and learning paths are tested separately. Combined `secure_distributed_learning_e2e.rs` is listed as M1 milestone but not yet implemented.
- **Impact:** Grant reviewers may question whether encryption + learning work together.
- **Recommendation:** Implement `tests/secure_distributed_learning_e2e.rs` — deterministic Ed25519 identities, signed-frame acceptance, unsigned/modified-frame rejection, encrypted payload transport, replay rejection, synapse update only after authentication.
- **Effort:** Medium. **Priority:** High (grant milestone M1).

#### B. No WAN/Internet-Scale Test
- **Issue:** All tests run on localhost. No real Internet deployment validation.
- **Impact:** Core research question (RQ1) remains unanswered.
- **Recommendation:** Funded-phase: 20-node, 3-region pilot (NA/EU/APAC).
- **Effort:** High (requires funding). **Priority:** Funded-phase.

#### C. No NAT Traversal Validation
- **Issue:** STUN auto-discovery exists but no real asymmetric-NAT pair test.
- **Recommendation:** Funded-phase: ≥3 real asymmetric-NAT pairs connect.
- **Effort:** High. **Priority:** Funded-phase.

#### D. No Valid Network-Partition Experiment
- **Issue:** `emulate_network.sh` uses iptables loopback split, which doesn't fully isolate (documented as caveat).
- **Recommendation:** Use netns or Docker veth for real partition isolation. Funded-phase task.
- **Effort:** Medium. **Priority:** Funded-phase (M4).

#### E. No Real-Task Learning Benchmark
- **Issue:** Hebbian STDP is validated on synthetic signals, not real ML tasks.
- **Recommendation:** Funded-phase: MNIST/CIFAR on P2P topology.
- **Effort:** High. **Priority:** Funded-phase.

#### F. Stress Tests Are `#[ignore]`d
- **Issue:** `tests/stress.rs` has 9 stress tests, all `#[ignore]`d. They don't run in CI.
- **Recommendation:** Run at least `stress_ping_pong` in CI with a shorter duration (e.g., 5s instead of 30s). Or create a separate `stress-ci` profile.
- **Effort:** Low. **Priority:** Medium.

#### G. Fuzz Targets Not in CI
- **Issue:** 4 fuzz targets exist but aren't run in CI (cargo-fuzz requires nightly).
- **Recommendation:** Add a nightly CI job that runs fuzz targets for 60s each. Or use `cargo-fuzz` with `--max-total-time`.
- **Effort:** Low. **Priority:** Medium.

### 3.3 Documentation

#### A. Stale Commit References (Fixed in this session)
- **Issue:** Grant docs referenced `265e216` as evidence commit (correct for v0.3.1). Now updated with current HEAD.
- **Status:** ✅ Fixed.

#### B. 10 Paper Drafts vs. 2-4 Credible Near-Term
- **Issue:** `papers/` has 11 files (10 drafts + README). Reviewer flagged excessive paper count.
- **Recommendation:** Consolidate to 2-4 credible papers: (1) DHT convergence, (2) bounded routing, (3) Hebbian learning E2E, (4) negative results. Move others to `archive/`.
- **Effort:** Medium. **Priority:** Medium (grant credibility).

#### C. 214 Result Files Need Curation
- **Issue:** `results/` has 214 committed benchmark outputs. Many are historical/intermediate.
- **Recommendation:** Curate to ~20-30 representative files. Move rest to `archive/` or remove (they're in git history).
- **Effort:** Low-Medium. **Priority:** Medium (repo hygiene).

#### D. `docs/REPOSITORY_FILE_REPORT.md` — HEAD at generation is stale
- **Issue:** Report says "HEAD at generation: `ea97e04`" but current HEAD is `8e77eb6`.
- **Recommendation:** Regenerate or add a note that it's a snapshot.
- **Effort:** Trivial. **Priority:** Low.

#### E. `FOUNDATIONAL_QNA.md` — Stale commit reference
- **Issue:** References commit `5a739f9` which is very old.
- **Recommendation:** Update to current HEAD or remove stale reference.
- **Effort:** Trivial. **Priority:** Low.

#### F. `STATS.md` — Stale commit reference
- **Issue:** References commit `f3c9157`.
- **Recommendation:** Update to current HEAD.
- **Effort:** Trivial. **Priority:** Low.

### 3.4 Infrastructure & Deployment

#### A. No zylvon.com Website
- **Issue:** DigitalOcean Hatch requires a company website. zylvon.com resolves (Cloudflare DNS) but returns HTTP 000.
- **Recommendation:** Stand up a minimal static site (even a single page) on zylvon.com. This unlocks DO Hatch credits ($250+).
- **Effort:** Low. **Priority:** High (unblocks grant).

#### B. Dashboard (Vercel) — Needs Update
- **Issue:** Dashboard at `dashboard/` (5 files) is deployed to neuron-wire-dashboard.vercel.app. May need updating to reflect latest evidence.
- **Recommendation:** Update dashboard with E1.2 bounded-routing data and current evidence.
- **Effort:** Low-Medium. **Priority:** Medium.

#### C. `website/` — Placeholder Content
- **Issue:** `website/` has only 2 files (README.md + index.html). Likely a placeholder.
- **Recommendation:** Either build out the website or remove the directory.
- **Effort:** Low. **Priority:** Low.

#### D. `demo/` — WASM Browser Demo
- **Issue:** Demo exists (6 files) but may be stale. Deployed to sentient-foundation.vercel.app.
- **Recommendation:** Verify demo works with current code. Update if needed.
- **Effort:** Low-Medium. **Priority:** Low.

#### E. `deploy/` — Ansible Playbooks
- **Issue:** Ansible deployment files exist but no real deployment has happened.
- **Recommendation:** Funded-phase: use for 20-node pilot deployment.
- **Effort:** High. **Priority:** Funded-phase.

### 3.5 Security

#### A. CodeQL Alert #4 — Hard-coded Cryptographic Value (FIXED)
- **Issue:** `src/secure_channel.rs:292` had `NONCE_SIZE = 16` (hard-coded) but XChaCha20-Poly1305 requires 24.
- **Status:** ✅ Fixed — updated to 24. Need to verify CodeQL clears.

#### B. CodeQL Alerts #1-#3 — False Positives
- **Issue:** Unused variable warnings in tests (`adaptive_lr.rs:113`, `stress.rs:292`, `security_integration.rs:263`).
- **Status:** These are false positives (variables are used in assertions). No code change needed. May need CodeQL suppression comments.

#### C. No External Security Audit
- **Issue:** Cryptographic and transport layers have not been externally audited.
- **Recommendation:** Funded-phase: $5,000 external security review (budgeted in grant).

#### D. Encryption Off by Default
- **Issue:** XChaCha20-Poly1305 encryption is optional and off by default. AEAD handshake not hardened.
- **Recommendation:** Make encryption on by default for production. Harden ECDH handshake.
- **Effort:** Medium. **Priority:** High (security).

### 3.6 Grant & Funding

#### A. NLnet OIS — Paused Until Post-Summer 2026
- **Status:** Draft ready (`NLNET_OIS.md`). Re-verify nlnet.nl/apply weekly. Submit within 2 weeks of reopen.
- **Action:** Set calendar reminder for 2026-08-15 re-verify.

#### B. Rust Foundation — Age-Blocked (Applicant is 16)
- **Status:** Draft ready (`RUST_FOUNDATION.md`). Need to email grants@rustfoundation.org re: fiscal-sponsor/institutional pathway.
- **Action:** Send email with 3 questions: (a) fiscal-sponsor/institutional pathway, (b) age waiver w/ guardian consent, (c) Nepal receipt.

#### C. DigitalOcean Hatch — Blocked (No Website)
- **Status:** zylvon.com resolves but returns HTTP 000.
- **Action:** Stand up minimal zylvon.com site. Then apply.

#### D. Protocol Labs — Paused Indefinitely
- **Status:** Draft preserved as template. Do NOT submit.

#### E. Filecoin Foundation — Concept Only
- **Status:** Go/no-go pending. No real Filecoin integration yet.
- **Action:** Evaluate go/no-go criteria before any submission.

#### F. Microsoft Founders Hub / AWS Activate / Ethereum ESP
- **Status:** Active — verify. Need to check eligibility.

### 3.7 Repository Hygiene

#### A. `.gitignore` — Minimal (6 lines)
- **Issue:** `.gitignore` only has `results/` and `known-good/`. Should also ignore `target/`, `.env`, IDE files, etc.
- **Recommendation:** Expand `.gitignore` with standard Rust + IDE patterns.
- **Effort:** Trivial. **Priority:** Low.

#### B. No `CHANGELOG.md` Auto-Generation
- **Issue:** CHANGELOG.md is manually maintained (93 lines). Could use `git-cliff` or `cargo-release`.
- **Recommendation:** Adopt `git-cliff` for automated changelog generation from conventional commits.
- **Effort:** Low. **Priority:** Low.

#### C. `Cargo.lock` Committed (Good)
- **Status:** ✅ Correctly committed for a binary/library hybrid.

#### D. `results/` Directory — 214 Files
- **Issue:** 214 committed benchmark outputs. Many are historical/intermediate.
- **Recommendation:** Curate to ~20-30 representative files. Archive rest.
- **Effort:** Low-Medium. **Priority:** Medium.

#### E. `papers/` — 11 Files (10 Drafts)
- **Issue:** Excessive paper count for near-term credibility.
- **Recommendation:** Consolidate to 2-4 credible papers. Move rest to `archive/`.
- **Effort:** Medium. **Priority:** Medium.

### 3.8 Performance

#### A. No Cross-Platform Benchmark Corpus
- **Issue:** Benchmarks run only on Linux CI. No macOS/Windows/ARM data.
- **Recommendation:** Funded-phase: M2 milestone (cross-platform benchmark corpus).
- **Effort:** Medium. **Priority:** Funded-phase.

#### B. No Congestion Control
- **Issue:** UDP transport has no built-in congestion control. Runtime manages its own sending rate.
- **Recommendation:** Implement basic congestion control (e.g., AIMD) or document as limitation.
- **Effort:** Medium. **Priority:** Medium.

#### C. No Application-Layer Fragmentation
- **Issue:** Large messages (>~1400 bytes MTU) need fragmentation. Not implemented.
- **Recommendation:** Implement fragmentation/reassembly for large gradients.
- **Effort:** Medium. **Priority:** Medium.

#### D. No QUIC Migration
- **Issue:** Custom reliability layer is younger than battle-tested TCP/QUIC stacks.
- **Recommendation:** Long-term: consider QUIC (RFC 9000) for production. Current approach is fine for research.
- **Effort:** High. **Priority:** Long-term.

---

## 4. Prioritized Action Plan

### Immediate (This Week)
1. ✅ Fix CodeQL Alert #4 (NONCE_SIZE) — DONE
2. ✅ Update GitHub URLs (cianmag → zylvon) — DONE
3. ✅ Sync Obsidian vault — DONE
4. ✅ Update PROJECT_STATUS.md, EVIDENCE_REPORT.md, TRACKER.md — DONE
5. ⬜ Stand up minimal zylvon.com site (unblocks DO Hatch)
6. ⬜ Email Rust Foundation (3 questions)
7. ⬜ Re-verify nlnet.nl/apply (weekly)

### Short-Term (Next 2 Weeks)
8. ⬜ Implement `tests/secure_distributed_learning_e2e.rs` (M1 milestone)
9. ⬜ Run stress tests in CI (shorter duration)
10. ⬜ Add nightly fuzz CI job
11. ⬜ Consolidate papers (10 → 2-4)
12. ⬜ Curate results/ (214 → ~30)
13. ⬜ Update FOUNDATIONAL_QNA.md and STATS.md commit references
14. ⬜ Expand .gitignore

### Medium-Term (Next Month)
15. ⬜ Split engine_loop.rs into phase modules
16. ⬜ Feature-gate ML subsystems behind `ml` flag
17. ⬜ Move `src/io.rs` TCP helpers to examples/tests
18. ⬜ Make encryption on by default
19. ⬜ Implement congestion control
20. ⬜ Implement application-layer fragmentation

### Funded-Phase (Post-Grant)
21. ⬜ 20-node, 3-region pilot deployment
22. ⬜ NAT traversal (STUN hole punching)
23. ⬜ 7-day soak with churn/partition injection
24. ⬜ Baseline comparison (FedAvg + decentralized SGD)
25. ⬜ External security audit
26. ⬜ Independent reproduction
27. ⬜ Real-task learning benchmark (MNIST/CIFAR)
28. ⬜ Cross-platform benchmark corpus
29. ⬜ Valid network-partition experiment (netns/veth)

---

## 5. Metrics Summary

| Metric | Value |
|--------|-------|
| Tracked files | 487 |
| Rust source files | 51 |
| Lines of Rust (src/) | ~21,000 |
| Test functions | 342 (passing on Linux CI) |
| Property tests | 20 |
| Wire protocol tests | 14 |
| Stress tests | 9 (all `#[ignore]`d) |
| Security tests | 7 |
| E2E tests | 8 |
| Distributed learning E2E | 2 |
| Fuzz targets | 4 |
| Benchmarks | 19 |
| CI workflows | 7 |
| Documentation files | 39 (root) + 37 (docs/) |
| Paper drafts | 11 |
| Result files | 214 |
| Grant drafts | 8 |
| CodeQL alerts | 4 (1 fixed, 3 false positives) |
| GitHub commits | ~120+ |
| Evidence releases | 1 (v0.3.1) + 1 (evidence-2026-08-02-e12) |

---

## 6. Conclusion

Neuron-Wire is a **well-engineered, evidence-driven research prototype** with strong foundations in reproducibility, testing, and documentation. The main gaps are:

1. **No real-world deployment** (funded-phase)
2. **No external security audit** (funded-phase)
3. **Combined encrypted+learning E2E not yet demonstrated** (implementable now)
4. **Engine loop monolith** (refactoring needed for maintainability)
5. **ML scope creep** (14+ modules beyond core claim)
6. **Excessive paper/result file count** (repo hygiene)

The project is **grant-ready** — the evidence package is strong, the documentation is comprehensive, and the limitations are honestly documented. The next milestone should be implementing the combined secure distributed-learning E2E test (M1), which directly addresses the reviewer's top concern and strengthens the grant application.

---

*Report generated from repository ground truth at HEAD `8e77eb6` on 2026-08-26.*
