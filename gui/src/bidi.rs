//! Bidirectional text reordering for egui.
//!
//! epaint 0.36 shapes text with harfrust (HarfBuzz), which handles Arabic
//! joining and reverses a run it guesses to be RTL, but it does not implement
//! the Unicode Bidirectional Algorithm — see the `TODO(emilk): heed bidi
//! characters` in `epaint::text::font` and the note in `text_layout` that
//! segmentation is by font face rather than by script.
//!
//! Shaping happens independently per [`LayoutSection`], so we do the one job
//! epaint leaves out: split the text into bidi level runs, emit each as its own
//! section, and order the sections visually. harfrust then gets each run with a
//! single resolved direction and does the joining and intra-run reversal.

use eframe::egui::text::{ByteIndex, LayoutJob, LayoutSection, TextFormat};
use unicode_bidi::ParagraphBidiInfo;

/// Append `input` to `job` as one section per bidi level run, in visual order.
pub fn append_bidi(job: &mut LayoutJob, input: &str, format: &TextFormat) {
    // Newlines are handled here rather than by `ParagraphBidiInfo` because rule
    // L1 resets a paragraph separator to the paragraph level, which would
    // reorder a trailing '\n' to the front of an RTL line.
    for (i, line) in input.split('\n').enumerate() {
        if i > 0 {
            push_section(job, "\n", format);
        }
        if line.is_empty() {
            continue;
        }
        let bidi = ParagraphBidiInfo::new(line, None);
        let (_levels, runs) = bidi.visual_runs(0..line.len());
        for run in runs {
            push_section(job, &line[run], format);
        }
    }
}

/// Lay out `text` on its own, with every run in the same `format`.
pub fn bidi_job(text: &str, format: TextFormat) -> LayoutJob {
    let mut job = LayoutJob::default();
    append_bidi(&mut job, text, &format);
    job
}

/// Push `text` as a section of its own.
///
/// [`LayoutJob::append`] merges into the previous section when the format
/// matches, which would put two runs back into a single shaping buffer, so the
/// section has to be pushed by hand.
fn push_section(job: &mut LayoutJob, text: &str, format: &TextFormat) {
    let start = ByteIndex(job.text.len());
    job.text.push_str(text);
    job.sections.push(LayoutSection {
        leading_space: 0.0,
        byte_range: start..ByteIndex(job.text.len()),
        format: format.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sections(job: &LayoutJob) -> Vec<&str> {
        job.sections
            .iter()
            .map(|s| &job.text[s.byte_range.start.0..s.byte_range.end.0])
            .collect()
    }

    #[test]
    fn digits_keep_their_order_in_an_rtl_line() {
        let job = bidi_job("الجوال: 0590675578", TextFormat::default());
        // The number is a separate run, still in logical order, and placed to
        // the left of the Arabic.
        assert_eq!(sections(&job), ["0590675578", "الجوال: "]);
    }

    #[test]
    fn a_pure_rtl_line_is_a_single_run() {
        let job = bidi_job("محمد الشجرة", TextFormat::default());
        assert_eq!(sections(&job), ["محمد الشجرة"]);
    }

    #[test]
    fn a_pure_ltr_line_is_untouched() {
        let job = bidi_job("John Smith 42", TextFormat::default());
        assert_eq!(sections(&job), ["John Smith 42"]);
    }

    #[test]
    fn newlines_stay_between_their_lines() {
        let job = bidi_job("محمد\nJohn", TextFormat::default());
        assert_eq!(sections(&job), ["محمد", "\n", "John"]);
    }

    #[test]
    fn empty_input_produces_no_sections() {
        let job = bidi_job("", TextFormat::default());
        assert!(job.sections.is_empty());
        assert!(job.text.is_empty());
    }
}
