module core;

const FLOAT32_MIN: extern Float32;
const FLOAT32_MAX: extern Float32;
const FLOAT64_MIN: extern Float64;
const FLOAT64_MAX: extern Float64;

func strlen(str: String): extern Int32;
func slice(str: String, start: Int32, end: Int32): extern String;
func concat(str1: String, str2: String): extern String;

func int32ToChar(n: Int32): extern Char;
func int64ToChar(n: Int64): extern Char;

func int32ToStr(n: Int32): extern String;
func int64ToStr(n: Int64): extern String;

func float32ToStr(n: Float32): extern String;
func float64ToStr(n: Float64): extern String;

func boolToStr(b: Bool): extern String;
func boolToInt32(b: Bool): extern Int32;

func strToInt32(str: String): extern Int32;
func strToInt64(str: String): extern Int64;
func strToFloat32(str: String): extern Float32;
func strToFloat64(str: String): extern Float64;

func charToStr(c: Char): extern String;