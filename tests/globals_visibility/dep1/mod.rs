// intentionally declare child1 as public,
pub mod child1;

// and child 2 as private.
mod child2;

const D1_PRIV: u32 = 10;
pub const D1_PUB: u32 = 10;
pub(crate) const D1_PUB_CRATE: u32 = 10;
pub(self) const D1_PUB_SELF: u32 = 10;
pub(crate) const D1_PUB_IN_CRATE: u32 = 10;

// not necessarily meaningful in this context?
// but will be visible to main file (crate:: level)
pub(super) const D1_PUB_SUPER: u32 = 10;

fn fn_in_dep1() {
    let a = crate::dep1::D1_PRIV;
    let a = crate::dep1::D1_PUB;
    let a = crate::dep1::D1_PUB_CRATE;
    let a = crate::dep1::D1_PUB_IN_CRATE;
    let a = crate::dep1::D1_PUB_SELF;
    let a = crate::dep1::D1_PUB_SUPER;

    let a = crate::dep1::child1::C1_PUB;
    let a = crate::dep1::child1::C1_PUB_CRATE;
    let a = crate::dep1::child1::C1_PUB_IN_CRATE;
    let a = crate::dep1::child1::C1_PUB_SUPER;

    let a = crate::dep1::child2::C2_PUB;
    let a = crate::dep1::child2::C2_PUB_CRATE;
    let a = crate::dep1::child2::C2_PUB_IN_CRATE;
    let a = crate::dep1::child2::C2_PUB_SUPER;

    let a = crate::dep2::D2_PUB;
    let a = crate::dep2::D2_PUB_CRATE;
    let a = crate::dep2::D2_PUB_IN_CRATE;

    let a = crate::dep2::pub_crate_mod::PUB_CRATE_MOD_PUB;
    let a = crate::dep2::pub_crate_mod::PUB_CRATE_MOD_PUB_CRATE;
    let a = crate::dep2::pub_crate_mod::PUB_CRATE_MOD_PUB_IN_CRATE;

    let a = crate::dep2::pub_mod::PUB_MOD_PUB;
    let a = crate::dep2::pub_mod::PUB_MOD_PUB_CRATE;
    let a = crate::dep2::pub_mod::PUB_MOD_PUB_IN_CRATE;
}
