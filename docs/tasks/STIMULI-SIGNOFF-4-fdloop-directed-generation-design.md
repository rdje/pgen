# STIMULI-SIGNOFF.4 — Directed / learned generation (FDLOOP): research read-out, reconciliation, and design

- Owning leaf: `STIMULI-SIGNOFF.4` (co-owned with `SV-EXH-PROOF.7.4.6` for the SV application)
- Slice: research + DESIGN (docs-only), session #58, 2026-07-07
- Doctrine basis: [[feedback_research_grounded_sota_no_trial_and_revert]] (survey first, act on a
  citation + worked mapping), [[feedback_ast_pipeline_parser_agnostic]] (general capability, no
  name-gates), [[feedback_no_codebase_change_without_tool_backed_facts]] (this doc contains the
  facts the code slices will act on).

---

## 1. The paper (read in full via the arXiv HTML, 2026-07-07)

**FdLoop — "Directed Grammar-Based Test Generation"**, Lukas Kirschner (Saarland University /
University of Luxembourg) & Ezekiel Soremekun (Singapore University of Technology and Design),
arXiv **2508.01472** (submitted to IEEE TSE, 2025). Replication package: `tinyurl.com/FDLoop-V3`.

### 1.1 The algorithm (the six-stage feedback loop)

Per generation (iteration), FdLoop runs:

1. **Grammar learning** — parse the current seed/selected inputs into derivation trees; count how
   often each alternative of each production expands; normalize per rule into a **probabilistic
   grammar** (e.g. alternatives seen 10/30/60 times → probabilities 0.1/0.3/0.6).
2. **Generation** — sample `n` new inputs from the learned probabilistic grammar (stochastic
   alternative choice by learned probability).
3. **Mutation** — produce `n` more candidates via bit-flip AND parse-tree-swap mutations
   (subtree exchange only between subtrees rooted at the same rule → validity-preserving).
4. **Execution + feedback** — run all `2n` candidates against the subject program; collect the
   goal metrics.
5. **Selection** — `selectBestInput(f, T, P, F)`: keep the SINGLE input maximizing a weighted-sum
   fitness `F = μ₁x₁ + μ₂x₂ + μ₃x₃ + μ₄x₄` over normalized-to-[0,1] goal metrics.
6. **Grammar refinement** — re-learn probabilities from the selected input's derivation tree,
   then **mutate the learned grammar**: reset randomly-chosen rules' distributions back to
   uniform, to keep exploring beyond what the best-so-far input exhibits.

Loop for ~48–50 generations; the selected inputs accumulate in a set `I` that biases learning.

### 1.2 Goals (fitness ingredients)

- **Code coverage** (fraction of subject methods executed),
- **Mappings** `M := F × E` (unique ⟨observed-grammar-production, executed-function⟩ pairs —
  input-feature × code-behavior interaction),
- **Exceptions** (total + unique),
- **Execution time**.
Weights `μᵢ` steer single- vs multi-goal runs (e.g. weight 10 on one goal to focus it).

### 1.3 Generation mechanics and depth control

