
// configUSE_16_BIT_TICKS = 0
pub type TickType_t = u32;
pub type BaseType_t = i64;
pub type UBaseType_t = u64;

#[cfg(feature = "configUSE_16_BIT_TICKS")]
pub const portMAX_DELAY: TickType_t = 0xffff;
#[cfg(not(feature = "configUSE_16_BIT_TICKS"))]
pub const portMAX_DELAY: TickType_t = 0xffffffff;
