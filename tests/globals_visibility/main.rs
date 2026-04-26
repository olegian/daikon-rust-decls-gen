mod dep1;
mod dep2;

fn main() {
    // Isolating what's available where is weirdly difficult.
    // In this file, I will try to write out what constants are accessible.
    let a = crate::dep1::D1_PUB;
    let a = crate::dep1::D1_PUB_CRATE;
    let a = crate::dep1::D1_PUB_IN_CRATE;
    let a = crate::dep1::D1_PUB_SUPER;

    let a = crate::dep1::child1::C1_PUB;
    let a = crate::dep1::child1::C1_PUB_CRATE;
    let a = crate::dep1::child1::C1_PUB_IN_CRATE;

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
