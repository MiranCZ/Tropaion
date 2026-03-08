pub type Bp = u32;


pub struct BindingPower {
    pub lbp: Bp,
    pub rbp: Bp,
}
impl BindingPower {

    pub const fn right_assoc(bp: Bp) -> Self {
        Self {
            lbp: bp,
            rbp: bp -1,
        }
    }

    pub const fn left_assoc(bp: Bp) -> Self {
        Self {
            lbp: bp-1,
            rbp: bp
        }
    }

    pub const fn non_assoc(bp: Bp) -> Self {
        Self {
            lbp: bp,
            rbp: bp
        }
    }

    pub const fn prefix(bp: Bp) -> Self {
        Self {
            lbp: 0,
            rbp: bp
        }
    }

}

pub const DEFAULT: BindingPower = BindingPower::non_assoc(0);
pub const COMMA: BindingPower = BindingPower::left_assoc(10);
pub const ASSIGNMENT: BindingPower = BindingPower::right_assoc(20);
pub const NULL_DECONSTRUCT: BindingPower = BindingPower::right_assoc(30);
pub const LOGICAL_ADD: BindingPower = BindingPower::left_assoc(40);
pub const LOGICAL_MULT: BindingPower = BindingPower::left_assoc(50);
pub const COMPARING: BindingPower = BindingPower::non_assoc(60);
pub const NUMERIC_ADD: BindingPower = BindingPower::left_assoc(70);
pub const NUMERIC_MULT: BindingPower = BindingPower::left_assoc(80);
pub const UNARY: BindingPower = BindingPower::prefix(90);
pub const MEMBER: BindingPower = BindingPower::left_assoc(100);
pub const CALL: BindingPower = BindingPower::left_assoc(110);
pub const PRIMARY: BindingPower = BindingPower::left_assoc(120);