Stochastic top-down expansion by learned probability; **after expansion depth 3 the generator
switches to the alternative leading to the shortest possible subtree** (their anti-blowup rule —
functionally the same idea as PGEN's Purdom min-terminal ordering / `construct_mode`).

### 1.4 Headline results and limitations (authors' own)

- Beats all 5 baselines (random, uniform-grammar, static-probabilistic, inverse-probabilistic,
  EvoGFuzz; plus DynaMOSA off-grammar) in **86 % of settings**; up to **2×** EvoGFuzz on
  exception-revealing (Mann-Whitney p = 0.009). Subjects: JSON/CSS/JavaScript grammars over 20
  open-source Java projects.
- Limitations they state: Java-only subjects; heavy compute (72 CPU-days for the 4-subject
  sensitivity study); manually-chosen fitness weights; needs a formal grammar; weighted-sum
  scalarization can miss Pareto-optimal trade-offs.

### 1.5 The one-line thesis, PGEN terms

FdLoop = **learn a per-choice-point probability distribution from inputs that score well on a
goal, generate from that distribution, keep the best, re-learn, and deliberately re-flatten parts
of the distribution to keep exploring**. It directs the WHOLE distribution of generated inputs at
a goal — it does not force a single named target.

---

## 2. Reconciliation with the co-owned `SV-EXH-PROOF.7.4.6` (MANDATED by the resume pointer)

The `.4` leaf was written (2026-06-03) when the SV closed-loop residual stood at **273** and the
deterministic reach-plan SEARCH was timing out on deep targets; its acceptance said "SV residual
273 → ~0". That framing is now **partially superseded** — the `.7.4.6` arc landed a stronger,
deterministic mechanism for exactly that problem:

| `.7.4.6.x` | What landed | Residual effect |
|---|---|---|
| `.4.4`/`.4.5` | Purdom witness ordering + decoupled witness budget | 753 → 273 |
| `.6.3` | **Derivation-directed CONSTRUCTION** (`construct_mode`: commit to the single shortest-terminating branch, min-repeat quantifiers, backtrack-FREE, O(tree-size), falls back to search) | 273 → 97 |
| `.6.4`/`.6.5` | Timeout root cause = allocation churn; `strip_probability_prefix` → `Cow` | witness ~2× faster |
| `.6.7` | Phantom/duplicate branch de-dup (25 permanently-uncoverable targets out of the universe) | metric unmoved — see below |
| `.6.6`/`.6.7` findings | Residual decomposes into (A) deep-recursion witness TIMEOUTS (budget-solvable, `property_expr`-class) + (B) shadowed/dead branches (PEG-shadowing / universe-classification territory, PARSE-SOTA lint); and the metric itself is **NOISE-DOMINATED** (runs read 97/89/105/120, ±~25) | best-known ≈ **84**; literal-0 unprovable against a ±25-noisy number |

Consequences for this leaf (the honest re-frame):

1. **The reach problem FdLoop was recruited for is already solved deterministically.**
   Construction reaches deep targets in bounded time with NO search; a learned distribution can
   only reach them probabilistically. For single named targets, construction dominates FdLoop —
   adopting FdLoop for that would be trial-and-revert against a better landed mechanism.
2. **The remaining SV residual is not a distribution problem.** (A) is a budget/perf question and
   (B) is a proof/classification question; neither is "the distribution never goes there".
   The SV-side prerequisite (make the residual metric deterministic, `.7.4.6.8`) stays owned by
   `SV-EXH-PROOF` — this tree must NOT chase a ±25-noisy number.
3. **What survives — and it is substantial — is the GENERAL capability**: PGEN today has NO way
   to (a) LEARN a generation distribution from a corpus, nor (b) drive generation at a
   DISTRIBUTIONAL goal (a property of the whole sample population, not a single branch). Every
   current steering mechanism is either uniform/declared-probability sampling
   (`build_weights`, `stimuli_generator.rs:10444`), hand-tuned coverage-deficit multipliers
   (`coverage_guidance_multiplier`, `:10702`), or single-target forcing (reach plans `:2782`,
   `construct_mode` `:1421`). That is the gap `.4` closes.

The leaf's acceptance is re-written accordingly (see §5): the SV literal-0 lane becomes a
*measurement consumer* of this capability (after `.7.4.6.8` lands determinism), not its
acceptance bar.

---

## 3. Worked mapping — FdLoop stages onto the PGEN generator (symbol-cited)

All paths `rust/src/ast_pipeline/stimuli_generator.rs` unless noted. Line numbers at HEAD
(`PGEN-STIMULI-SIGNOFF-0008`).

### 3.1 What PGEN already HAS per FdLoop stage

| FdLoop stage | PGEN surface | Status |
|---|---|---|
| 2. Generate from probabilistic grammar | `generate_or` phase-2 weighted tournament: grammar-declared per-branch probability prefixes (`strip_probability_prefix` `:10381`), `build_weights` `:10444`, `WeightedIndex` sampling, seeded `StdRng::seed_from_u64` `:1638` | HAVE (declared, static probabilities — not learned) |
| 3. Validity-preserving mutation | `StimuliMutationMode::GrammarAwareLocal` `:149` (grammar-aware local mutation ≈ parse-tree-swap); `NearValidLocal` negative profile `:173` (≈ bit-flip-class near-miss) | HAVE |
| 4. Execute + feedback | closed-loop parseability round-trip (the parser IS our subject program); rule/branch coverage counters (`record_branch_selected` `:408`, `record_branch_success` `:434`); k-path recorder (`covered_k_paths` `:6166`, `k_path_coverage_report` `:6188`, leaf `.2`) | HAVE (feedback signals exist; nothing consumes them as FITNESS) |
| 1/5/6. Learn → select-best → re-learn loop | `run_coverage_guided_fuzz_loop` (`main.rs:4099`, `--coverage-guided-fuzz-rounds` + `--coverage-guided-fuzz-seed-start`) | PARTIAL — a rounds loop with coverage ACCUMULATION only: no learned distribution, no fitness, no select-best, no re-learn/refine |
| Depth control (their depth-3 shortest-subtree rule) | Purdom min-terminal table (`compute_min_terminal_lengths` `:6303`, `witness_min_terminal_lengths`) + `construct_mode` `:1421` | HAVE — strictly stronger (deterministic, provably bounded) |

### 3.2 What PGEN LACKS (the FdLoop delta = the `.4` capability)

1. **A learned per-choice-point distribution layer.**
   `LearnedBranchDistributions = HashMap<group_key, Vec<u64>>` where `group_key` is the existing
   `"{rule}::{node_path}"` branch-group key (the same key `coverage_guidance_multiplier` and the
   coverage record already use — structural, parser-agnostic, zero name literals). Consulted in
   `generate_or` phase-2 as a weight source composed with (not replacing) the declared
   probabilities and coverage-deficit multipliers. **Absent map ⇒ byte-identical generation**
   (the `.12`/`.7.4.x` default-OFF monotonicity pattern; proven by the seeds-0/7/42 byte-compare
   across shipped grammars).
2. **Learning front-ends** (two, per input source):
   - **(a) Self-derivation counting** — when the generator itself produced the sample, it can
     record its own per-sample branch choices (a per-sample selection log alongside the existing
     cumulative counters) and `learn_branch_distributions(&[samples])` folds the accepted
     (round-trip-passing) samples' counts into the map. No parsing needed; free attribution.
   - **(b) External-corpus learning** — for inputs PGEN did NOT generate (real-world files), the
     branch attribution comes from the **gen-AST interpreter**
     (`rust/src/parse_harness_interpreter.rs`, TOOLBOX 1.5): it walks the SAME normalized gen-AST
     the generator consumes and knows `(rule, node_path, branch_index)` natively at each choice it
     resolves — the parser-agnostic derivation-tree counter FdLoop's stage 1 needs. (The typed AST
     alone cannot give this — return annotations fold structure away; the `_meta` carrier is
     deferred. The interpreter is the landed, differential-certified surface that has exactly this
     information in hand.)
