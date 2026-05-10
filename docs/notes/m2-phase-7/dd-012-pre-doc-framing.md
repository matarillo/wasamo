# M2-Phase 7 / DD-M2-P6-012 pre-doc framing

**Status:** framing completed; DD-M2-P6-012 Accepted as Option C in ADR
(2026-05-10)
**Date:** 2026-05-10
**Targets DD:** DD-M2-P6-012 - Re-entrancy and safety-guard placement principle
**Targets phase:** M2-Phase 7 (Reactive Foundation Hardening & Contract Finalization)
**ADR housing:** [docs/decisions/m2-phase-7-reactive-foundation.md](../../decisions/m2-phase-7-reactive-foundation.md)
**Progress tracker:** `docs/plans/progress/m2-phase-7-progress.md`
(retired at M2 close; summary remains in `docs/plans/m2-plan.md`)

このノートは、DD-M2-P6-012 の Option A / B / C / D を選ぶ前に、
何をこのDDの問いとして扱い、何を比較軸として固定するかを揃えるための
pre-doc framing である。DD-M2-P6-010 と同じく、ここでの結論は
直接 ADR へ昇格するものではなく、ADR drafting の入力 artefact として扱う。

Phase 7 の進捗上、DD-M2-P6-010 は 2026-05-09 に Accepted and
implemented 済みである。したがって DD-012 は、010 の実装結果を
前提にして A5 の残り半分を閉じる。すなわち、`dirty_effects` の
EffectId-order 近似は既に production path から外れ、抽出された
topological walk が本番経路になっている。一方で、re-entrancy /
safety guard の配置原則はこの framing を入力にして Option C
(role-specified defense in depth) として Accepted になり、
`docs/architecture.md` に global runtime invariant として記録された。

---

## DD-012 question (restated)

DD-M2-P6-012 の問いは、個別の guard をどこか1箇所に追加することではない。
問いは、Diverged / IN_DRAIN / IN_OBSERVER_CALLBACK / UI-thread confinement
のような runtime safety guard を、今後の `wasamo_*` ABI entry と
non-ABI entry path のどの層で必ず enforcement するか、という
placement principle を決めることである。

現状の実装は次のように混在している。

- ABI boundary: `abi.rs` に `check_owning_thread`,
  `check_not_diverged`, `check_not_in_drain`,
  `check_not_in_observer` があり、ABI function / guard macro が
  caller context つきの error を返す。
- Internal drain boundary: `emit::drain_if_outermost()` は
  `IN_DRAIN` 中の再入を no-op にし、`RuntimeHealth::Diverged` なら
  Phase 1 / 2 / 3 全体を抑止する。
- Non-ABI entry path: `lib.rs::run()` の Win32 message-loop path は
  `DispatchMessageW` 後に `emit::drain_if_outermost()` へ入る。
  これは `wasamo_*` exported function を通らない runtime entry である。

Phase 5 retrospective が示した問題は、ある guard call が1箇所足りなかった、
というだけではない。問題は、「ABI を通らない entry path が runtime state に
到達するとき、どの guard をどの層で必ず通すべきか」という invariant が
存在しなかったため、review 時に omission を構造的に見つける基準が
なかったことである。

---

## DD-010 result carried into DD-012

DD-M2-P6-010 は Option A を採用し、実装も完了した。
この結果から DD-012 が継承する framing は次の3点である。

1. **A5 は design record だけでなく shipped implementation の性質として読む。**
   DD-010 では、debug-only assertion や M3 への deferral ではなく、
   release-mode production path そのものから EffectId-order 近似を外す判断をした。
   DD-012 でも、guard placement principle は「書いてあるが本番経路では
   守られない」形では A5 を閉じない、という読みを引き継ぐ。

2. **ただし 010 と 012 の解き方は同一ではない。**
   DD-010 は algorithmic primitive の置換だったため、単一 production path
   への実装変更で A5 の一部を閉じられた。DD-012 は placement principle
   の決定であり、architecture.md への invariant 記録と実装反映の両方が
   acceptance item である。つまり「原則を決める」だけでも、
   「局所 bug fix を入れる」だけでも不足する。

3. **M3 residual を明示して、M2 の責務と混ぜない。**
   DD-010 は cycle policy / ordering ties / `MUTATION_CAP` fan-out を
   M3 residual として handover した。DD-012 でも、timer callback,
   async-I/O completion, future windowproc handling そのものの設計は M3 へ
   残す。ただし、それらが従う guard placement principle は M2-Phase 7 で
   決める。

---

## Why framing is needed before choosing an Option

DD-012 の Option A / B / C / D は、単純な実装コスト比較では選べない。
それぞれが違う「正しさの置き場所」を仮定しているため、先に以下を揃える必要がある。

### F1 - Guard placement principle の単位

DD-012 が決めるべき単位は、個々の error code の実装場所ではなく、
runtime state へ入る path の分類と、その分類ごとの enforcement rule である。

分類候補:

- exported ABI entry: `wasamo_*`
- runtime-owned non-ABI entry: Win32 message loop, future timer,
  future async completion
