# Services Register — PGEN as a Service Provider

> **Direction (2026-10-19, the director, verbatim):** *"I would like PGEN become a truly a very high quality parsing,
> or compilation and/or elaboration service provider for downstream consumers, projects. If PGEN parsers can share
> services other PGEN parsers, and become like lego blocks where some of this shared services can themselve be
> leveraged by downstream consumers too, that's a win on many account."* — and: *"I really want PGEN to become like the
> Swiss-Army-Knife of all things parsing, compilation and eloboration, made of very well contracted parts or services
> other downstream projects can seamlessly leverage for their own benefit."* Recorded in
> `docs/decisions/project_pgen_is_a_service_provider_shared_contracted_services.md`; the tree is `PGEN-SERVICES`.

## What a service is here

A **service** is a part of PGEN that more than one parser uses, or that a downstream project could use on its own,
behind one boundary. The register gives every candidate five columns, each measured before it is written:

- **Contract** — the document a consumer codes against.
- **Stability axes** — the service's own release version and, where it emits structured output, an output schema
  version; independent of any parser's release and schema.
- **Neutrality tier** — *engine* (knows no language; `ENGINE-NEUTRALITY` enforces it) or a *family component*
  implementing a neutral socket.
- **Consumers** — the PGEN parsers that use it, and the surface a downstream project reaches it through (Rust typed,
  by-name, CLI).
- **Currency gate** — the gate that holds the contract to the code and refuses on drift.

A row with all five is a **block** — a lego brick in the director's sense. A row missing any is a **component**: real,
used, but not yet something another project can lean on. A **spec** is a contract with no implementation behind it.

## The register (first measurement, 2026-10-19)

| # | Service | Contract | Axes | Tier | Currency gate | Verdict |
|---|---|---|---|---|---|---|
| 1 | Embedding API — parse, AST dump, typed outcomes, by-name forms | `rust/docs/EMBEDDING_API_CONTRACT.md` | API `2.2.0`; per family: parser release + AST-dump schema | engine | `embedding_api_gate`, `PUBLISHED-VERSION-CURRENCY` | **block** |
| 2 | AST / JSON carrier — `AstDumpPayload`, the typed shapes, the annotation inventories | the embedding contract; each family's contract and `json-carrier.md` | per-family AST-dump schema | engine carrier, family shapes | `PARSER-BOOK-CURRENCY`; the shape-contract manifests | **block** |
| 3 | Parse limits + the dedicated parse stack | the embedding contract (`1.3.x`) | the API version | engine | `embedding_api_gate` | **block** |
| 4 | SystemVerilog standard-library bundle (embedded; one override) | the embedding contract (`2.1.0`); the SV contract | the API version; the SV release | family component | `embedding_api_gate`; `SV-CONTRACT-CURRENCY` | **block**, family-scoped |
| 5 | Preprocessing — the neutral stage socket + the SystemVerilog expander | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` (`1.0`, 2026-10-20) | service release `1.0.0` (`PREPROCESSING_SERVICE_VERSION`); output schema `1` (`OUTPUT_SCHEMA_VERSION`) → `2` at `SVPP-EXPANSION.9.3` | engine socket (+ the neutral `FileProvider`), family implementation | `SVPP-EXPANSION-CONTRACT-CURRENCY` (`scripts/check_svpp_expansion_contract_currency.sh`) | **block** since `.9.2` |
| 6 | Source text reading — `pgen::source_text` | none (published in this book's embedding chapter only) | none | engine | none | **component** — finding, routed to `PGEN-SERVICES.3` |
| 7 | Stimuli generation + the certificate | none as a service | none | engine | `GRAMMAR-CERT-CURRENCY` (the table) | **component** — candidate for downstream use |
| 8 | Semantic store | four `PGEN_SEMANTIC_STORE_*` documents (spec `0.1.0`) | the spec version | engine by design | none (no code) | **spec** |
| 9 | The family socket (`FamilyHooks`) | the `ENGINE-NEUTRALITY` register | none | engine | `ENGINE-NEUTRALITY` | **component**, internal |
| 10 | The parse harness | none | none | engine | the gates | **component**, internal |
| 11 | Keyword version tables (IEEE 1800 / 1364 reserved words per version) | the SV contract's §22.14 text | the SV release | family component | `SV-KEYWORD-VERSIONS` | **component** — candidate |
| 12 | Parser generation from EBNF (the generator, the self-hosted meta-grammar) | partial: the two annotation contracts; the EBNF dialect in this book | none for the generator | engine | the self-hosting dual run; `EBNF-ALTERNATIVE-PARITY` | **component** (contract partial) |

The full rows — consumers named, every cell with the command that re-derives it — are in `docs/tasks/PGEN-SERVICES.md`.

## What it says

PGEN has one contracted service family today, the embedding API and the three things contracted inside it, and it is
the model for the rest: a versioned document, a gate that holds it, a surface every parser shares. The next block is
preprocessing (`SVPP-EXPANSION.9`, in flight — see the SystemVerilog preprocessor book's chapter *Preprocessing as a
Service*). The one finding is a published surface without a contract (row 6), owned by `PGEN-SERVICES.3`. The two
candidates (rows 7 and 12) become services when a downstream project names the need; the specification (row 8) is a
contract waiting for its implementation. The register's own doctrine — a row that claims a contract names a gate that
runs; a published surface without a row is a finding — is `PGEN-SERVICES.2`.
