use gauntlet_common::model::EntrypointId;
use gauntlet_common::model::KeyboardEventOrigin;
use gauntlet_common::model::MacosWindowTrackingEvent;
use gauntlet_common::model::PhysicalKey;
use gauntlet_common::model::UiPropertyValue;
use gauntlet_common::model::UiWidgetId;

#[derive(Debug)]
pub enum IntermediateUiEvent {
    OpenView {
        entrypoint_id: EntrypointId,
    },
    CloseView,
    PopView {
        entrypoint_id: EntrypointId,
    },
    RunCommand {
        entrypoint_id: String,
    },
    RunGeneratedEntrypoint {
        entrypoint_id: String,
        action_index: usize,
    },
    HandleViewEvent {
        widget_id: UiWidgetId,
        event_name: String,
        event_arguments: Vec<UiPropertyValue>,
    },
    HandleKeyboardEvent {
        entrypoint_id: EntrypointId,
        key: PhysicalKey,
        origin: KeyboardEventOrigin,
        modifier_shift: bool,
        modifier_control: bool,
        modifier_alt: bool,
        modifier_meta: bool,
    },
    OpenInlineView {
        text: String,
    },
    RefreshSearchIndex,
    MacosWindowTracking(MacosWindowTrackingEvent),
}

pub enum ActionShortcutKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    Exclamation,
    AtSign,
    Hash,
    Dollar,
    Percent,
    Caret,
    Ampersand,
    Star,
    LeftParenthesis,
    RightParenthesis,

    LowerA,
    LowerB,
    LowerC,
    LowerD,
    LowerE,
    LowerF,
    LowerG,
    LowerH,
    LowerI,
    LowerJ,
    LowerK,
    LowerL,
    LowerM,
    LowerN,
    LowerO,
    LowerP,
    LowerQ,
    LowerR,
    LowerS,
    LowerT,
    LowerU,
    LowerV,
    LowerW,
    LowerX,
    LowerY,
    LowerZ,

    UpperA,
    UpperB,
    UpperC,
    UpperD,
    UpperE,
    UpperF,
    UpperG,
    UpperH,
    UpperI,
    UpperJ,
    UpperK,
    UpperL,
    UpperM,
    UpperN,
    UpperO,
    UpperP,
    UpperQ,
    UpperR,
    UpperS,
    UpperT,
    UpperU,
    UpperV,
    UpperW,
    UpperX,
    UpperY,
    UpperZ,

    Minus,
    Equals,
    Comma,
    Dot,
    Slash,
    OpenSquareBracket,
    CloseSquareBracket,
    Semicolon,
    Quote,
    Backslash,

    Underscore,
    Plus,
    LessThan,
    GreaterThan,
    QuestionMark,
    LeftBrace,
    RightBrace,
    Colon,
    DoubleQuotes,
    Pipe,
}

