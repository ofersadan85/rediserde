#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RespDataKind {
    SimpleString,
    SimpleError,
    Integer,
    BulkString,
    Array,
    Null,
    Boolean,
    Float,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Attributes,
    Set,
    Push,
}

impl RespDataKind {
    pub(crate) fn to_prefix_char(self) -> char {
        match self {
            Self::SimpleString => '+',
            Self::SimpleError => '-',
            Self::Integer => ':',
            Self::BulkString => '$',
            Self::Array => '*',
            Self::Null => '_',
            Self::Boolean => '#',
            Self::Float => ',',
            Self::BigNumber => '(',
            Self::BulkError => '!',
            Self::VerbatimString => '=',
            Self::Map => '%',
            Self::Attributes => '|',
            Self::Set => '~',
            Self::Push => '>',
        }
    }

    pub(crate) fn to_prefix_bytes(self) -> u8 {
        u8::try_from(self.to_prefix_char()).expect("All prefixes are known ASCII characters")
    }

    fn from_prefix_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::SimpleString),
            '-' => Some(Self::SimpleError),
            ':' => Some(Self::Integer),
            '$' => Some(Self::BulkString),
            '*' => Some(Self::Array),
            '_' => Some(Self::Null),
            '#' => Some(Self::Boolean),
            ',' => Some(Self::Float),
            '(' => Some(Self::BigNumber),
            '!' => Some(Self::BulkError),
            '=' => Some(Self::VerbatimString),
            '%' => Some(Self::Map),
            '|' => Some(Self::Attributes),
            '~' => Some(Self::Set),
            '>' => Some(Self::Push),
            _ => None,
        }
    }

    fn from_prefix_bytes(b: u8) -> Option<Self> {
        Self::from_prefix_char(char::from(b))
    }
}

impl From<RespDataKind> for u8 {
    fn from(kind: RespDataKind) -> Self {
        kind.to_prefix_bytes()
    }
}

impl From<RespDataKind> for char {
    fn from(kind: RespDataKind) -> Self {
        kind.to_prefix_char()
    }
}

impl TryFrom<char> for RespDataKind {
    type Error = ();

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Self::from_prefix_char(value).ok_or(())
    }
}

impl TryFrom<u8> for RespDataKind {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Self::from_prefix_bytes(value).ok_or(())
    }
}

impl std::fmt::Display for RespDataKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_prefix_char())
    }
}
