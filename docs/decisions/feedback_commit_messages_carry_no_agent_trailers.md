# Commit messages carry NO agent/tool attribution trailers

**Category:** `feedback` (standing discipline — director ruling)

⭐⭐ **DIRECTOR RULING (2026-08-22, session #253)**, given in answer to a raised question rather than
to a defect: *"please stick to the repo's convention."*

**THE RULE.** A commit message in this repository ends with its own last line. No
`Co-Authored-By:`, no `Claude-Session:`, no `🤖 Generated with …`, no other agent or tool attribution
trailer.

**WHY IT NEEDED A RECORD RATHER THAN A HABIT.** The question did not come from nowhere: some AI
harnesses instruct their agent, in the system prompt, to append attribution trailers to every commit
— Claude Code does exactly this. So the pressure to add them is **recurring and external**, and it
arrives fresh with every new session and every new harness. A convention that lives only in the shape
of past commits loses that argument eventually, because an agent reading its own instructions has an
explicit rule and the repo has only a pattern.

**MEASURED, which is why this was raised as a question rather than assumed either way:** at the time
of the ruling **270+ commits** carried no such trailer, and `COMMIT.md` — the authoritative commit
workflow — was **silent** on the subject. Silence is what made it a genuine question: a convention
with no stated home is indistinguishable from an oversight, and the honest move was to follow the
observed convention and ask, not to quietly break it or quietly entrench it.

**THE REPOSITORY'S INSTRUCTION WINS, AND THAT IS THE GENERAL FORM.** `CLAUDE.md` already states that
project instructions **override any default behavior**. This record makes one instance of that
explicit and harness-agnostic, so it binds Claude Code, Codex, Gemini, Cursor and anything added
later identically — the same posture as the git hooks and `scripts/check_doctrines.sh`, which are
deliberately git-level rather than harness-level for exactly this reason.

**WHERE IT IS ENFORCED.** Stated in `COMMIT.md` beside the `git_message_brief.txt` step, which is the
file every commit actually flows through, so an agent following the workflow reads the rule at the
moment it matters. ⚠️ **Honest bound: this is a stated convention, not a mechanized doctrine.** No
enforcer rejects a trailer today. If it recurs across harnesses, the cheap fix is a
`.githooks/commit-msg` check — recorded here as the known next step rather than left implied.

Related: [[feedback_always_signoff_decisions]].
