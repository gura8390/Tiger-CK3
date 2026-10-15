# Tiger CK3 — CK3 1.20.1 adaptation

A community-maintained adaptation of [amtep/tiger](https://github.com/amtep/tiger) for **Crusader Kings III 1.20.1 (Crozier)**. It checks mod scripts for syntax, references, scope errors, and selected GUI/data-binding problems.

**This repository extends Tiger 1.19.0 with CK3 1.20.1 rules and diagnostic fixes.** It is independently maintained, is not an official upstream release, and currently publishes source code rather than downloadable binaries.

中文说明：本项目基于原版 Tiger 1.19.0，适配 CK3 1.20.1，补充新版本规则并修复部分误报。下文列出相对上游的实际改动、构建方法和检查边界。

## Credits and thanks / 致谢

Tiger was created by [@amtep](https://github.com/amtep). Thank you to @amtep and all [upstream contributors](CONTRIBUTORS.md) for building and maintaining Tiger, and for the time, expertise, and care invested in its tools. This adaptation builds on their work and retains the upstream history, author notices, and license.

感谢原作者 [@amtep](https://github.com/amtep) 及所有上游贡献者创建和持续维护 Tiger，感谢他们为模组检查工具投入的时间、专业知识与心血。

## What changed from upstream?

Baseline: upstream [Tiger 1.19.0, commit `9e3d3b25`](https://github.com/amtep/tiger/commit/9e3d3b25c7524cdcc86d5f4190f3f8cffbc4d630). Adaptation revision: **CK3 1.20.1 maintenance-r3**. The CLI retains the original package version with a local adaptation suffix; it does not indicate a new official Tiger release.

| Area | Changes in this adaptation |
|---|---|
| CK3 1.20 content | Updated law/law-group handling and religious definitions; added rite, tenet, puppet, spiritual-fulfillment, and election-candidate-set support. |
| Script rules and scopes | Added or updated CK3 triggers, effects, iterators, scope links, and related item references for the target game version. |
| Decisions and interactions | Updated decision/widget controllers, interaction fields, and related scope setup; improved interaction-category index diagnostics. |
| GUI and data bindings | Added selected interface types and member signatures, corrected member/global binding expansion, and checked invalid direct widget nesting. Some return types remain incomplete. |
| Macro and scope diagnostics | Separated unfinished macro expansion state between workers, retained call-site context in cached results, and adjusted guarded scope probes and list-construction checks to reduce specific false positives. |
| Government definitions | Checked the final merged government registration list for omissions and duplicates. An optional project policy can warn about missing acquisition gates. |
| CLI and CI integration | Added opt-in failure thresholds, a separate metadata report, native source-line comparisons, and GUI state-command auditing. Version mismatches warn and validation continues. |

These are targeted rules and fixes, not a claim that all CK3 1.20.1 syntax or engine behavior is supported. The inherited source tree also contains other game backends; **this adaptation is maintained for CK3**.

## Build

Install a current stable Rust toolchain and the build tools required by its Windows target (Visual Studio C++ Build Tools for MSVC, or a configured GNU toolchain). From the repository root:

```powershell
cargo build --release -p ck3-tiger --locked
```

The Windows command-line executable is written to `target/release/ck3-tiger.exe`, unless `CARGO_TARGET_DIR` overrides the output directory. The current adaptation has been checked with a Windows GNU toolchain; this does not establish Linux, MSVC, or minimum-supported-Rust compatibility.

## Check a mod

Provide your own CK3 installation and mod descriptor:

```powershell
.\target\release\ck3-tiger.exe --game "C:\Games\Crusader Kings III" "C:\Mods\MyMod\descriptor.mod"
```

For JSON output with an explicit CI error threshold:

```powershell
.\target\release\ck3-tiger.exe --game "C:\Games\Crusader Kings III" --json --no-color --fail-on error "C:\Mods\MyMod\descriptor.mod" > diagnostics.json
```

Run `--help` for all arguments. See [ck3-tiger.conf](ck3-tiger.conf) for configuration, [filter.md](filter.md) for output filtering, and [annotations.md](annotations.md) for inline suppression comments. Filtering affects visible reports and the failure threshold; use a reviewed configuration when treating results as a CI gate.

### Added optional arguments

| Argument | Purpose and limits |
|---|---|
| `--fail-on error\|warning\|fatal` | Return a nonzero exit code when visible diagnostics reach the selected threshold. Without this option, legacy exit behavior remains. With it: threshold-reaching error/fatal = 1, warning = 2, parse/read or internal infrastructure failure = 3. |
| `--metadata-json <path>` | Write a separate checker/version/input manifest while preserving the diagnostic JSON array. FNV-1a fingerprints detect changes; they are not cryptographic authentication and exclude image assets. |
| `--native-origin` | Compare diagnostic source lines with vanilla files in the same category. A matching line is provenance evidence, not proof that a whole definition is identical or correct. |
| `--audit-gui-state` | Flag selected GUI state commands/effects for runtime review. It cannot verify engine scheduling or prove that repeat loops are absent. |
| `--government-policy <path>` | Apply optional project expectations to named governments. It does not simulate engine government selection. |

A government policy file uses the following shape; replace the example ID with a government defined by your loaded inputs:

```json
{
  "governments": {
    "my_custom_government": "player_only"
  }
}
```

Accepted policy values are `open`, `gated`, and `player_only`. The latter two warn if `can_get_government` is absent; this check does not prove that an existing gate implements the intended policy.

## Validation status and known limits

- Current Windows checks passed **13 library unit tests and 2 shared CLI tests**. Most game-version regression evidence remains local; this is not a claim that every adaptation rule has a portable automated test.
- The shared metadata implementation currently causes **Victoria 3 and Imperator compilation failures**. This source tree should not be treated as a validated multi-game replacement for upstream Tiger.
- Full vanilla semantic coverage, all-mod compatibility, engine acceptance, old-save behavior, AI behavior, and GUI scheduling have not been established. False positives and incomplete checks remain possible.
- A clean report only means the implemented checks found no visible problems under the selected configuration. Retained warnings should be reviewed, not hidden to imply compatibility.
- Rule tables target CK3 1.20.1. Later game versions require renewed source comparison and testing; a version-string change alone is not an adaptation.

To run the source-level tests:

```powershell
cargo test -p tiger-lib --lib --locked
cargo test -p tiger-bin-shared --lib --locked
```

The repository includes source, user-facing instructions, and upstream contributor/license records. Machine-specific handoffs, design documents, verification snapshots, game files, build caches, and local release archives are excluded.

## License

[GPL-3.0-or-later](LICENSE). The original Tiger authors and contributors retain credit for their work.


## Related project

[Tiger Stellar](https://github.com/gura8390/Tiger-Stellar) is a separately maintained Stellaris checker with its own rules and validation limits. Its game semantics are not part of this CK3 adaptation.
