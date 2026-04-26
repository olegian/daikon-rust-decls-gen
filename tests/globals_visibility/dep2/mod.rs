
const D2_PRIV: u32 = 10;
pub const D2_PUB: u32 = 10;
pub(crate) const D2_PUB_CRATE: u32 = 10;
pub(self) const D2_PUB_SELF: u32 = 10;
pub(crate) const D2_PUB_IN_CRATE: u32 = 10;

mod priv_mod {
    const PRIV_MOD_PRIV: u32 = 10;
    pub const PRIV_MOD_PUB: u32 = 10;
    pub(crate) const PRIV_MOD_PUB_CRATE: u32 = 10;
    pub(self) const PRIV_MOD_PUB_SELF: u32 = 10;
    pub(crate) const PRIV_MOD_PUB_IN_CRATE: u32 = 10;

    mod priv_child_mod {
        const PRIV_CHILD_MOD_PRIV: u32 = 10;
        pub const PRIV_CHILD_MOD_PUB: u32 = 10;
        pub(crate) const PRIV_CHILD_MOD_PUB_CRATE: u32 = 10;
        pub(self) const PRIV_CHILD_MOD_PUB_SELF: u32 = 10;
        pub(crate) const PRIV_CHILD_MOD_PUB_IN_CRATE: u32 = 10;
    }

    pub mod pub_child_mod {
        const PUB_CHILD_MOD_PRIV: u32 = 10;
        pub const PUB_CHILD_MOD_PUB: u32 = 10;
        pub(crate) const PUB_CHILD_MOD_PUB_CRATE: u32 = 10;
        pub(self) const PUB_CHILD_MOD_PUB_SELF: u32 = 10;
        pub(crate) const PUB_CHILD_MOD_PUB_IN_CRATE: u32 = 10;

        pub const REEXPORTED_PUB_CHILD_MOD_PUB_IN_CRATE: u32 = 10;

        fn from_d2_priv_mod_pub_child_mod() {
            let a = crate::dep1::D1_PUB;
            let a = crate::dep1::D1_PUB_CRATE;
            let a = crate::dep1::D1_PUB_IN_CRATE;
            let a = crate::dep1::D1_PUB_SUPER;

            let a = crate::dep1::child1::C1_PUB;
            let a = crate::dep1::child1::C1_PUB_CRATE;
            let a = crate::dep1::child1::C1_PUB_IN_CRATE;

            let a = crate::dep2::D2_PRIV;
            let a = crate::dep2::D2_PUB;
            let a = crate::dep2::D2_PUB_CRATE;
            let a = crate::dep2::D2_PUB_IN_CRATE;
            let a = crate::dep2::D2_PUB_SELF;

            let a = crate::dep2::priv_mod::PRIV_MOD_PRIV;
            let a = crate::dep2::priv_mod::PRIV_MOD_PUB;
            let a = crate::dep2::priv_mod::PRIV_MOD_PUB_CRATE;
            let a = crate::dep2::priv_mod::PRIV_MOD_PUB_SELF;
            let a = crate::dep2::priv_mod::PRIV_MOD_PUB_IN_CRATE;
            let a = crate::dep2::priv_mod::REEXPORTED_PUB_CHILD_MOD_PUB_IN_CRATE;

            let a = crate::dep2::priv_mod::priv_child_mod::PRIV_CHILD_MOD_PUB;
            let a = crate::dep2::priv_mod::priv_child_mod::PRIV_CHILD_MOD_PUB_CRATE;
            let a = crate::dep2::priv_mod::priv_child_mod::PRIV_CHILD_MOD_PUB_IN_CRATE;

            let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PRIV;
            let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB;
            let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB_CRATE;
            let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB_IN_CRATE;
            let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB_SELF;
            let a = crate::dep2::priv_mod::pub_child_mod::REEXPORTED_PUB_CHILD_MOD_PUB_IN_CRATE;

            let a = crate::dep2::priv_mod::pub_in_super_child_mod::PUB_IN_CHILD_MOD_PUB;
            let a = crate::dep2::priv_mod::pub_in_super_child_mod::PUB_IN_CHILD_MOD_PUB_CRATE;
            let a = crate::dep2::priv_mod::pub_in_super_child_mod::PUB_IN_CHILD_MOD_PUB_IN_CRATE;
        }
    }