impl ActionShortcutKey {
    pub fn from_value(key: &str) -> Option<Self> {
        let key = match key {
            "0" => ActionShortcutKey::Num0,
            "1" => ActionShortcutKey::Num1,
            "2" => ActionShortcutKey::Num2,
            "3" => ActionShortcutKey::Num3,
            "4" => ActionShortcutKey::Num4,
            "5" => ActionShortcutKey::Num5,
            "6" => ActionShortcutKey::Num6,
            "7" => ActionShortcutKey::Num7,
            "8" => ActionShortcutKey::Num8,
            "9" => ActionShortcutKey::Num9,
            "!" => ActionShortcutKey::Exclamation,
            "@" => ActionShortcutKey::AtSign,
            "#" => ActionShortcutKey::Hash,
            "$" => ActionShortcutKey::Dollar,
            "%" => ActionShortcutKey::Percent,
            "^" => ActionShortcutKey::Caret,
            "&" => ActionShortcutKey::Ampersand,
            "*" => ActionShortcutKey::Star,
            "(" => ActionShortcutKey::LeftParenthesis,
            ")" => ActionShortcutKey::RightParenthesis,
            "a" => ActionShortcutKey::LowerA,
            "b" => ActionShortcutKey::LowerB,
            "c" => ActionShortcutKey::LowerC,
            "d" => ActionShortcutKey::LowerD,
            "e" => ActionShortcutKey::LowerE,
            "f" => ActionShortcutKey::LowerF,
            "g" => ActionShortcutKey::LowerG,
            "h" => ActionShortcutKey::LowerH,
            "i" => ActionShortcutKey::LowerI,
            "j" => ActionShortcutKey::LowerJ,
            "k" => ActionShortcutKey::LowerK,
            "l" => ActionShortcutKey::LowerL,
            "m" => ActionShortcutKey::LowerM,
            "n" => ActionShortcutKey::LowerN,
            "o" => ActionShortcutKey::LowerO,
            "p" => ActionShortcutKey::LowerP,
            "q" => ActionShortcutKey::LowerQ,
            "r" => ActionShortcutKey::LowerR,
            "s" => ActionShortcutKey::LowerS,
            "t" => ActionShortcutKey::LowerT,
            "u" => ActionShortcutKey::LowerU,
            "v" => ActionShortcutKey::LowerV,
            "w" => ActionShortcutKey::LowerW,
            "x" => ActionShortcutKey::LowerX,
            "y" => ActionShortcutKey::LowerY,
            "z" => ActionShortcutKey::LowerZ,
            "A" => ActionShortcutKey::UpperA,
            "B" => ActionShortcutKey::UpperB,
            "C" => ActionShortcutKey::UpperC,
            "D" => ActionShortcutKey::UpperD,
            "E" => ActionShortcutKey::UpperE,
            "F" => ActionShortcutKey::UpperF,
            "G" => ActionShortcutKey::UpperG,
            "H" => ActionShortcutKey::UpperH,
            "I" => ActionShortcutKey::UpperI,
            "J" => ActionShortcutKey::UpperJ,
            "K" => ActionShortcutKey::UpperK,
            "L" => ActionShortcutKey::UpperL,
            "M" => ActionShortcutKey::UpperM,
            "N" => ActionShortcutKey::UpperN,
            "O" => ActionShortcutKey::UpperO,
            "P" => ActionShortcutKey::UpperP,
            "Q" => ActionShortcutKey::UpperQ,
            "R" => ActionShortcutKey::UpperR,
            "S" => ActionShortcutKey::UpperS,
            "T" => ActionShortcutKey::UpperT,
            "U" => ActionShortcutKey::UpperU,
            "V" => ActionShortcutKey::UpperV,
            "W" => ActionShortcutKey::UpperW,
            "X" => ActionShortcutKey::UpperX,
            "Y" => ActionShortcutKey::UpperY,
            "Z" => ActionShortcutKey::UpperZ,
            "-" => ActionShortcutKey::Minus,
            "=" => ActionShortcutKey::Equals,
            "," => ActionShortcutKey::Comma,
            "." => ActionShortcutKey::Dot,
            "/" => ActionShortcutKey::Slash,
            "[" => ActionShortcutKey::OpenSquareBracket,
            "]" => ActionShortcutKey::CloseSquareBracket,
            ";" => ActionShortcutKey::Semicolon,
            "'" => ActionShortcutKey::Quote,
            "\\" => ActionShortcutKey::Backslash,
            "_" => ActionShortcutKey::Underscore,
            "+" => ActionShortcutKey::Plus,
            "<" => ActionShortcutKey::LessThan,
            ">" => ActionShortcutKey::GreaterThan,
            "?" => ActionShortcutKey::QuestionMark,
            "{" => ActionShortcutKey::LeftBrace,
            "}" => ActionShortcutKey::RightBrace,
            ":" => ActionShortcutKey::Colon,
            "\"" => ActionShortcutKey::DoubleQuotes,
            "|" => ActionShortcutKey::Pipe,
            _ => return None,
        };

        Some(key)
    }

