use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use lazy_static::lazy_static;
use crate::cfg::Cfg;
use crate::err::ProcessingResult;
use crate::proc::MatchAction::*;
use crate::proc::MatchMode::*;
use crate::proc::Processor;
#[cfg(feature = "js-esbuild")]
use {
    std::sync::Arc,
    minify::json::minify,
    crate::proc::EsbuildSection,
    crate::proc::checkpoint::WriteCheckpoint,
};

lazy_static! {
    static ref SCRIPT_END: AhoCorasick = AhoCorasickBuilder::new().ascii_case_insensitive(true).build(&["</script"]);
}

#[inline(always)]
pub fn process_json(proc: &mut Processor, cfg: &Cfg) -> ProcessingResult<()> {
    #[cfg(feature = "js-esbuild")]
    let start = WriteCheckpoint::new(proc);
    proc.require_not_at_end()?;
    proc.m(WhileNotSeq(&SCRIPT_END), Keep);
    // `process_tag` will require closing tag.

    // TODO This is copied from style.rs.
    #[cfg(feature = "js-esbuild")]
    if cfg.minify_js {
        let src = start.written_range(proc);
        let (wg, results) = proc.new_esbuild_section();
        let raw_json = unsafe { String::from_utf8_unchecked(proc[src].to_vec()) };
        let result = minify(&raw_json[..]);
        let mut guard = results.lock().unwrap();
        guard.push(EsbuildSection {
            src,
            escaped: result.as_bytes().to_vec(),
        });
        drop(guard);
        drop(results);
        drop(wg);
    };

    Ok(())
}
