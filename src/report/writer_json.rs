use std::io::Write;

use serde_json::json;

use crate::fileset::FileStage;
use crate::report::errors::Errors;
use crate::report::writer::kind_tag;
use crate::report::{LogReportMetadata, LogReportPointers};

/// Log the report in JSON format.
pub fn log_report_json<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    report: &LogReportMetadata,
    pointers: &LogReportPointers,
) {
    let pointers: Vec<_> = pointers
        .iter()
        .map(|pointer| {
            let path = pointer.loc.pathname();
            let fullpath = pointer.loc.fullpath();
            json!({
                "path": path,
                "from": kind_tag(errors, pointer.loc.kind),
                "stage": stage_desc(pointer.loc.stage),
                "fullpath": fullpath,
                "linenr": if pointer.loc.line == 0 { None } else { Some(pointer.loc.line) },
                "column": if pointer.loc.column == 0 { None } else { Some(pointer.loc.column) },
                "length": if pointer.length == 0 { None } else { Some(pointer.length) },
                "line": errors.cache.get_line(pointer.loc),
                "tag": pointer.msg,
                "origin": source_origin(errors, pointer.loc, errors.cache.get_line(pointer.loc)),
            })
        })
        .collect();
    let report = json!({
        "severity": report.severity,
        "confidence": report.confidence,
        "key": report.key,
        "message": &report.msg,
        "info": &report.info,
        "wiki": &report.wiki,
        "locations": pointers,
    });

    if let Err(e) = serde_json::to_writer_pretty(output, &report) {
        eprintln!("JSON error: {e:#}");
    }
}

fn stage_desc(stage: FileStage) -> Option<&'static str> {
    match stage {
        #[cfg(feature = "eu5")]
        FileStage::LoadingScreen => Some("loading_screen"),
        #[cfg(feature = "eu5")]
        FileStage::MainMenu => Some("main_menu"),
        #[cfg(feature = "eu5")]
        FileStage::InGame => Some("in_game"),
        FileStage::NoStage => None,
    }
}

fn source_origin(errors: &Errors, loc: crate::token::Loc, line: Option<&str>) -> serde_json::Value {
    if loc.kind.counts_as_vanilla() { return json!({"classification":"native","comparison":"file layer"}); }
    let Some(root)=&errors.native_origin_root else { return serde_json::Value::Null; };
    let path=loc.pathname();
    let Some(parent)=path.parent() else { return json!({"classification":"unresolved"}); };
    let target=line.unwrap_or("").trim();
    if target.is_empty() { return json!({"classification":"unresolved"}); }
    let mut matches=Vec::new();
    if let Ok(entries)=std::fs::read_dir(root.join(parent)) {
        for entry in entries.flatten() {
            let candidate=entry.path();
            if !candidate.is_file() || candidate.extension()!=path.extension() { continue; }
            if let Ok(text)=std::fs::read_to_string(&candidate) {
                for (idx,native_line) in text.lines().enumerate() {
                    if native_line.trim()==target { matches.push(json!({"path":candidate,"linenr":idx+1})); }
                }
            }
        }
    }
    json!({"classification":if matches.is_empty() {"no_same_native_line_in_category"} else {"same_native_line"},
        "comparison":"exact trimmed source line within the corresponding native directory; not proof of identical definition or engine correctness",
        "layer":kind_tag(errors,loc.kind),"native_locations":matches})
}
