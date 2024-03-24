pub struct Port(pub u16);

impl Port {
    pub fn write_u8(&self, value: u8) {
        unsafe {
            core::arch::asm!(
                "outb %al, %dx",
                in("al") value,
                in("dx") self.0,
                options(att_syntax)
            );
        }
    }
}
