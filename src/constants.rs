// constants

// controls
pub const LEFT: char             = 'e';
pub const DOWN: char             = 'i';
pub const UP: char               = 'o';
pub const RIGHT: char            = 'n';
pub const URL: char              = 'g';

// config
pub const INIT_LINK: &str        = "gemini://geminiprotocol.net/";

// protocol
pub const GOPHER_SCHEME: &str    = "gopher";
pub const HTTPS_SCHEME: &str     = "https";
pub const HTTP_SCHEME: &str      = "http";
pub const LINK_SYMBOL: &str      = "=>";
pub const TOGGLE_SYMBOL: &str    = "```";
pub const QUOTE_SYMBOL: &str     = ">";
pub const LIST_ITEM_SYMBOL: &str = "*";
pub const HEADING_1_SYMBOL: &str = "#";
pub const HEADING_2_SYMBOL: &str = "##";
pub const HEADING_3_SYMBOL: &str = "###";
pub const GEMINI_PORT: &str      = "1965";
pub const GEMINI_SCHEME: &str    = "gemini";
pub const STATUS_REGEX: &str     = r"^(\d{1,3})[ \t](.*)\r\n$";
pub const LINK_REGEX: &str       = r"^\s*(\S*)\s*(.*)?$";

