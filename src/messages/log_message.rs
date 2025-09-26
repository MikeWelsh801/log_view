use crate::Filter;

pub(crate) enum Message {
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    AddChar(char),
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    MoveUpPage,
    MoveDownPage,
    ToggleSearch,
    ApplyFilter(Filter),
    Quit,
}
