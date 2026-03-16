#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MessageKind {
    Success,
    Warning,
}

pub const SORT_ARROW_UP: &str = " \u{25b2}";
pub const SORT_ARROW_DOWN: &str = " \u{25bc}";
