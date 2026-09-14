// ENGINE-UNIVERSAL-SERVICES.107 / SV-CORPUS-GRAD.3.62 - the same defect one byte along: through
// 1.0.230 this was REJECTED, but for the wrong reason - the skipper had already swallowed the `/*`
// and the parse died on the leftover `x` at end of input. The refusal must land ON the `/*`.
/* x