    pub(super) mod pub_in_super_child_mod {
        const PUB_IN_CHILD_MOD_PRIV: u32 = 10;
        pub const PUB_IN_CHILD_MOD_PUB: u32 = 10;
        pub(crate) const PUB_IN_CHILD_MOD_PUB_CRATE: u32 = 10;
        pub(self) const PUB_IN_CHILD_MOD_PUB_SELF: u32 = 10;
        pub(crate) const PUB_IN_CHILD_MOD_PUB_IN_CRATE: u32 = 10;
    }

    // priv mod will still stop this from appearing outside of this file.
    pub use self::pub_child_mod::REEXPORTED_PUB_CHILD_MOD_PUB_IN_CRATE;
}

pub mod pub_mod {
    const PUB_MOD_PRIV: u32 = 10;
    pub const PUB_MOD_PUB: u32 = 10;
    pub(crate) const PUB_MOD_PUB_CRATE: u32 = 10;
    pub(self) const PUB_MOD_PUB_SELF: u32 = 10;
    pub(crate) const PUB_MOD_PUB_IN_CRATE: u32 = 10;
}

pub(crate) mod pub_crate_mod {
    const PUB_CRATE_MOD_PRIV: u32 = 10;
    pub const PUB_CRATE_MOD_PUB: u32 = 10;
    pub(crate) const PUB_CRATE_MOD_PUB_CRATE: u32 = 10;
    pub(self) const PUB_CRATE_MOD_PUB_SELF: u32 = 10;
    pub(crate) const PUB_CRATE_MOD_PUB_IN_CRATE: u32 = 10;
}

pub(self) mod pub_self_mod {
    const PUB_SELF_MOD_PRIV: u32 = 10;
    pub const PUB_SELF_MOD_PUB: u32 = 10;
    pub(crate) const PUB_SELF_MOD_PUB_CRATE: u32 = 10;
    pub(self) const PUB_SELF_MOD_PUB_SELF: u32 = 10;
    pub(crate) const PUB_SELF_MOD_PUB_IN_CRATE: u32 = 10;
}

fn fn_in_dep2() {
    let a = crate::dep1::D1_PUB;
    let a = crate::dep1::D1_PUB_CRATE;
    let a = crate::dep1::D1_PUB_IN_CRATE;
    let a = crate::dep1::D1_PUB_SUPER;

    let a = crate::dep1::child1::C1_PUB;
    let a = crate::dep1::child1::C1_PUB_CRATE;
    let a = crate::dep1::child1::C1_PUB_IN_CRATE;

    let a = crate::dep2::D2_PRIV;
    let a = crate::dep2::D2_PUB;
    let a = crate::dep2::D2_PUB_CRATE;
    let a = crate::dep2::D2_PUB_IN_CRATE;
    let a = crate::dep2::D2_PUB_SELF;

    let a = crate::dep2::priv_mod::PRIV_MOD_PUB;
    let a = crate::dep2::priv_mod::PRIV_MOD_PUB_CRATE;
    let a = crate::dep2::priv_mod::PRIV_MOD_PUB_IN_CRATE;
    let a = crate::dep2::priv_mod::REEXPORTED_PUB_CHILD_MOD_PUB_IN_CRATE;

    let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB;
    let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB_CRATE;
    let a = crate::dep2::priv_mod::pub_child_mod::PUB_CHILD_MOD_PUB_IN_CRATE;
    let a = crate::dep2::priv_mod::pub_child_mod::REEXPORTED_PUB_CHILD_MOD_PUB_IN_CRATE;

    let a = crate::dep2::priv_mod::pub_in_super_child_mod::PUB_IN_CHILD_MOD_PUB;
    let a = crate::dep2::priv_mod::pub_in_super_child_mod::PUB_IN_CHILD_MOD_PUB_CRATE;
    let a = crate::dep2::priv_mod::pub_in_super_child_mod::PUB_IN_CHILD_MOD_PUB_IN_CRATE;

    let a = crate::dep2::pub_crate_mod::PUB_CRATE_MOD_PUB;
    let a = crate::dep2::pub_crate_mod::PUB_CRATE_MOD_PUB_CRATE;
    let a = crate::dep2::pub_crate_mod::PUB_CRATE_MOD_PUB_IN_CRATE;

    let a = crate::dep2::pub_self_mod::PUB_SELF_MOD_PUB;
    let a = crate::dep2::pub_self_mod::PUB_SELF_MOD_PUB_CRATE;
    let a = crate::dep2::pub_self_mod::PUB_SELF_MOD_PUB_IN_CRATE;
}
