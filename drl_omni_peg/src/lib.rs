pub mod env;

pub mod gymnasium {
    include!(concat!(env!("OUT_DIR"), "/bindings_gymnasium.rs"));
    pub use gymnasium::*;
}

pub mod stable_baselines3 {
    include!(concat!(env!("OUT_DIR"), "/bindings_sb3.rs"));
    pub use sb3_contrib as contrib;
    pub use stable_baselines3::*;
}