3. **The directed loop driver** (FdLoop stages 4–6 grafted onto the existing rounds loop):
   per round — generate `n` from the current distribution + `n` grammar-aware mutants → score
   each sample with a **goal fitness function** → select the best `k` (FdLoop uses k=1; keep `k`
   a parameter) → re-learn the distribution from the accumulated selected set → **distribution
   mutation**: reset the distribution of a deterministically-RNG-chosen rule back to uniform
   (their exploration device, seeded via the run's `StdRng` so runs are reproducible).

### 3.3 The PGEN goal vocabulary (fitness functions, v1)

Each goal is deterministic, machine-measurable, and already has (or gets) a report surface:

- **G1 — k-path coverage delta** (primary; lands first). Fitness of a sample = number of NEW
  k-paths it covers (from the `.2` recorder). The loop's headline metric = `covered/universe`
  before → after. SV sv_2017 k=2 baseline is already measured: **679/4105 = 16.5 %**
  (`PGEN-STIMULI-SIGNOFF-0004`) — a ready-made, deterministic before→after for the acceptance
  checklist, on the very metric this tree defined as the signoff bar.
- **G2 — parse-failure revelation** (the FdLoop "exceptions" analogue, PGEN-shaped): fitness
  rewards samples the parser REJECTS (generator-emitted-but-parser-rejected = a generator⟷parser
  duality break, exactly the class [[feedback_be_alert_root_cause_fishy_immediately]] treats as a
  DEFECT to surface). Turns the generator into an active duality-break hunter instead of a
  passive round-trip checker. Every hit feeds the shrinker + a tracked defect.
- **G3 — corpus-mimicry** (needs front-end (b)): learn the distribution from an external corpus
  and generate inputs distributionally LIKE it (realistic stimuli — e.g. SV that statistically
  resembles real RTL rather than uniform-grammar soup). Fitness = distribution proximity
  (per-group L1 distance between the sample's counts and the corpus distribution).
- **G4 — parser code coverage** — explicitly DEFERRED to leaf `.3` (its own gap); the fitness
  plug-in point designed here is where `.3`'s signal will connect (input-k-path → code-coverage
  is the Havrikov-Zeller chain; FdLoop's "mappings" goal is the same idea).

### 3.4 Determinism, monotonicity, parser-agnosticism (project invariants)

- **Determinism**: every stochastic step (sampling, distribution mutation, mutant choice) draws
  from the run's seeded `StdRng` (`:1638`); the loop is reproducible per seed; verification runs
  the canonical 0/7/42 triplicate. FdLoop's own stochasticity is the only impedance mismatch with
  PGEN doctrine and seeding resolves it.
- **Monotonicity / no-regression**: the learned layer is default-OFF (no map installed ⇒
  byte-identical output, proven by byte-compare); the directed loop is a NEW opt-in mode (CLI
  surface extending the existing `--coverage-guided-fuzz-rounds` family) — the diverse pass, the
  witness pass, cert-coverage, and every gate are untouched.
- **Parser-agnostic**: keyed exclusively on `(rule, node_path, branch_index)` structure — the
  same keys the coverage record uses; zero grammar-name or rule-name literals
  ([[feedback_features_parser_agnostic_enable_all_parsers]]).

---

## 4. Slice plan (each slice a leaf-owned code change with its own acceptance checklist)

- **`.4.1` — the learned-distribution layer** (bounded): `LearnedBranchDistributions` store +
  `generate_or` phase-2 hook (compose with declared probabilities × coverage multipliers) +
  self-derivation per-sample selection log + `learn_branch_distributions(samples)`.
  Proof: default-OFF byte-identity at seeds 0/7/42 across the shipped grammars (cert +
  corpus byte-compares, the `.12` discipline); unit tests on a synthetic grammar showing a
  learned skew provably shifts sampling; clippy; both lib suites.
- **`.4.2` — the FdLoop driver, goal G1** (the capability landing): fitness/select-best/re-learn/
  distribution-mutation loop extending the `run_coverage_guided_fuzz_loop` shape (new CLI mode,
  e.g. `--directed-generation-goal k_path --directed-rounds N`); measured k-path coverage
  before→after on ≥2 grammars (SV sv_2017 k=2 from the 16.5 % baseline + one small fully-certified
  grammar as the control); deterministic per seed; report surface (JSON summary like the fuzz
  replay report).
- **`.4.3` — external-corpus learning (front-end (b)) + goal G3**: interpreter-driven derivation
  counting (`interpret_parse` walk exposes `(rule,node_path,branch)` selections); demonstrate
  corpus-mimicry on a real corpus (candidates: the regex PCRE2 corpus bundle or the SV external
  corpus files); fitness = per-group L1 proximity; measured "mimicry score" report.
- **`.4.4` — goal G2 (duality-break hunter)**: fitness = parser-rejection revelation; wire hits
  into the existing shrinker (`shrink_parseability_counterexample`, `main.rs:3773`) + a tracked
  finding per unique rejection signature. (Order `.4.3`/`.4.4` may swap on evidence; `.4.2` first
  is fixed — it is the smallest slice that proves the loop works end-to-end.)
- **`.4.5` — SV lane measurement** (the reconciled `.7.4.6` co-ownership): ONLY after
  `SV-EXH-PROOF.7.4.6.8` makes the residual metric deterministic, run the directed loop with a
  residual-targeting fitness and measure honestly against the deterministic number. This slice is
  a MEASUREMENT consumer, not the capability's acceptance bar.
- Lockstep per slice: book (stimuli chapter + annotation/CLI surfaces as touched), steering
  matrix / normative spec only if a new directive appears (none planned — this capability is
  CLI/engine-level, not grammar-annotation-level), CHANGES/DEVELOPMENT_NOTES/MEMORY, KM card when
  the capability lands.

## 5. Leaf acceptance re-frame (recorded in the tree same-commit)

OLD: "directed generation reaches targets the search misses; SV residual 273 → ~0;
parser-agnostic."

NEW: "(1) a learned per-choice-point distribution layer, default-OFF byte-identical, seeds
0/7/42-proven; (2) a deterministic directed-generation loop (learn → generate+mutate → score →
select → re-learn → explore) driving at least goal G1, with a measured k-path coverage
improvement over the diverse pass on ≥2 grammars from the 16.5 % SV baseline; (3) external-corpus
learning via the interpreter front-end with a measured mimicry demonstration; (4) parser-agnostic
throughout (structural keys only); (5) the SV residual lane re-measured only against the
deterministic metric once `SV-EXH-PROOF.7.4.6.8` lands (co-ownership honored, noise never
chased)."

## 6. Novel-direction findings surfaced to the director (durable copy)

- **Corpus-mimicry (G3) is a genuinely NEW capability direction** unlocked by the FdLoop
  mechanism but not named in the original gap list: "generate stimuli that statistically resemble
  THIS corpus" (e.g. UVM-like SV, real-world-like regexes) — realistic-workload stimulus
  generation for downstream consumers (Nexsim), beyond coverage pushing. Raised prominently in the
  session report; recorded here durably.
- **The honest supersession**: FdLoop's headline use-case (reach deep targets search misses) is
  already better-served in PGEN by landed deterministic construction (`.7.4.6.3`). The literature
  survey said "read FDLOOP before building" — reading it AFTER construction landed shows the
  right adoption is the distributional loop, not the reach mechanism. This is the
  research-grounded doctrine working as intended (no trial-and-revert).
