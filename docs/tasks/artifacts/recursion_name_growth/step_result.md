# PGEN-RGX-0078-0190 result

Verdict: **HOLD; exact name-stack growth contributes 4.005 ns but practical
margin remains only 1.425 ns.**

The full-SHA-pinned classifier verifies four inline name-growth calls, one
outlined name-growth call, and the separate required ID-growth control call in
the preserved binary. It then uses only their exact return-address frames in
the three banked call trees. The resulting child counts are **33/43/34** with
zero parent residual; aggregate `Vec<24>` symbol rows and unrelated consumers
are excluded.

Inline calls contribute **0.672504411 ns** and the outlined `enter_id` arm
contributes **3.332561166 ns**. With the 0.7% corpus tail assigned zero, the
conservative full-corpus contribution is **0.317006932% = 4.005065577 ns**.
The covered-band normalized cross-check is 4.033298668 ns.

The honest pre-lookup bundle becomes **100.749486548 ns**; 30% capture is
**30.224845964 ns**, only **1.424845964 ns** above the 28.8 ns same-binary noise
floor. The deliberately optimistic thin-index composite reaches 2.572227983 ns
margin, but both views still omit mandatory error-path ID-to-name
reconstruction and representation/model uncertainty. This is smaller than
previously rejected practical-margin slivers, so no regeneration chain is
licensed.

No parser, runtime, emitter, generated artifact, corpus, probe, capture, floor,
MAX, public contract, mdBook, or product behavior changed.