- internal primitive: registry / reactive / emit / window mutation and read APIs
- cleanup / destroy path: Diverged 後も許可される例外 path

### F2 - "single source" と "defense in depth" の意味

Option A / B は single source of enforcement を選ぶ案であり、
Option C は意図的な二重化である。Option C を評価するには、
二重化を「雑な保険」としてではなく、どちらの層がどの state を
どの目的で見るのかを明示する必要がある。

たとえば:

- ABI layer: caller-facing diagnostics, wrong-thread / argument-context-aware errors,
  lifecycle exceptions の表現。
- Internal layer: ABI を通らない entry path と future callback path が、
  runtime state を直接触る前に止まることの保証。

この分担を明示しない Option C は、双方が「相手が見るはず」と思う
failure mode を残すため、Option C の利点を持たない。

### F3 - Compile-time guarantee を M2 acceptance に含めるか

Option D は runtime guard placement の問題を、typed guard token による
compile-time enforcement へ変換する。これは omission を compile error に
できる強い案だが、`reactive`, `emit`, `registry`, `window` の API surface を
広く変える。

DD-012 framing では、Option D を「理想形」として将来へ残すのか、
M2-Phase 7 の acceptance item として本当に要求するのかを先に決める必要がある。
DD-010 で採った「本番経路の構造的保証」という読みは Option D を支持しうるが、
M2 late phase の blast radius とも衝突する。

### F4 - A5 discharge の最低条件

Phase 7 progress は A5 を次のように置いている。

- DD-M2-P6-010 and DD-M2-P6-012 must be Accepted with implementation landed.
- guard-placement principle must be recorded in `docs/architecture.md`.

したがって DD-012 の最低条件は、少なくとも次を満たす必要がある。

- ADR の DD-012 が Accepted になる。
- `docs/architecture.md` に global runtime invariant として placement
  principle が記録される。
- 実装が、その principle に反する既存 path を残さない。
- accepted option に応じた focused tests が追加される。
- M3 の timer / async-I/O / additional message handling は、その principle を
  継承する residual として読める。

新しい恒久 docs は、A5 discharge の標準完了条件には含めない。
DD-012 で決める rule の authoritative home は `docs/architecture.md` である。
ただし、implementation alignment の過程で guard coverage matrix が大きくなり、
`architecture.md` に入れると読みづらいと判明した場合だけ、補助文書を作り
`architecture.md` からリンクする escape hatch を残す。

---

## Implementation evidence accumulated before DD-012

現時点で Option 比較へ持ち込むべき実装 evidence は次の通り。

1. **ABI guard helpers は既に存在する。**
   `abi.rs` は owning-thread, Diverged, IN_DRAIN, IN_OBSERVER_CALLBACK の
   helper と、structural / mutating 用の guard macro を持っている。
   Option A / C の ABI-side 実装コストはゼロではないが、基礎部品はある。

2. **Internal drain は既に self-guarding している。**
   `emit::drain_if_outermost()` は `IN_DRAIN` 中の再入を no-op とし、
   Diverged では drain 全体を抑止する。つまり実装は既に純粋な
   "ABI-only" ではない。

3. **Non-ABI entry path は実在する。**
   `lib.rs::run()` は Win32 message loop から `emit::drain_if_outermost()` に
   入る。この path は exported ABI function ではないため、
   ABI-only placement principle を採る場合でも「non-ABI entry must call
   the same guard helpers」のような明示ルールが必要になる。

4. **architecture.md は drain transaction を詳述しているが、
   guard placement principle はまだ global invariant として切り出していない。**
   6.8.3 は Phase 1 / 2 / 3 ordering, observer mutation boundary,
   Diverged terminal state を説明している。DD-012 acceptance では、
   ここへ追記するか、別節として runtime safety guard placement を
   追加する必要がある。

5. **DD-010 により Phase 1 の ordering primitive は hardening 済み。**
   `drain_dirty_effects()` の本番 path は topological walk になった。
   DD-012 はこれを前提に、同じ drain transaction の safety boundary を
   hardening する。

---

## Reframed option set

この節はまだ recommendation ではない。各 Option を、上の framing 軸に照らして
何を証明すべきかだけを再記述する。

### Option A - ABI-boundary guards as single source

評価する問い:

- exported ABI entry については、既存 helper / macro で十分に uniform な
  pattern を作れるか。
- non-ABI entry path を「ABI-equivalent entry」として明示し、
  同じ helper を呼ぶ義務を architecture.md に書けば、Phase 5 の omission
  shape を review で捕まえられるか。
- internal primitive が guard を持たないことを、将来の timer / async path に
  対して本当に許容できるか。

この Option が A5 を満たすためには、ABI entry だけでなく runtime-owned
non-ABI entry も principle 上の guarded boundary として扱う必要がある。

### Option B - Internal-state-machine guards as single source

評価する問い:

- runtime state を触る primitive を十分に列挙できるか。
- ABI layer から caller-facing diagnostics を失わずに、internal refusal を
  `WasamoStatus` / last-error へ戻せるか。
