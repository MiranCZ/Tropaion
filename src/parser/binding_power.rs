pub type Bp = u32;

pub const DEFAULT: Bp = 0;
pub const COMMA: Bp = 10;
pub const ASSIGNMENT: Bp = 20;
pub const COMPARING: Bp = 30;
pub const LOGICAL_ADD: Bp = 40;
pub const LOGICAL_MULT: Bp = 50;
pub const NUMERIC_ADD: Bp = 60;
pub const NUMERIC_MULT: Bp = 70;
pub const UNARY: Bp = 80;
pub const CALL: Bp = 90;
pub const MEMBER: Bp = 100;
pub const PRIMARY: Bp = 110;