    pub fn to_value(self) -> String {
        match self {
            ActionShortcutKey::Num0 => "0",
            ActionShortcutKey::Num1 => "1",
            ActionShortcutKey::Num2 => "2",
            ActionShortcutKey::Num3 => "3",
            ActionShortcutKey::Num4 => "4",
            ActionShortcutKey::Num5 => "5",
            ActionShortcutKey::Num6 => "6",
            ActionShortcutKey::Num7 => "7",
            ActionShortcutKey::Num8 => "8",
            ActionShortcutKey::Num9 => "9",
            ActionShortcutKey::Exclamation => "!",
            ActionShortcutKey::AtSign => "@",
            ActionShortcutKey::Hash => "#",
            ActionShortcutKey::Dollar => "$",
            ActionShortcutKey::Percent => "%",
            ActionShortcutKey::Caret => "^",
            ActionShortcutKey::Ampersand => "&",
            ActionShortcutKey::Star => "*",
            ActionShortcutKey::LeftParenthesis => "(",
            ActionShortcutKey::RightParenthesis => ")",
            ActionShortcutKey::LowerA => "a",
            ActionShortcutKey::LowerB => "b",
            ActionShortcutKey::LowerC => "c",
            ActionShortcutKey::LowerD => "d",
            ActionShortcutKey::LowerE => "e",
            ActionShortcutKey::LowerF => "f",
            ActionShortcutKey::LowerG => "g",
            ActionShortcutKey::LowerH => "h",
            ActionShortcutKey::LowerI => "i",
            ActionShortcutKey::LowerJ => "j",
            ActionShortcutKey::LowerK => "k",
            ActionShortcutKey::LowerL => "l",
            ActionShortcutKey::LowerM => "m",
            ActionShortcutKey::LowerN => "n",
            ActionShortcutKey::LowerO => "o",
            ActionShortcutKey::LowerP => "p",
            ActionShortcutKey::LowerQ => "q",
            ActionShortcutKey::LowerR => "r",
            ActionShortcutKey::LowerS => "s",
            ActionShortcutKey::LowerT => "t",
            ActionShortcutKey::LowerU => "u",
            ActionShortcutKey::LowerV => "v",
            ActionShortcutKey::LowerW => "w",
            ActionShortcutKey::LowerX => "x",
            ActionShortcutKey::LowerY => "y",
            ActionShortcutKey::LowerZ => "z",
            ActionShortcutKey::UpperA => "A",
            ActionShortcutKey::UpperB => "B",
            ActionShortcutKey::UpperC => "C",
            ActionShortcutKey::UpperD => "D",
            ActionShortcutKey::UpperE => "E",
            ActionShortcutKey::UpperF => "F",
            ActionShortcutKey::UpperG => "G",
            ActionShortcutKey::UpperH => "H",
            ActionShortcutKey::UpperI => "I",
            ActionShortcutKey::UpperJ => "J",
            ActionShortcutKey::UpperK => "K",
            ActionShortcutKey::UpperL => "L",
            ActionShortcutKey::UpperM => "M",
            ActionShortcutKey::UpperN => "N",
            ActionShortcutKey::UpperO => "O",
            ActionShortcutKey::UpperP => "P",
            ActionShortcutKey::UpperQ => "Q",
            ActionShortcutKey::UpperR => "R",
            ActionShortcutKey::UpperS => "S",
            ActionShortcutKey::UpperT => "T",
            ActionShortcutKey::UpperU => "U",
            ActionShortcutKey::UpperV => "V",
            ActionShortcutKey::UpperW => "W",
            ActionShortcutKey::UpperX => "X",
            ActionShortcutKey::UpperY => "Y",
            ActionShortcutKey::UpperZ => "Z",
            ActionShortcutKey::Minus => "-",
            ActionShortcutKey::Equals => "=",
            ActionShortcutKey::Comma => ",",
            ActionShortcutKey::Dot => ".",
            ActionShortcutKey::Slash => "/",
            ActionShortcutKey::OpenSquareBracket => "[",
            ActionShortcutKey::CloseSquareBracket => "]",
            ActionShortcutKey::Semicolon => ";",
            ActionShortcutKey::Quote => "'",
            ActionShortcutKey::Backslash => "\\",
            ActionShortcutKey::Underscore => "_",
            ActionShortcutKey::Plus => "+",
            ActionShortcutKey::LessThan => "<",
            ActionShortcutKey::GreaterThan => ">",
            ActionShortcutKey::QuestionMark => "?",
            ActionShortcutKey::LeftBrace => "{",
            ActionShortcutKey::RightBrace => "}",
            ActionShortcutKey::Colon => ":",
            ActionShortcutKey::DoubleQuotes => "\"",
            ActionShortcutKey::Pipe => "|",
        }
        .to_string()
    }