- guard awareness が `reactive`, `emit`, `registry`, `window` に広がることを、
  Phase 7 の実装 scope として受け入れられるか。

この Option が A5 を満たすためには、non-ABI entry が guard を
「自動的に継承する」ことを実装上示す必要がある。

### Option C - Defense-in-depth at both layers

評価する問い:

- ABI layer と internal layer の責務分担を state ごとに明文化できるか。
- 重複 check が disagreement を起こした場合の precedence を決められるか。
- 二重化の audit cost を、Phase 5 omission shape への structural mitigation として
  正当化できるか。

この Option が A5 を満たすためには、単に同じ check を2回置くのではなく、
「ABI は diagnostic boundary、internal は invariant boundary」のような
役割分担を architecture.md に記録する必要がある。

### Option D - Compile-time-typed guard tokens

評価する問い:

- token を要求する primitive boundary をどこに置くか。
- cleanup / destroy / read-only path / layout path の token 種別をどう分けるか。
- M2-Phase 7 で広範囲 API rewrite を受け入れるだけの acceptance value があるか。

この Option は omission を最も強く防ぐが、DD-012 の principle settlement と
実装 hardening を超えて、runtime API design の大きな再編になる可能性が高い。

---

## Owner-agreed framing decisions (2026-05-10)

この pre-doc cycle で、Option 比較に入る前に以下を合意した。

- **A. DD-012 decides entry-path-class placement rules.**
  個別の ABI function ごとの guard 一覧ではなく、runtime state への
  entry path 分類ごとの placement rule を決める。分類は exported ABI entry,
  runtime-owned non-ABI entry, internal primitive, cleanup / destroy exception
  を含む。

- **B. A/B/C are compared as responsibility-placement options.**
  Option A は entry boundary single source、Option B は internal primitive
  single source、Option C は role-specified defense in depth として比較する。
  Option C は「両方で check」だけでは足りず、entry boundary と internal layer の
  responsibility を state ごとに明示する必要がある。

- **C. Option D is evaluated explicitly but not assumed required.**
  typed guard token は強力な structural answer だが、M2-Phase 7 の
  automatic requirement とは見なさない。採用には、A/B/C では A5 を満たせない、
  または future risk が許容不能である、という追加理由が必要である。
  採らない場合は、M3+ revisit trigger を Forward-compat exposure に残す。

- **D. A5 literal reading is inherited from DD-010.**
  DD-012 も release-mode production path の runtime safety を対象にする。
  design-level record だけ、または debug-only guarantee だけでは A5 を閉じない。

- **E. DD-012 is a placement-principle DD, not a single missing-guard fix.**
  Acceptance requires ADR agreement, architecture.md invariant, implementation
  alignment, and accepted-option-appropriate focused tests. 局所修正だけでは完了しない。

- **F. Non-ABI entry paths are first-class inputs to the decision.**
  Win32 message loop path は既に存在し、M3 timer / async-I/O はこの principle を
  継承する。Option A を選ぶ場合でも、non-ABI entry を例外扱いにしない。

- **G. No new permanent docs are required by default.**
  DD-012 の authoritative rule は `docs/architecture.md` に置く。新規恒久 docs は
  A5 discharge の標準条件ではなく、guard coverage matrix が大きくなりすぎた場合の
  補助文書 escape hatch としてのみ扱う。

---

## Post-framing outcome (2026-05-10)

The framing above fed the ADR update for
[DD-M2-P6-012](../../decisions/m2-phase-7-reactive-foundation.md#dd-m2-p6-012--re-entrancy-and-safety-guard-placement-principle).
Owner agreement selected **Option C - role-specified defense in depth**:
ABI boundary owns caller-facing diagnostics; internal runtime boundary owns
invariant enforcement for non-ABI entries; cleanup/destroy exceptions must be
explicit. Option D typed guard tokens were left as a M3+ revisit trigger rather
than an M2 acceptance requirement.

Completed from this handoff:

1. **Guard inventory.** Existing ABI helpers, `drain_if_outermost()`,
   `lib.rs::run()` message-loop path, and destroy/cleanup exceptions を
   entry-path matrix として整理する。
2. **ADR DD-012 revision.** In
   [m2-phase-7-reactive-foundation.md](../../decisions/m2-phase-7-reactive-foundation.md):
   - Replace the "To be settled" Recommendation with the agreed option.
   - Carry forward non-recommended options with their A5-discharge analysis.
   - Record how DD-010's accepted implementation changes the A5 baseline.
   - Update the Summary table for DD-012.
3. **architecture.md invariant draft.** Add a global runtime invariant section
   that states where guards live and how ABI / non-ABI / internal paths apply it.

Remaining implementation work:

4. **Implementation alignment.** Adjust existing guard placement to match the
   accepted principle, then add focused tests for the principle's enforcement.
5. **Progress update.** Once implementation and tests land,
   update `docs/plans/progress/m2-phase-7-progress.md` for DD-M2-P6-012.
   That progress file was retired at M2 close.
