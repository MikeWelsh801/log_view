use crate::Filter;

pub(crate) enum Message {
    MoveUp,
    MoveTop,
    MoveBottom,
    AddChar(char),
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    MoveDown,
    ToggleSearch,
    ApplyFilter(Filter),
    Quit,
}
