pub struct RedisType;

/// RESP data type
impl RedisType {
    pub const BOOLEAN: char = '#';

    pub const INTEGER: char = ':';
    pub const BIG_NUMBER: char = '(';
    pub const DOUBLE: char = ',';

    pub const SIMPLE_STRING: char = '+';
    pub const BULK_STRING: char = '$';
    pub const VERBATIM_STRING: char = '=';

    pub const SIMPLE_ERROR: char = '-';
    pub const BULK_ERROR: char = '!';

    pub const NULL: char = '_';

    pub const ARRAY: char = '*';
    pub const MAP: char = '%';
    pub const SET: char = '~';
    pub const PUSH: char = '>';

    pub const CRLF: &'static str = "\r\n";
}
