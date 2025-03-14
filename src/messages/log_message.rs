use crate::Filter;

pub(crate) enum Message {
    MoveUp,
    MoveDown,
    ToggleSearch,
    ApplyFilter(Filter),
    Quit,
}
