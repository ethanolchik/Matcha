module core;

/* Integer Methods */
func (Int32) MIN(): pub builtin Int32 {
    return -2147483648;
}

func (Int32) MAX(): pub builtin Int32 {
    return 2147483647;
}

func (Int64) MIN(): pub builtin Int64 {
    return -9223372036854775808;
}

func (Int64) MAX(): pub builtin Int64 {
    return 9223372036854775807;
}

func (self: Int32) str(): pub builtin String {
    return int32ToStr(self);
}

func (self: Int32) chr(): pub builtin Char {
    return int32ToChar(self);
}

func (self: Int64) str(): pub builtin String {
    return int64ToStr(self);
}

func (self: Int64) chr(): pub builtin Char {
    return int64ToChar(self);
}

/* Float Methods */
func (Float32) MIN(): pub builtin Float32 {
    return FLOAT32_MIN;
}

func (Float32) MAX(): pub builtin Float32 {
    return FLOAT32_MAX;
}

func (Float64) MIN(): pub builtin Float64 {
    return FLOAT64_MIN;
}

func (Float64) MAX(): pub builtin Float64 {
    return FLOAT64_MAX;
}

func (self: Float32) str(): pub builtin String {
    return float32ToStr(self);
}

func (self: Float64) str(): pub builtin String {
    return float64ToStr(self);
}

/* Bool Methods */
func (self: Bool) str(): pub builtin String {
    return boolToStr(self);
}

func (self: Bool) i32(): pub builtin Int32 {
    return boolToInt32(self);
}

func (self: Bool) i64(): pub builtin Int64 {
    return boolToInt64(self);
}

/* String Methods */
func (self: String) concat(other: String): pub builtin String {
    return concat(self, other);
}

func (self: String) len(): pub builtin Int32 {
    return strlen(self);
}

func (self: String) charAt(index: Int32): pub builtin Char {
    return self[index] as Char;
}

func (self: String) slice(start: Int32, end: Int32): pub builtin String {
    return slice(self, start, end);
}

func (self: String) i32(): pub builtin Int32 {
    return strToInt32(self);
}

func (self: String) i64(): pub builtin Int64 {
    return strToInt32(self) as Int64;
}

func (self: String) f32(): pub builtin Float32 {
    return strToFloat32(self);
}

func (self: String) f64(): pub builtin Float64 {
    return strToFloat64(self);
}

/* Char Methods */
func (self: Char) str(): pub builtin String {
    return charToStr(self);
}