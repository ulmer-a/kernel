/// An entry in the bootloader-provided module list.
#[repr(C)]
pub struct Module {
    /// Start address of the module.
    mod_start: u32,

    /// End address of the module.
    mod_end: u32,

    /// The `string` field provides an arbitrary zero-terminated ASCII string to be associated with
    /// that particular module. It may also be null if there is no associated string.
    string: *const core::ffi::c_char,

    /// Must be ignored by the OS.
    _reserved: u32,
}
