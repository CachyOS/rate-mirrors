use std::fmt::Debug;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
pub struct StdinTarget {
    /// Path to be joined to a mirror url and used for speed testing
    #[structopt(long = "path-to-test", default_value = "", verbatim_doc_comment)]
    pub path_to_test: String,

    /// Path to be joined to a mirror url before returning results
    #[structopt(long = "path-to-return", default_value = "", verbatim_doc_comment)]
    pub path_to_return: String,

    /// comment prefix to use when printing debug info
    #[structopt(long = "comment-prefix", default_value = "# ")]
    pub comment_prefix: String,

    /// output prefix to use when printing results
    #[structopt(long = "output-prefix", default_value = "")]
    pub output_prefix: String,

    /// input separator to use when parsing mirrors list
    #[structopt(long = "separator", default_value = "\t")]
    pub input_separator: String,
}