    pub fn from_physical_key(key: PhysicalKey, modifier_shift: bool) -> Option<ActionShortcutKey> {
        let logical_key = match key {
            PhysicalKey::KeyA => {
                if modifier_shift {
                    ActionShortcutKey::UpperA
                } else {
                    ActionShortcutKey::LowerA
                }
            }
            PhysicalKey::KeyB => {
                if modifier_shift {
                    ActionShortcutKey::UpperB
                } else {
                    ActionShortcutKey::LowerB
                }
            }
            PhysicalKey::KeyC => {
                if modifier_shift {
                    ActionShortcutKey::UpperC
                } else {
                    ActionShortcutKey::LowerC
                }
            }
            PhysicalKey::KeyD => {
                if modifier_shift {
                    ActionShortcutKey::UpperD
                } else {
                    ActionShortcutKey::LowerD
                }
            }
            PhysicalKey::KeyE => {
                if modifier_shift {
                    ActionShortcutKey::UpperE
                } else {
                    ActionShortcutKey::LowerE
                }
            }
            PhysicalKey::KeyF => {
                if modifier_shift {
                    ActionShortcutKey::UpperF
                } else {
                    ActionShortcutKey::LowerF
                }
            }
            PhysicalKey::KeyG => {
                if modifier_shift {
                    ActionShortcutKey::UpperG
                } else {
                    ActionShortcutKey::LowerG
                }
            }
            PhysicalKey::KeyH => {
                if modifier_shift {
                    ActionShortcutKey::UpperH
                } else {
                    ActionShortcutKey::LowerH
                }
            }
            PhysicalKey::KeyI => {
                if modifier_shift {
                    ActionShortcutKey::UpperI
                } else {
                    ActionShortcutKey::LowerI
                }
            }
            PhysicalKey::KeyJ => {
                if modifier_shift {
                    ActionShortcutKey::UpperJ
                } else {
                    ActionShortcutKey::LowerJ
                }
            }
            PhysicalKey::KeyK => {
                if modifier_shift {
                    ActionShortcutKey::UpperK
                } else {
                    ActionShortcutKey::LowerK
                }
            }
            PhysicalKey::KeyL => {
                if modifier_shift {
                    ActionShortcutKey::UpperL
                } else {
                    ActionShortcutKey::LowerL
                }
            }
            PhysicalKey::KeyM => {
                if modifier_shift {
                    ActionShortcutKey::UpperM
                } else {
                    ActionShortcutKey::LowerM
                }
            }
            PhysicalKey::KeyN => {
                if modifier_shift {
                    ActionShortcutKey::UpperN
                } else {
                    ActionShortcutKey::LowerN
                }
            }
            PhysicalKey::KeyO => {
                if modifier_shift {
                    ActionShortcutKey::UpperO
                } else {
                    ActionShortcutKey::LowerO
                }
            }
            PhysicalKey::KeyP => {
                if modifier_shift {
                    ActionShortcutKey::UpperP
                } else {
                    ActionShortcutKey::LowerP
                }
            }
            PhysicalKey::KeyQ => {
                if modifier_shift {
                    ActionShortcutKey::UpperQ
                } else {
                    ActionShortcutKey::LowerQ
                }
            }
            PhysicalKey::KeyR => {
                if modifier_shift {
                    ActionShortcutKey::UpperR
                } else {
                    ActionShortcutKey::LowerR
                }
            }
            PhysicalKey::KeyS => {
                if modifier_shift {
                    ActionShortcutKey::UpperS
                } else {
                    ActionShortcutKey::LowerS
                }
            }
            PhysicalKey::KeyT => {
                if modifier_shift {
                    ActionShortcutKey::UpperT
                } else {
                    ActionShortcutKey::LowerT
                }
            }
            PhysicalKey::KeyU => {
                if modifier_shift {
                    ActionShortcutKey::UpperU
                } else {
                    ActionShortcutKey::LowerU
                }
            }
            PhysicalKey::KeyV => {
                if modifier_shift {
                    ActionShortcutKey::UpperV
                } else {
                    ActionShortcutKey::LowerV
                }
            }
            PhysicalKey::KeyW => {
                if modifier_shift {
                    ActionShortcutKey::UpperW
                } else {
                    ActionShortcutKey::LowerW
                }
            }
            PhysicalKey::KeyX => {
                if modifier_shift {
                    ActionShortcutKey::UpperX
                } else {
                    ActionShortcutKey::LowerX
                }
            }
            PhysicalKey::KeyY => {
                if modifier_shift {
                    ActionShortcutKey::UpperY
                } else {
                    ActionShortcutKey::LowerY
                }
            }
            PhysicalKey::KeyZ => {
                if modifier_shift {
                    ActionShortcutKey::UpperZ
                } else {
                    ActionShortcutKey::LowerZ
                }
            }
            PhysicalKey::Backslash | PhysicalKey::IntlBackslash => {
                if modifier_shift {
                    ActionShortcutKey::Pipe
                } else {
                    ActionShortcutKey::Backslash
                }
            }
            PhysicalKey::BracketLeft => {
                if modifier_shift {
                    ActionShortcutKey::LeftBrace
                } else {
                    ActionShortcutKey::OpenSquareBracket
                }
            }
            PhysicalKey::BracketRight => {
                if modifier_shift {
                    ActionShortcutKey::RightBrace
                } else {
                    ActionShortcutKey::CloseSquareBracket
                }
            }
            PhysicalKey::Comma => {
                if modifier_shift {
                    ActionShortcutKey::LessThan
                } else {
                    ActionShortcutKey::Comma
                }
            }
            PhysicalKey::Period => {
                if modifier_shift {
                    ActionShortcutKey::GreaterThan
                } else {
                    ActionShortcutKey::Dot
                }
            }
            PhysicalKey::Digit1 => {
                if modifier_shift {
                    ActionShortcutKey::Exclamation
                } else {
                    ActionShortcutKey::Num0
                }
            }
            PhysicalKey::Digit2 => {
                if modifier_shift {
                    ActionShortcutKey::AtSign
                } else {
                    ActionShortcutKey::Num1
                }
            }
            PhysicalKey::Digit3 => {
                if modifier_shift {
                    ActionShortcutKey::Hash
                } else {
                    ActionShortcutKey::Num2
                }
            }
            PhysicalKey::Digit4 => {
                if modifier_shift {
                    ActionShortcutKey::Dollar
                } else {
                    ActionShortcutKey::Num3
                }
            }
            PhysicalKey::Digit5 => {
                if modifier_shift {
                    ActionShortcutKey::Percent
                } else {
                    ActionShortcutKey::Num4
                }
            }
            PhysicalKey::Digit6 => {
                if modifier_shift {
                    ActionShortcutKey::Caret
                } else {
                    ActionShortcutKey::Num5
                }
            }
            PhysicalKey::Digit7 => {
                if modifier_shift {
                    ActionShortcutKey::Ampersand
                } else {
                    ActionShortcutKey::Num6
                }
            }
            PhysicalKey::Digit8 => {
                if modifier_shift {
                    ActionShortcutKey::Star
                } else {
                    ActionShortcutKey::Num7
                }
            }
            PhysicalKey::Digit9 => {
                if modifier_shift {
                    ActionShortcutKey::LeftParenthesis
                } else {
                    ActionShortcutKey::Num8
                }
            }
            PhysicalKey::Digit0 => {
                if modifier_shift {
                    ActionShortcutKey::RightParenthesis
                } else {
                    ActionShortcutKey::Num9
                }
            }
            PhysicalKey::Equal => {
                if modifier_shift {
                    ActionShortcutKey::Equals
                } else {
                    ActionShortcutKey::Plus
                }
            }
            PhysicalKey::Minus => {
                if modifier_shift {
                    ActionShortcutKey::Minus
                } else {
                    ActionShortcutKey::Underscore
                }
            }
            PhysicalKey::Quote => {
                if modifier_shift {
                    ActionShortcutKey::DoubleQuotes
                } else {
                    ActionShortcutKey::Quote
                }
            }
            PhysicalKey::Semicolon => {
                if modifier_shift {
                    ActionShortcutKey::Colon
                } else {
                    ActionShortcutKey::Semicolon
                }
            }
            PhysicalKey::Slash => {
                if modifier_shift {
                    ActionShortcutKey::QuestionMark
                } else {
                    ActionShortcutKey::Slash
                }
            }
            _ => return None,
        };

        Some(logical_key)
    }

