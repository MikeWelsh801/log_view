use crate::Filter;

pub(crate) enum Message {
    MoveUp,
    AddChar(char),
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    MoveDown,
    ToggleSearch,
    ApplyFilter(Filter),
    RefreshLogs,
    Quit,
}
