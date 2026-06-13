pub mod guard;

#[cfg(all(target_os = "windows", feature = "steam"))]
pub mod sync;