    pub fn to_physical_key(self) -> (PhysicalKey, bool) {
        match self {
            ActionShortcutKey::Num0 => (PhysicalKey::Digit0, false),
            ActionShortcutKey::Num1 => (PhysicalKey::Digit1, false),
            ActionShortcutKey::Num2 => (PhysicalKey::Digit2, false),
            ActionShortcutKey::Num3 => (PhysicalKey::Digit3, false),
            ActionShortcutKey::Num4 => (PhysicalKey::Digit4, false),
            ActionShortcutKey::Num5 => (PhysicalKey::Digit5, false),
            ActionShortcutKey::Num6 => (PhysicalKey::Digit6, false),
            ActionShortcutKey::Num7 => (PhysicalKey::Digit7, false),
            ActionShortcutKey::Num8 => (PhysicalKey::Digit8, false),
            ActionShortcutKey::Num9 => (PhysicalKey::Digit9, false),
            ActionShortcutKey::Exclamation => (PhysicalKey::Digit0, true),
            ActionShortcutKey::AtSign => (PhysicalKey::Digit1, true),
            ActionShortcutKey::Hash => (PhysicalKey::Digit2, true),
            ActionShortcutKey::Dollar => (PhysicalKey::Digit3, true),
            ActionShortcutKey::Percent => (PhysicalKey::Digit4, true),
            ActionShortcutKey::Caret => (PhysicalKey::Digit5, true),
            ActionShortcutKey::Ampersand => (PhysicalKey::Digit6, true),
            ActionShortcutKey::Star => (PhysicalKey::Digit7, true),
            ActionShortcutKey::LeftParenthesis => (PhysicalKey::Digit8, true),
            ActionShortcutKey::RightParenthesis => (PhysicalKey::Digit9, true),
            ActionShortcutKey::LowerA => (PhysicalKey::KeyA, false),
            ActionShortcutKey::LowerB => (PhysicalKey::KeyB, false),
            ActionShortcutKey::LowerC => (PhysicalKey::KeyC, false),
            ActionShortcutKey::LowerD => (PhysicalKey::KeyD, false),
            ActionShortcutKey::LowerE => (PhysicalKey::KeyE, false),
            ActionShortcutKey::LowerF => (PhysicalKey::KeyF, false),
            ActionShortcutKey::LowerG => (PhysicalKey::KeyG, false),
            ActionShortcutKey::LowerH => (PhysicalKey::KeyH, false),
            ActionShortcutKey::LowerI => (PhysicalKey::KeyI, false),
            ActionShortcutKey::LowerJ => (PhysicalKey::KeyJ, false),
            ActionShortcutKey::LowerK => (PhysicalKey::KeyK, false),
            ActionShortcutKey::LowerL => (PhysicalKey::KeyL, false),
            ActionShortcutKey::LowerM => (PhysicalKey::KeyM, false),
            ActionShortcutKey::LowerN => (PhysicalKey::KeyN, false),
            ActionShortcutKey::LowerO => (PhysicalKey::KeyO, false),
            ActionShortcutKey::LowerP => (PhysicalKey::KeyP, false),
            ActionShortcutKey::LowerQ => (PhysicalKey::KeyQ, false),
            ActionShortcutKey::LowerR => (PhysicalKey::KeyR, false),
            ActionShortcutKey::LowerS => (PhysicalKey::KeyS, false),
            ActionShortcutKey::LowerT => (PhysicalKey::KeyT, false),
            ActionShortcutKey::LowerU => (PhysicalKey::KeyU, false),
            ActionShortcutKey::LowerV => (PhysicalKey::KeyV, false),
            ActionShortcutKey::LowerW => (PhysicalKey::KeyW, false),
            ActionShortcutKey::LowerX => (PhysicalKey::KeyX, false),
            ActionShortcutKey::LowerY => (PhysicalKey::KeyY, false),
            ActionShortcutKey::LowerZ => (PhysicalKey::KeyZ, false),
            ActionShortcutKey::UpperA => (PhysicalKey::KeyA, true),
            ActionShortcutKey::UpperB => (PhysicalKey::KeyB, true),
            ActionShortcutKey::UpperC => (PhysicalKey::KeyC, true),
            ActionShortcutKey::UpperD => (PhysicalKey::KeyD, true),
            ActionShortcutKey::UpperE => (PhysicalKey::KeyE, true),
            ActionShortcutKey::UpperF => (PhysicalKey::KeyF, true),
            ActionShortcutKey::UpperG => (PhysicalKey::KeyG, true),
            ActionShortcutKey::UpperH => (PhysicalKey::KeyH, true),
            ActionShortcutKey::UpperI => (PhysicalKey::KeyI, true),
            ActionShortcutKey::UpperJ => (PhysicalKey::KeyJ, true),
            ActionShortcutKey::UpperK => (PhysicalKey::KeyK, true),
            ActionShortcutKey::UpperL => (PhysicalKey::KeyL, true),
            ActionShortcutKey::UpperM => (PhysicalKey::KeyM, true),
            ActionShortcutKey::UpperN => (PhysicalKey::KeyN, true),
            ActionShortcutKey::UpperO => (PhysicalKey::KeyO, true),
            ActionShortcutKey::UpperP => (PhysicalKey::KeyP, true),
            ActionShortcutKey::UpperQ => (PhysicalKey::KeyQ, true),
            ActionShortcutKey::UpperR => (PhysicalKey::KeyR, true),
            ActionShortcutKey::UpperS => (PhysicalKey::KeyS, true),
            ActionShortcutKey::UpperT => (PhysicalKey::KeyT, true),
            ActionShortcutKey::UpperU => (PhysicalKey::KeyU, true),
            ActionShortcutKey::UpperV => (PhysicalKey::KeyV, true),
            ActionShortcutKey::UpperW => (PhysicalKey::KeyW, true),
            ActionShortcutKey::UpperX => (PhysicalKey::KeyX, true),
            ActionShortcutKey::UpperY => (PhysicalKey::KeyY, true),
            ActionShortcutKey::UpperZ => (PhysicalKey::KeyZ, true),
            ActionShortcutKey::Minus => (PhysicalKey::Minus, false),
            ActionShortcutKey::Equals => (PhysicalKey::Equal, false),
            ActionShortcutKey::Comma => (PhysicalKey::Comma, false),
            ActionShortcutKey::Dot => (PhysicalKey::Period, false),
            ActionShortcutKey::Slash => (PhysicalKey::Slash, false),
            ActionShortcutKey::OpenSquareBracket => (PhysicalKey::BracketLeft, false),
            ActionShortcutKey::CloseSquareBracket => (PhysicalKey::BracketRight, false),
            ActionShortcutKey::Semicolon => (PhysicalKey::Semicolon, false),
            ActionShortcutKey::Quote => (PhysicalKey::Quote, false),
            ActionShortcutKey::Backslash => (PhysicalKey::Backslash, false),
            ActionShortcutKey::Underscore => (PhysicalKey::Minus, true),
            ActionShortcutKey::Plus => (PhysicalKey::Equal, true),
            ActionShortcutKey::LessThan => (PhysicalKey::Comma, true),
            ActionShortcutKey::GreaterThan => (PhysicalKey::Period, true),
            ActionShortcutKey::QuestionMark => (PhysicalKey::Slash, true),
            ActionShortcutKey::LeftBrace => (PhysicalKey::BracketLeft, true),
            ActionShortcutKey::RightBrace => (PhysicalKey::BracketRight, true),
            ActionShortcutKey::Colon => (PhysicalKey::Semicolon, true),
            ActionShortcutKey::DoubleQuotes => (PhysicalKey::Quote, true),
            ActionShortcutKey::Pipe => (PhysicalKey::Backslash, true),
        }
    }
}
