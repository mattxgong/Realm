use std::fmt::{self, Display, Write};

const EVIDENCE_STATEMENT: &str = "RLM-0002 table evidence only; not production conformance.";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum IntegerType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl IntegerType {
    const ALL: [Self; 8] = [
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
        }
    }

    const fn width(self) -> u32 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 => 32,
            Self::I64 | Self::U64 => 64,
        }
    }

    const fn is_signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64)
    }

    fn minimum(self) -> i128 {
        if self.is_signed() {
            -(1_i128 << (self.width() - 1))
        } else {
            0
        }
    }

    fn maximum(self) -> i128 {
        if self.is_signed() {
            (1_i128 << (self.width() - 1)) - 1
        } else {
            (1_i128 << self.width()) - 1
        }
    }

    fn fits(self, value: i128) -> bool {
        (self.minimum()..=self.maximum()).contains(&value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Fault {
    Overflow,
    DivisionByZero,
    NegativeShift,
    ShiftTooWide,
    InvalidCast,
}

impl Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Overflow => "overflow",
            Self::DivisionByZero => "division-by-zero",
            Self::NegativeShift => "negative-shift",
            Self::ShiftTooWide => "shift-too-wide",
            Self::InvalidCast => "invalid-cast",
        })
    }
}

type Checked = Result<i128, Fault>;

fn checked_result(integer_type: IntegerType, value: i128) -> Checked {
    integer_type
        .fits(value)
        .then_some(value)
        .ok_or(Fault::Overflow)
}

fn checked_add(integer_type: IntegerType, lhs: i128, rhs: i128) -> Checked {
    checked_result(integer_type, lhs.checked_add(rhs).ok_or(Fault::Overflow)?)
}

fn checked_sub(integer_type: IntegerType, lhs: i128, rhs: i128) -> Checked {
    checked_result(integer_type, lhs.checked_sub(rhs).ok_or(Fault::Overflow)?)
}

fn checked_mul(integer_type: IntegerType, lhs: i128, rhs: i128) -> Checked {
    checked_result(integer_type, lhs.checked_mul(rhs).ok_or(Fault::Overflow)?)
}

fn checked_neg(integer_type: IntegerType, value: i128) -> Checked {
    if !integer_type.is_signed() {
        return Err(Fault::InvalidCast);
    }
    checked_result(integer_type, value.checked_neg().ok_or(Fault::Overflow)?)
}

fn checked_div(integer_type: IntegerType, lhs: i128, rhs: i128) -> Checked {
    if rhs == 0 {
        return Err(Fault::DivisionByZero);
    }
    if integer_type.is_signed() && lhs == integer_type.minimum() && rhs == -1 {
        return Err(Fault::Overflow);
    }
    checked_result(integer_type, lhs / rhs)
}

fn checked_rem(integer_type: IntegerType, lhs: i128, rhs: i128) -> Checked {
    if rhs == 0 {
        return Err(Fault::DivisionByZero);
    }
    if integer_type.is_signed() && lhs == integer_type.minimum() && rhs == -1 {
        return Err(Fault::Overflow);
    }
    checked_result(integer_type, lhs % rhs)
}

fn shift_count(integer_type: IntegerType, count: i128) -> Result<u32, Fault> {
    if count < 0 {
        Err(Fault::NegativeShift)
    } else if count >= i128::from(integer_type.width()) {
        Err(Fault::ShiftTooWide)
    } else {
        Ok(u32::try_from(count).expect("validated shift count fits u32"))
    }
}

fn checked_shl(integer_type: IntegerType, value: i128, count: i128) -> Checked {
    let count = shift_count(integer_type, count)?;
    let factor = 1_i128 << count;
    checked_result(
        integer_type,
        value.checked_mul(factor).ok_or(Fault::Overflow)?,
    )
}

fn checked_shr(integer_type: IntegerType, value: i128, count: i128) -> Checked {
    let count = shift_count(integer_type, count)?;
    let result = if integer_type.is_signed() {
        value >> count
    } else {
        let unsigned = u128::try_from(value).map_err(|_| Fault::InvalidCast)?;
        i128::try_from(unsigned >> count).map_err(|_| Fault::Overflow)?
    };
    checked_result(integer_type, result)
}

fn cast_integer_to_integer(target: IntegerType, value: i128) -> Checked {
    target
        .fits(value)
        .then_some(value)
        .ok_or(Fault::InvalidCast)
}

fn integer_cast_failure_boundaries(
    source: IntegerType,
    target: IntegerType,
) -> Vec<(&'static str, i128)> {
    [
        ("below", target.minimum() - 1),
        ("above", target.maximum() + 1),
    ]
    .into_iter()
    .filter(|(_, value)| source.fits(*value))
    .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BinaryFormat {
    exponent_bits: u32,
    fraction_bits: u32,
    bias: i32,
}

const BINARY32: BinaryFormat = BinaryFormat {
    exponent_bits: 8,
    fraction_bits: 23,
    bias: 127,
};
const BINARY64: BinaryFormat = BinaryFormat {
    exponent_bits: 11,
    fraction_bits: 52,
    bias: 1023,
};

fn round_right_ties_even(value: u128, shift: u32) -> u128 {
    if shift == 0 {
        return value;
    }
    if shift > 128 {
        return 0;
    }
    if shift == 128 {
        let halfway = 1_u128 << 127;
        return u128::from(value > halfway);
    }
    let retained = value >> shift;
    let mask = (1_u128 << shift) - 1;
    let discarded = value & mask;
    let halfway = 1_u128 << (shift - 1);
    retained + u128::from(discarded > halfway || (discarded == halfway && retained & 1 == 1))
}

fn integer_to_binary_bits(value: i128, format: BinaryFormat) -> u64 {
    if value == 0 {
        return 0;
    }

    let sign = u64::from(value.is_negative()) << (format.exponent_bits + format.fraction_bits);
    let magnitude = value.unsigned_abs();
    let highest_bit = 127 - magnitude.leading_zeros();
    let mut exponent = i32::try_from(highest_bit).expect("bit index fits i32");
    let mut significand = if highest_bit <= format.fraction_bits {
        magnitude << (format.fraction_bits - highest_bit)
    } else {
        round_right_ties_even(magnitude, highest_bit - format.fraction_bits)
    };

    if significand == 1_u128 << (format.fraction_bits + 1) {
        significand >>= 1;
        exponent += 1;
    }

    let exponent_field = u64::try_from(exponent + format.bias).expect("integer casts are finite");
    let fraction_mask = (1_u128 << format.fraction_bits) - 1;
    sign | (exponent_field << format.fraction_bits)
        | u64::try_from(significand & fraction_mask).expect("fraction fits u64")
}

fn float_bits_to_integer(bits: u64, format: BinaryFormat, target: IntegerType) -> Checked {
    let sign_shift = format.exponent_bits + format.fraction_bits;
    let negative = bits >> sign_shift != 0;
    let exponent_mask = (1_u64 << format.exponent_bits) - 1;
    let fraction_mask = (1_u64 << format.fraction_bits) - 1;
    let exponent_field = (bits >> format.fraction_bits) & exponent_mask;
    let fraction = bits & fraction_mask;

    if exponent_field == exponent_mask {
        return Err(Fault::InvalidCast);
    }

    let (significand, exponent) = if exponent_field == 0 {
        (u128::from(fraction), 1 - format.bias)
    } else {
        (
            (1_u128 << format.fraction_bits) | u128::from(fraction),
            i32::try_from(exponent_field).expect("exponent fits i32") - format.bias,
        )
    };
    let binary_shift = exponent - i32::try_from(format.fraction_bits).expect("width fits i32");
    let magnitude = if binary_shift >= 0 {
        significand
            .checked_shl(u32::try_from(binary_shift).expect("nonnegative shift"))
            .ok_or(Fault::InvalidCast)?
    } else {
        let right = binary_shift.unsigned_abs();
        if right >= 128 {
            0
        } else {
            significand >> right
        }
    };

    let value = if negative {
        let minimum_magnitude = 1_u128 << 127;
        if magnitude > minimum_magnitude {
            return Err(Fault::InvalidCast);
        }
        if magnitude == minimum_magnitude {
            i128::MIN
        } else {
            -i128::try_from(magnitude).expect("negative magnitude fits i128")
        }
    } else {
        i128::try_from(magnitude).map_err(|_| Fault::InvalidCast)?
    };

    target
        .fits(value)
        .then_some(value)
        .ok_or(Fault::InvalidCast)
}

fn widen_f32_bits(bits: u32) -> u64 {
    let sign = u64::from(bits >> 31) << 63;
    let exponent = (bits >> 23) & 0xff;
    let fraction = bits & 0x7f_ffff;
    match (exponent, fraction) {
        (0, 0) => sign,
        (0, _) => {
            let highest = 31 - fraction.leading_zeros();
            let unbiased = i32::try_from(highest).expect("bit index fits i32") - 149;
            let exponent64 = u64::try_from(unbiased + 1023).expect("f32 subnormal fits f64");
            let leading = 1_u32 << highest;
            sign | (exponent64 << 52) | (u64::from(fraction - leading) << (52 - highest))
        }
        (0xff, 0) => sign | (0x7ff_u64 << 52),
        (0xff, _) => sign | (0x7ff_u64 << 52) | (u64::from(fraction) << 29),
        _ => {
            let exponent64 =
                u64::try_from(i32::try_from(exponent).expect("f32 exponent fits i32") - 127 + 1023)
                    .expect("normal f32 exponent fits f64");
            sign | (exponent64 << 52) | (u64::from(fraction) << 29)
        }
    }
}

fn narrow_f64_bits(bits: u64) -> u32 {
    let sign = u32::try_from(bits >> 63).expect("sign fits u32") << 31;
    let exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    if exponent == 0x7ff {
        if fraction == 0 {
            return sign | 0x7f80_0000;
        }
        let payload = u32::try_from(fraction >> 29).expect("narrowed payload fits u32");
        return sign | 0x7f80_0000 | payload | 0x0040_0000;
    }
    if exponent == 0 && fraction == 0 {
        return sign;
    }

    let (significand, power) = if exponent == 0 {
        (u128::from(fraction), -1074)
    } else {
        (
            (1_u128 << 52) | u128::from(fraction),
            i32::try_from(exponent).expect("exponent fits i32") - 1023 - 52,
        )
    };
    let highest = 127 - significand.leading_zeros();
    let mut actual_exponent = power + i32::try_from(highest).expect("bit index fits i32");
    if actual_exponent > 127 {
        return sign | 0x7f80_0000;
    }

    if actual_exponent >= -126 {
        let mut rounded = if highest > 23 {
            round_right_ties_even(significand, highest - 23)
        } else {
            significand << (23 - highest)
        };
        if rounded == 1_u128 << 24 {
            rounded >>= 1;
            actual_exponent += 1;
        }
        if actual_exponent > 127 {
            return sign | 0x7f80_0000;
        }
        let exponent32 = u32::try_from(actual_exponent + 127).expect("normal exponent fits");
        let fraction32 = u32::try_from(rounded & 0x7f_ffff).expect("fraction fits u32");
        sign | (exponent32 << 23) | fraction32
    } else {
        let scale = power + 149;
        let units = if scale >= 0 {
            significand
                .checked_shl(u32::try_from(scale).expect("nonnegative scale"))
                .unwrap_or(u128::MAX)
        } else {
            round_right_ties_even(significand, scale.unsigned_abs())
        };
        if units >= 1_u128 << 23 {
            sign | 0x0080_0000
        } else {
            sign | u32::try_from(units).expect("subnormal fraction fits u32")
        }
    }
}

fn integer_to_char(value: i128) -> Result<u32, Fault> {
    let scalar = u32::try_from(value).map_err(|_| Fault::InvalidCast)?;
    char::from_u32(scalar)
        .map(u32::from)
        .ok_or(Fault::InvalidCast)
}

fn char_to_integer(target: IntegerType, scalar: u32) -> Checked {
    if char::from_u32(scalar).is_none() {
        return Err(Fault::InvalidCast);
    }
    cast_integer_to_integer(target, i128::from(scalar))
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SemanticType {
    Primitive(&'static str),
    Nominal(u32),
    Tuple(&'static [&'static str]),
    Array(&'static str, u64),
    SharedReference(&'static str),
    MutableReference(&'static str),
    Slice(&'static str),
    Function(&'static [&'static str], &'static str, bool),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CoercionType {
    Value(&'static str),
    Never,
    SharedReference(&'static str),
    MutableReference(&'static str),
    SharedArrayReference(&'static str, u64),
    MutableArrayReference(&'static str, u64),
    SharedSliceReference(&'static str),
    MutableSliceReference(&'static str),
    FunctionItem(&'static str),
    FunctionPointer(&'static str),
}

fn coercion_rule(source: CoercionType, target: CoercionType) -> Option<&'static str> {
    if source == target {
        return Some("identity");
    }
    match (source, target) {
        (CoercionType::Never, _) => Some("never"),
        (CoercionType::MutableReference(source), CoercionType::SharedReference(target))
            if source == target =>
        {
            Some("shared-reborrow")
        }
        (
            CoercionType::MutableArrayReference(source, _),
            CoercionType::MutableSliceReference(target),
        ) if source == target => Some("array-to-mutable-slice"),
        (
            CoercionType::SharedArrayReference(source, _),
            CoercionType::SharedSliceReference(target),
        ) if source == target => Some("array-to-shared-slice"),
        (
            CoercionType::MutableArrayReference(source, _),
            CoercionType::SharedSliceReference(target),
        ) if source == target => Some("weaken-and-array-to-shared-slice"),
        (CoercionType::FunctionItem(source), CoercionType::FunctionPointer(target))
            if source == target =>
        {
            Some("function-item-to-pointer")
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Layout {
    size: u64,
    alignment: u64,
}

const UNIT_LAYOUT: Layout = Layout {
    size: 0,
    alignment: 1,
};
const BYTE_LAYOUT: Layout = Layout {
    size: 1,
    alignment: 1,
};
const WORD16_LAYOUT: Layout = Layout {
    size: 2,
    alignment: 2,
};
const WORD64_LAYOUT: Layout = Layout {
    size: 8,
    alignment: 8,
};
const STRING_LAYOUT: Layout = Layout {
    size: 24,
    alignment: 8,
};
const SLICE_REFERENCE_LAYOUT: Layout = Layout {
    size: 16,
    alignment: 8,
};

fn align_up(offset: u64, alignment: u64) -> u64 {
    let remainder = offset % alignment;
    if remainder == 0 {
        offset
    } else {
        offset + alignment - remainder
    }
}

fn array_layout(element: Layout, length: u64) -> Option<Layout> {
    Some(Layout {
        size: element.size.checked_mul(length)?,
        alignment: element.alignment,
    })
}

fn tuple_layout(elements: &[Layout]) -> Option<(Layout, Vec<u64>)> {
    let mut offset = 0_u64;
    let mut alignment = 1_u64;
    let mut offsets = Vec::with_capacity(elements.len());
    for element in elements {
        offset = align_up(offset, element.alignment);
        offsets.push(offset);
        offset = offset.checked_add(element.size)?;
        alignment = alignment.max(element.alignment);
    }
    Some((
        Layout {
            size: align_up(offset, alignment),
            alignment,
        },
        offsets,
    ))
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Diagnostic {
    case_id: &'static str,
    code: &'static str,
}

fn deterministic_diagnostics(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort_unstable();
    diagnostics
}

fn result_text(result: Checked) -> String {
    match result {
        Ok(value) => format!("ok:{value}"),
        Err(fault) => format!("fault:{fault}"),
    }
}

fn push_row(rows: &mut Vec<String>, category: &str, case: &str, result: impl Display) {
    rows.push(format!("{category}|{case}|{result}"));
}

fn integer_evidence(rows: &mut Vec<String>) {
    for integer_type in IntegerType::ALL {
        let minimum = integer_type.minimum();
        let maximum = integer_type.maximum();
        for (label, value) in [
            ("min-1", minimum - 1),
            ("min", minimum),
            ("min+1", minimum + 1),
            ("negative-one", -1),
            ("zero", 0),
            ("one", 1),
            ("max-1", maximum - 1),
            ("max", maximum),
            ("max+1", maximum + 1),
        ] {
            push_row(
                rows,
                "integer-fit",
                &format!("{}:{label}:{value}", integer_type.name()),
                integer_type.fits(value),
            );
        }

        let ordinary_lhs = if integer_type.is_signed() { -7 } else { 7 };
        for (operation, result) in [
            ("add-ordinary", checked_add(integer_type, ordinary_lhs, 3)),
            ("add-overflow", checked_add(integer_type, maximum, 1)),
            ("sub-ordinary", checked_sub(integer_type, 7, 3)),
            ("sub-overflow", checked_sub(integer_type, minimum, 1)),
            ("mul-ordinary", checked_mul(integer_type, ordinary_lhs, 3)),
            ("mul-overflow", checked_mul(integer_type, maximum, 2)),
            ("div-ordinary", checked_div(integer_type, ordinary_lhs, 3)),
            ("div-zero", checked_div(integer_type, ordinary_lhs, 0)),
            ("rem-ordinary", checked_rem(integer_type, ordinary_lhs, 3)),
            ("rem-zero", checked_rem(integer_type, ordinary_lhs, 0)),
        ] {
            push_row(
                rows,
                "integer-op",
                &format!("{}:{operation}", integer_type.name()),
                result_text(result),
            );
        }
        if integer_type.is_signed() {
            for (operation, result) in [
                ("neg-ordinary", checked_neg(integer_type, 1)),
                ("neg-min", checked_neg(integer_type, minimum)),
                (
                    "div-min-negative-one",
                    checked_div(integer_type, minimum, -1),
                ),
                (
                    "rem-min-negative-one",
                    checked_rem(integer_type, minimum, -1),
                ),
            ] {
                push_row(
                    rows,
                    "integer-op",
                    &format!("{}:{operation}", integer_type.name()),
                    result_text(result),
                );
            }
        }

        let mut shift_values = vec![0, 1, maximum];
        if integer_type.is_signed() {
            shift_values.extend([minimum, -1]);
        }
        let width = i128::from(integer_type.width());
        for value in shift_values {
            for count in [-1, 0, 1, width - 1, width, width + 1] {
                push_row(
                    rows,
                    "shift-left",
                    &format!("{}:{value}:{count}", integer_type.name()),
                    result_text(checked_shl(integer_type, value, count)),
                );
                push_row(
                    rows,
                    "shift-right",
                    &format!("{}:{value}:{count}", integer_type.name()),
                    result_text(checked_shr(integer_type, value, count)),
                );
            }
        }
    }
}

fn cast_evidence(rows: &mut Vec<String>) {
    for source in IntegerType::ALL {
        for target in IntegerType::ALL {
            let inside = source.minimum().max(target.minimum());
            push_row(
                rows,
                "cast-int-int",
                &format!("{}:{}:inside", source.name(), target.name()),
                result_text(cast_integer_to_integer(target, inside)),
            );
            let failure_boundaries = integer_cast_failure_boundaries(source, target);
            if failure_boundaries.is_empty() {
                push_row(
                    rows,
                    "cast-int-int-domain",
                    &format!("{}:{}", source.name(), target.name()),
                    "all-source-values-fit",
                );
            }
            for (boundary, value) in failure_boundaries {
                push_row(
                    rows,
                    "cast-int-int",
                    &format!("{}:{}:{boundary}:{value}", source.name(), target.name()),
                    result_text(cast_integer_to_integer(target, value)),
                );
            }
        }
    }

    for value in [
        i64::MIN as i128,
        -16_777_217,
        -1,
        0,
        1,
        16_777_217,
        u64::MAX as i128,
    ] {
        push_row(
            rows,
            "cast-int-f32",
            &value.to_string(),
            format!("{:08x}", integer_to_binary_bits(value, BINARY32)),
        );
        push_row(
            rows,
            "cast-int-f64",
            &value.to_string(),
            format!("{:016x}", integer_to_binary_bits(value, BINARY64)),
        );
    }

    let float_integer_cases = [
        ("positive-zero", 0x0000_0000_0000_0000),
        ("negative-zero", 0x8000_0000_0000_0000),
        ("positive-fraction", 0x3ff8_0000_0000_0000),
        ("negative-fraction", 0xbff8_0000_0000_0000),
        ("i32-max", 0x41df_ffff_ffe0_0000),
        ("i32-max-outside", 0x41e0_0000_0000_0000),
        ("i32-min", 0xc1e0_0000_0000_0000),
        ("i32-min-outside", 0xc1e0_0000_0020_0000),
        ("positive-infinity", 0x7ff0_0000_0000_0000),
        ("negative-infinity", 0xfff0_0000_0000_0000),
        ("quiet-nan", 0x7ff8_0000_0000_0001),
    ];
    for (case, bits) in float_integer_cases {
        push_row(
            rows,
            "cast-f64-i32",
            case,
            result_text(float_bits_to_integer(bits, BINARY64, IntegerType::I32)),
        );
    }

    for value in [-1, 0, 0xd7ff, 0xd800, 0xdfff, 0xe000, 0x10ffff, 0x110000] {
        let result = integer_to_char(value);
        push_row(
            rows,
            "cast-int-char",
            &value.to_string(),
            match result {
                Ok(scalar) => format!("ok:{scalar:06x}"),
                Err(fault) => format!("fault:{fault}"),
            },
        );
    }
    for scalar in [0, 0x7f, 0x80, 0xd7ff, 0xe000, 0x10ffff] {
        for target in IntegerType::ALL {
            push_row(
                rows,
                "cast-char-int",
                &format!("{scalar:06x}:{}", target.name()),
                result_text(char_to_integer(target, scalar)),
            );
        }
    }
}

fn classify_f32(bits: u32) -> &'static str {
    let exponent = (bits >> 23) & 0xff;
    let fraction = bits & 0x7f_ffff;
    match (exponent, fraction) {
        (0, 0) => "zero",
        (0, _) => "subnormal",
        (0xff, 0) => "infinity",
        (0xff, _) => "nan",
        _ => "normal",
    }
}

fn classify_f64(bits: u64) -> &'static str {
    let exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    match (exponent, fraction) {
        (0, 0) => "zero",
        (0, _) => "subnormal",
        (0x7ff, 0) => "infinity",
        (0x7ff, _) => "nan",
        _ => "normal",
    }
}

fn floating_evidence(rows: &mut Vec<String>) {
    let f32_cases = [
        ("positive-zero", 0x0000_0000),
        ("negative-zero", 0x8000_0000),
        ("minimum-subnormal", 0x0000_0001),
        ("maximum-subnormal", 0x007f_ffff),
        ("minimum-normal", 0x0080_0000),
        ("maximum-finite", 0x7f7f_ffff),
        ("positive-infinity", 0x7f80_0000),
        ("negative-infinity", 0xff80_0000),
        ("quiet-nan", 0x7fc0_0001),
    ];
    for (name, bits) in f32_cases {
        push_row(rows, "float-f32-class", name, classify_f32(bits));
        push_row(
            rows,
            "float-f32-widen",
            name,
            format!("{:016x}", widen_f32_bits(bits)),
        );
    }

    let f64_cases = [
        ("positive-zero", 0x0000_0000_0000_0000),
        ("negative-zero", 0x8000_0000_0000_0000),
        ("minimum-subnormal", 0x0000_0000_0000_0001),
        ("maximum-subnormal", 0x000f_ffff_ffff_ffff),
        ("minimum-normal", 0x0010_0000_0000_0000),
        ("maximum-finite", 0x7fef_ffff_ffff_ffff),
        ("positive-infinity", 0x7ff0_0000_0000_0000),
        ("negative-infinity", 0xfff0_0000_0000_0000),
        ("quiet-nan", 0x7ff8_0000_0000_0001),
    ];
    for (name, bits) in f64_cases {
        push_row(rows, "float-f64-class", name, classify_f64(bits));
        push_row(
            rows,
            "float-f64-narrow",
            name,
            format!("{:08x}", narrow_f64_bits(bits)),
        );
    }

    let positive_zero32 = f32::from_bits(0x0000_0000);
    let negative_zero32 = f32::from_bits(0x8000_0000);
    let nan32 = f32::from_bits(0x7fc0_0001);
    let nan32_peer = f32::from_bits(0x7fc0_0001);
    let operations32 = [
        (
            "zeros-equal",
            (positive_zero32 == negative_zero32).to_string(),
        ),
        ("nan-equal", (nan32 == nan32_peer).to_string()),
        ("nan-not-equal", (nan32 != nan32_peer).to_string()),
        ("nan-less", (nan32 < 1.0).to_string()),
        (
            "positive-div-zero",
            format!("{:08x}", (1.0_f32 / positive_zero32).to_bits()),
        ),
        (
            "negative-div-zero",
            format!("{:08x}", (-1.0_f32 / positive_zero32).to_bits()),
        ),
        (
            "positive-frem",
            format!("{:08x}", (5.5_f32 % 2.0).to_bits()),
        ),
        (
            "negative-frem",
            format!("{:08x}", (-5.5_f32 % 2.0).to_bits()),
        ),
    ];
    for (case, result) in operations32 {
        push_row(rows, "float-f32-op", case, result);
    }

    let positive_zero64 = f64::from_bits(0x0000_0000_0000_0000);
    let negative_zero64 = f64::from_bits(0x8000_0000_0000_0000);
    let nan64 = f64::from_bits(0x7ff8_0000_0000_0001);
    let nan64_peer = f64::from_bits(0x7ff8_0000_0000_0001);
    let operations64 = [
        (
            "zeros-equal",
            (positive_zero64 == negative_zero64).to_string(),
        ),
        ("nan-equal", (nan64 == nan64_peer).to_string()),
        ("nan-not-equal", (nan64 != nan64_peer).to_string()),
        ("nan-greater", (nan64 > 1.0).to_string()),
        (
            "positive-div-zero",
            format!("{:016x}", (1.0_f64 / positive_zero64).to_bits()),
        ),
        (
            "negative-div-zero",
            format!("{:016x}", (-1.0_f64 / positive_zero64).to_bits()),
        ),
        (
            "positive-frem",
            format!("{:016x}", (5.5_f64 % 2.0).to_bits()),
        ),
        (
            "negative-frem",
            format!("{:016x}", (-5.5_f64 % 2.0).to_bits()),
        ),
    ];
    for (case, result) in operations64 {
        push_row(rows, "float-f64-op", case, result);
    }
}

struct IdentityCase {
    name: &'static str,
    lhs: SemanticType,
    rhs: SemanticType,
    expected: bool,
}

fn identity_cases() -> Vec<IdentityCase> {
    vec![
        IdentityCase {
            name: "primitive-i32",
            lhs: SemanticType::Primitive("i32"),
            rhs: SemanticType::Primitive("i32"),
            expected: true,
        },
        IdentityCase {
            name: "primitive-char",
            lhs: SemanticType::Primitive("char"),
            rhs: SemanticType::Primitive("char"),
            expected: true,
        },
        IdentityCase {
            name: "nominal-same",
            lhs: SemanticType::Nominal(1),
            rhs: SemanticType::Nominal(1),
            expected: true,
        },
        IdentityCase {
            name: "tuple-same",
            lhs: SemanticType::Tuple(&["i32", "bool"]),
            rhs: SemanticType::Tuple(&["i32", "bool"]),
            expected: true,
        },
        IdentityCase {
            name: "tuple-unit",
            lhs: SemanticType::Tuple(&[]),
            rhs: SemanticType::Tuple(&[]),
            expected: true,
        },
        IdentityCase {
            name: "array-same",
            lhs: SemanticType::Array("u8", 4),
            rhs: SemanticType::Array("u8", 4),
            expected: true,
        },
        IdentityCase {
            name: "shared-ref-same",
            lhs: SemanticType::SharedReference("i64"),
            rhs: SemanticType::SharedReference("i64"),
            expected: true,
        },
        IdentityCase {
            name: "mut-ref-same",
            lhs: SemanticType::MutableReference("i64"),
            rhs: SemanticType::MutableReference("i64"),
            expected: true,
        },
        IdentityCase {
            name: "slice-same",
            lhs: SemanticType::Slice("char"),
            rhs: SemanticType::Slice("char"),
            expected: true,
        },
        IdentityCase {
            name: "fn-same",
            lhs: SemanticType::Function(&["i32"], "bool", false),
            rhs: SemanticType::Function(&["i32"], "bool", false),
            expected: true,
        },
        IdentityCase {
            name: "fn-throws-same",
            lhs: SemanticType::Function(&["i32"], "bool", true),
            rhs: SemanticType::Function(&["i32"], "bool", true),
            expected: true,
        },
        IdentityCase {
            name: "string-nominal-same",
            lhs: SemanticType::Nominal(100),
            rhs: SemanticType::Nominal(100),
            expected: true,
        },
        IdentityCase {
            name: "primitive-width",
            lhs: SemanticType::Primitive("i32"),
            rhs: SemanticType::Primitive("i64"),
            expected: false,
        },
        IdentityCase {
            name: "nominal-distinct",
            lhs: SemanticType::Nominal(1),
            rhs: SemanticType::Nominal(2),
            expected: false,
        },
        IdentityCase {
            name: "nominal-layout-irrelevant",
            lhs: SemanticType::Nominal(1),
            rhs: SemanticType::Tuple(&["i32"]),
            expected: false,
        },
        IdentityCase {
            name: "tuple-order",
            lhs: SemanticType::Tuple(&["i32", "bool"]),
            rhs: SemanticType::Tuple(&["bool", "i32"]),
            expected: false,
        },
        IdentityCase {
            name: "tuple-arity",
            lhs: SemanticType::Tuple(&["i32"]),
            rhs: SemanticType::Tuple(&["i32", "i32"]),
            expected: false,
        },
        IdentityCase {
            name: "array-element",
            lhs: SemanticType::Array("u8", 4),
            rhs: SemanticType::Array("i8", 4),
            expected: false,
        },
        IdentityCase {
            name: "array-length",
            lhs: SemanticType::Array("u8", 4),
            rhs: SemanticType::Array("u8", 5),
            expected: false,
        },
        IdentityCase {
            name: "reference-mutability",
            lhs: SemanticType::SharedReference("i64"),
            rhs: SemanticType::MutableReference("i64"),
            expected: false,
        },
        IdentityCase {
            name: "reference-referent",
            lhs: SemanticType::SharedReference("i64"),
            rhs: SemanticType::SharedReference("u64"),
            expected: false,
        },
        IdentityCase {
            name: "slice-element",
            lhs: SemanticType::Slice("char"),
            rhs: SemanticType::Slice("u32"),
            expected: false,
        },
        IdentityCase {
            name: "fn-return",
            lhs: SemanticType::Function(&["i32"], "bool", false),
            rhs: SemanticType::Function(&["i32"], "i32", false),
            expected: false,
        },
        IdentityCase {
            name: "fn-effect",
            lhs: SemanticType::Function(&["i32"], "bool", false),
            rhs: SemanticType::Function(&["i32"], "bool", true),
            expected: false,
        },
    ]
}

struct CoercionCase {
    name: &'static str,
    source: CoercionType,
    target: CoercionType,
    expected: Option<&'static str>,
}

fn coercion_cases() -> Vec<CoercionCase> {
    vec![
        CoercionCase {
            name: "identity-i32",
            source: CoercionType::Value("i32"),
            target: CoercionType::Value("i32"),
            expected: Some("identity"),
        },
        CoercionCase {
            name: "identity-string",
            source: CoercionType::Value("String"),
            target: CoercionType::Value("String"),
            expected: Some("identity"),
        },
        CoercionCase {
            name: "identity-shared-ref",
            source: CoercionType::SharedReference("i32"),
            target: CoercionType::SharedReference("i32"),
            expected: Some("identity"),
        },
        CoercionCase {
            name: "identity-mut-ref",
            source: CoercionType::MutableReference("i32"),
            target: CoercionType::MutableReference("i32"),
            expected: Some("identity"),
        },
        CoercionCase {
            name: "never-i32",
            source: CoercionType::Never,
            target: CoercionType::Value("i32"),
            expected: Some("never"),
        },
        CoercionCase {
            name: "never-string",
            source: CoercionType::Never,
            target: CoercionType::Value("String"),
            expected: Some("never"),
        },
        CoercionCase {
            name: "never-slice",
            source: CoercionType::Never,
            target: CoercionType::SharedSliceReference("u8"),
            expected: Some("never"),
        },
        CoercionCase {
            name: "shared-reborrow",
            source: CoercionType::MutableReference("i32"),
            target: CoercionType::SharedReference("i32"),
            expected: Some("shared-reborrow"),
        },
        CoercionCase {
            name: "mutable-array-slice",
            source: CoercionType::MutableArrayReference("i32", 4),
            target: CoercionType::MutableSliceReference("i32"),
            expected: Some("array-to-mutable-slice"),
        },
        CoercionCase {
            name: "shared-array-slice",
            source: CoercionType::SharedArrayReference("i32", 4),
            target: CoercionType::SharedSliceReference("i32"),
            expected: Some("array-to-shared-slice"),
        },
        CoercionCase {
            name: "weaken-array-slice",
            source: CoercionType::MutableArrayReference("i32", 4),
            target: CoercionType::SharedSliceReference("i32"),
            expected: Some("weaken-and-array-to-shared-slice"),
        },
        CoercionCase {
            name: "function-item",
            source: CoercionType::FunctionItem("fn(i32)->bool"),
            target: CoercionType::FunctionPointer("fn(i32)->bool"),
            expected: Some("function-item-to-pointer"),
        },
        CoercionCase {
            name: "integer-widen",
            source: CoercionType::Value("i32"),
            target: CoercionType::Value("i64"),
            expected: None,
        },
        CoercionCase {
            name: "signedness",
            source: CoercionType::Value("i32"),
            target: CoercionType::Value("u32"),
            expected: None,
        },
        CoercionCase {
            name: "integer-float",
            source: CoercionType::Value("i32"),
            target: CoercionType::Value("f64"),
            expected: None,
        },
        CoercionCase {
            name: "float-widen",
            source: CoercionType::Value("f32"),
            target: CoercionType::Value("f64"),
            expected: None,
        },
        CoercionCase {
            name: "bool-integer",
            source: CoercionType::Value("bool"),
            target: CoercionType::Value("u8"),
            expected: None,
        },
        CoercionCase {
            name: "nominal-wrapper",
            source: CoercionType::Value("UserId"),
            target: CoercionType::Value("u64"),
            expected: None,
        },
        CoercionCase {
            name: "string-allocation",
            source: CoercionType::Value("literal"),
            target: CoercionType::Value("String"),
            expected: None,
        },
        CoercionCase {
            name: "shared-to-mutable",
            source: CoercionType::SharedReference("i32"),
            target: CoercionType::MutableReference("i32"),
            expected: None,
        },
        CoercionCase {
            name: "reborrow-referent",
            source: CoercionType::MutableReference("i32"),
            target: CoercionType::SharedReference("u32"),
            expected: None,
        },
        CoercionCase {
            name: "array-element",
            source: CoercionType::SharedArrayReference("i32", 4),
            target: CoercionType::SharedSliceReference("u32"),
            expected: None,
        },
        CoercionCase {
            name: "shared-to-mutable-slice",
            source: CoercionType::SharedArrayReference("i32", 4),
            target: CoercionType::MutableSliceReference("i32"),
            expected: None,
        },
        CoercionCase {
            name: "slice-to-array",
            source: CoercionType::SharedSliceReference("i32"),
            target: CoercionType::SharedArrayReference("i32", 4),
            expected: None,
        },
        CoercionCase {
            name: "function-signature",
            source: CoercionType::FunctionItem("fn(i32)->bool"),
            target: CoercionType::FunctionPointer("fn(i64)->bool"),
            expected: None,
        },
        CoercionCase {
            name: "function-pointer-item",
            source: CoercionType::FunctionPointer("fn(i32)->bool"),
            target: CoercionType::FunctionItem("fn(i32)->bool"),
            expected: None,
        },
    ]
}

fn judgment_evidence(rows: &mut Vec<String>) {
    for case in identity_cases() {
        let actual = case.lhs == case.rhs;
        assert_eq!(actual, case.expected, "identity case {}", case.name);
        push_row(rows, "identity", case.name, actual);
    }
    for case in coercion_cases() {
        let actual = coercion_rule(case.source, case.target);
        assert_eq!(actual, case.expected, "coercion case {}", case.name);
        push_row(rows, "coercion", case.name, actual.unwrap_or("rejected"));
    }
}

fn layout_evidence(rows: &mut Vec<String>) {
    let aligned_empty_array = array_layout(WORD64_LAYOUT, 0).expect("word array layout");
    for (name, layout) in [
        (
            "[();0]",
            array_layout(UNIT_LAYOUT, 0).expect("unit array layout"),
        ),
        (
            "[();4]",
            array_layout(UNIT_LAYOUT, 4).expect("unit array layout"),
        ),
        (
            "[u8;0]",
            array_layout(BYTE_LAYOUT, 0).expect("byte array layout"),
        ),
        ("[u64;0]", aligned_empty_array),
        ("String", STRING_LAYOUT),
        ("&[T]", SLICE_REFERENCE_LAYOUT),
    ] {
        push_row(
            rows,
            "layout",
            name,
            format!("{}:{}", layout.size, layout.alignment),
        );
    }
    let (tuple, offsets) =
        tuple_layout(&[BYTE_LAYOUT, WORD64_LAYOUT, WORD16_LAYOUT]).expect("tuple layout");
    push_row(
        rows,
        "layout",
        "(u8,u64,u16)",
        format!("{}:{}:{offsets:?}", tuple.size, tuple.alignment),
    );
    let (aligned_empty_tuple, offsets) =
        tuple_layout(&[aligned_empty_array]).expect("zero-sized tuple layout");
    push_row(
        rows,
        "layout",
        "([u64;0],)",
        format!(
            "{}:{}:{offsets:?}",
            aligned_empty_tuple.size, aligned_empty_tuple.alignment
        ),
    );

    let representations = [
        ("i8/u8", "1:1:i8:integer-right-justified"),
        ("i16/u16", "2:2:i16:integer-right-justified"),
        ("i32/u32", "4:4:i32:integer-right-justified"),
        ("i64/u64", "8:8:i64:integer"),
        ("f32", "4:4:float:xmm-or-stack"),
        ("f64", "8:8:double:xmm-or-stack"),
        (
            "bool",
            "1:1:i8-memory-i1-ssa:canonical-integer-zero-extended",
        ),
        ("char", "4:4:i32:integer"),
        ("unit", "0:1:void:omitted"),
        ("never", "no-layout:unreachable:no-callable-value"),
        (
            "String",
            "24:8:ptr-u64-u64:indirect-argument-hidden-result-pointer",
        ),
        ("sized-reference", "8:8:ptr:pointer"),
        (
            "slice-reference",
            "16:8:ptr-u64:indirect-argument-hidden-result-pointer",
        ),
        ("zero-sized-aggregate", "0:max-member:omitted-payload"),
        ("function-pointer", "8:8:ptr:pointer"),
    ];
    for (name, representation) in representations {
        push_row(rows, "representation", name, representation);
    }
}

fn semantic_inventory() -> Vec<String> {
    let mut rows = Vec::new();
    integer_evidence(&mut rows);
    cast_evidence(&mut rows);
    floating_evidence(&mut rows);
    judgment_evidence(&mut rows);
    layout_evidence(&mut rows);
    for diagnostic in deterministic_diagnostics(vec![
        Diagnostic {
            case_id: "shift-width",
            code: "E-SHIFT-WIDTH",
        },
        Diagnostic {
            case_id: "cast-surrogate",
            code: "E-CAST-SCALAR",
        },
        Diagnostic {
            case_id: "cast-surrogate",
            code: "E-CAST-RANGE",
        },
    ]) {
        push_row(
            &mut rows,
            "diagnostic-order",
            diagnostic.case_id,
            diagnostic.code,
        );
    }
    rows.sort_unstable();
    rows
}

fn fnv1a64(rows: &[String]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for row in rows {
        for byte in row.bytes().chain(b"\n".iter().copied()) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
    }
    hash
}

fn output() -> String {
    let rows = semantic_inventory();
    let mut output = String::new();
    writeln!(
        output,
        "RLM-0002 semantic digest: fnv1a64:{:016x} rows:{}",
        fnv1a64(&rows),
        rows.len()
    )
    .expect("writing to String cannot fail");
    writeln!(output, "{EVIDENCE_STATEMENT}").expect("writing to String cannot fail");
    output
}

fn main() {
    print!("{}", output());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_integer_domains_include_only_minimum_through_maximum() {
        for integer_type in IntegerType::ALL {
            assert!(!integer_type.fits(integer_type.minimum() - 1));
            assert!(integer_type.fits(integer_type.minimum()));
            assert!(integer_type.fits(integer_type.maximum()));
            assert!(!integer_type.fits(integer_type.maximum() + 1));
        }
    }

    #[test]
    fn checked_add_subtract_and_multiply_reject_boundaries() {
        for integer_type in IntegerType::ALL {
            assert_eq!(
                checked_add(integer_type, integer_type.maximum(), 1),
                Err(Fault::Overflow)
            );
            assert_eq!(
                checked_sub(integer_type, integer_type.minimum(), 1),
                Err(Fault::Overflow)
            );
            assert_eq!(
                checked_mul(integer_type, integer_type.maximum(), 2),
                Err(Fault::Overflow)
            );
            assert_eq!(checked_add(integer_type, 1, 2), Ok(3));
        }
    }

    #[test]
    fn signed_negation_division_and_remainder_match_realm_boundaries() {
        for integer_type in [
            IntegerType::I8,
            IntegerType::I16,
            IntegerType::I32,
            IntegerType::I64,
        ] {
            let minimum = integer_type.minimum();
            assert_eq!(checked_neg(integer_type, minimum), Err(Fault::Overflow));
            assert_eq!(checked_div(integer_type, minimum, -1), Err(Fault::Overflow));
            assert_eq!(checked_rem(integer_type, minimum, -1), Err(Fault::Overflow));
            assert_eq!(checked_div(integer_type, -7, 3), Ok(-2));
            assert_eq!(checked_rem(integer_type, -7, 3), Ok(-1));
            assert_eq!(checked_div(integer_type, 1, 0), Err(Fault::DivisionByZero));
        }
    }

    #[test]
    fn shifts_reject_counts_and_left_value_overflow() {
        for integer_type in IntegerType::ALL {
            let width = i128::from(integer_type.width());
            assert_eq!(checked_shl(integer_type, 1, -1), Err(Fault::NegativeShift));
            assert_eq!(
                checked_shl(integer_type, 1, width),
                Err(Fault::ShiftTooWide)
            );
            assert_eq!(
                checked_shr(integer_type, 1, width + 1),
                Err(Fault::ShiftTooWide)
            );
            assert_eq!(
                checked_shl(integer_type, integer_type.maximum(), 1),
                Err(Fault::Overflow)
            );
            assert_eq!(checked_shl(integer_type, 1, 1), Ok(2));
        }
        assert_eq!(checked_shr(IntegerType::I8, -2, 1), Ok(-1));
        assert_eq!(checked_shr(IntegerType::U8, 0x80, 1), Ok(0x40));
    }

    #[test]
    fn integer_cast_matrix_checks_nearest_failure_boundaries() {
        for source in IntegerType::ALL {
            for target in IntegerType::ALL {
                let inside = source.minimum().max(target.minimum());
                assert!(source.fits(inside));
                assert_eq!(cast_integer_to_integer(target, inside), Ok(inside));

                let failure_boundaries = integer_cast_failure_boundaries(source, target);
                if failure_boundaries.is_empty() {
                    assert!(target.fits(source.minimum()));
                    assert!(target.fits(source.maximum()));
                }
                for (_, value) in failure_boundaries {
                    assert!(source.fits(value));
                    assert!(!target.fits(value));
                    assert_eq!(
                        cast_integer_to_integer(target, value),
                        Err(Fault::InvalidCast)
                    );
                }
            }
        }

        for target in IntegerType::ALL {
            assert_eq!(
                cast_integer_to_integer(target, target.minimum()),
                Ok(target.minimum())
            );
            assert_eq!(
                cast_integer_to_integer(target, target.maximum()),
                Ok(target.maximum())
            );
            assert_eq!(
                cast_integer_to_integer(target, target.minimum() - 1),
                Err(Fault::InvalidCast)
            );
            assert_eq!(
                cast_integer_to_integer(target, target.maximum() + 1),
                Err(Fault::InvalidCast)
            );
        }
        assert_eq!(
            cast_integer_to_integer(IntegerType::U64, -1),
            Err(Fault::InvalidCast)
        );
        assert_eq!(
            cast_integer_to_integer(IntegerType::I64, u64::MAX as i128),
            Err(Fault::InvalidCast)
        );
    }

    #[test]
    fn integer_to_float_rounding_is_computed_from_integer_bits() {
        assert_eq!(integer_to_binary_bits(0, BINARY32), 0x0000_0000);
        assert_eq!(integer_to_binary_bits(-1, BINARY32), 0xbf80_0000);
        assert_eq!(integer_to_binary_bits(16_777_217, BINARY32), 0x4b80_0000);
        assert_eq!(integer_to_binary_bits(16_777_219, BINARY32), 0x4b80_0002);
        assert_eq!(
            integer_to_binary_bits(i64::MIN as i128, BINARY64),
            0xc3e0_0000_0000_0000
        );
        assert_eq!(
            integer_to_binary_bits(u64::MAX as i128, BINARY64),
            0x43f0_0000_0000_0000
        );
    }

    #[test]
    fn float_to_integer_checks_zero_fraction_bounds_infinity_and_nan() {
        assert_eq!(
            float_bits_to_integer(0x0000_0000_0000_0000, BINARY64, IntegerType::I32),
            Ok(0)
        );
        assert_eq!(
            float_bits_to_integer(0x8000_0000_0000_0000, BINARY64, IntegerType::I32),
            Ok(0)
        );
        assert_eq!(
            float_bits_to_integer(0x3ff8_0000_0000_0000, BINARY64, IntegerType::I32),
            Ok(1)
        );
        assert_eq!(
            float_bits_to_integer(0xbff8_0000_0000_0000, BINARY64, IntegerType::I32),
            Ok(-1)
        );
        assert_eq!(
            float_bits_to_integer(0x41df_ffff_ffe0_0000, BINARY64, IntegerType::I32),
            Ok(i32::MAX.into())
        );
        assert_eq!(
            float_bits_to_integer(0x41e0_0000_0000_0000, BINARY64, IntegerType::I32),
            Err(Fault::InvalidCast)
        );
        assert_eq!(
            float_bits_to_integer(0xc1e0_0000_0000_0000, BINARY64, IntegerType::I32),
            Ok(i32::MIN.into())
        );
        assert_eq!(
            float_bits_to_integer(0x7ff0_0000_0000_0000, BINARY64, IntegerType::I32),
            Err(Fault::InvalidCast)
        );
        assert_eq!(
            float_bits_to_integer(0x7ff8_0000_0000_0001, BINARY64, IntegerType::I32),
            Err(Fault::InvalidCast)
        );
    }

    #[test]
    fn float_classes_cover_fixed_ieee_bit_patterns() {
        assert_eq!(classify_f32(0x8000_0000), "zero");
        assert_eq!(classify_f32(0x0000_0001), "subnormal");
        assert_eq!(classify_f32(0x0080_0000), "normal");
        assert_eq!(classify_f32(0x7f7f_ffff), "normal");
        assert_eq!(classify_f32(0x7f80_0000), "infinity");
        assert_eq!(classify_f32(0x7fc0_0001), "nan");
        assert_eq!(classify_f64(0x8000_0000_0000_0000), "zero");
        assert_eq!(classify_f64(0x0000_0000_0000_0001), "subnormal");
        assert_eq!(classify_f64(0x0010_0000_0000_0000), "normal");
        assert_eq!(classify_f64(0x7fef_ffff_ffff_ffff), "normal");
        assert_eq!(classify_f64(0x7ff0_0000_0000_0000), "infinity");
        assert_eq!(classify_f64(0x7ff8_0000_0000_0001), "nan");
    }

    #[test]
    fn float_comparison_division_and_frem_match_fixed_expectations() {
        let nan32 = f32::from_bits(0x7fc0_0001);
        let nan32_peer = f32::from_bits(0x7fc0_0001);
        assert_eq!(0.0_f32, -0.0_f32);
        assert_ne!(nan32, nan32_peer);
        assert!(nan32.partial_cmp(&1.0).is_none());
        assert_eq!((1.0_f32 / 0.0).to_bits(), 0x7f80_0000);
        assert_eq!((-1.0_f32 / 0.0).to_bits(), 0xff80_0000);
        assert_eq!((5.5_f32 % 2.0).to_bits(), 0x3fc0_0000);
        assert_eq!((-5.5_f32 % 2.0).to_bits(), 0xbfc0_0000);

        let nan64 = f64::from_bits(0x7ff8_0000_0000_0001);
        let nan64_peer = f64::from_bits(0x7ff8_0000_0000_0001);
        assert_eq!(0.0_f64, -0.0_f64);
        assert_ne!(nan64, nan64_peer);
        assert!(nan64.partial_cmp(&1.0).is_none());
        assert_eq!((1.0_f64 / 0.0).to_bits(), 0x7ff0_0000_0000_0000);
        assert_eq!((5.5_f64 % 2.0).to_bits(), 0x3ff8_0000_0000_0000);
    }

    #[test]
    fn widening_and_narrowing_cover_special_and_boundary_values() {
        assert_eq!(widen_f32_bits(0x0000_0001), 0x36a0_0000_0000_0000);
        assert_eq!(widen_f32_bits(0x3f80_0000), 0x3ff0_0000_0000_0000);
        assert_eq!(widen_f32_bits(0x7f80_0000), 0x7ff0_0000_0000_0000);
        assert_eq!(narrow_f64_bits(0x3ff0_0000_0000_0000), 0x3f80_0000);
        assert_eq!(narrow_f64_bits(0x47f0_0000_0000_0000), 0x7f80_0000);
        assert_eq!(narrow_f64_bits(0x36a0_0000_0000_0000), 0x0000_0001);
        assert_eq!(narrow_f64_bits(0x3690_0000_0000_0000), 0x0000_0000);
        assert_eq!(narrow_f64_bits(0xb690_0000_0000_0000), 0x8000_0000);
        assert_eq!(classify_f32(narrow_f64_bits(0x7ff8_0000_0000_0001)), "nan");
    }

    #[test]
    fn character_casts_enforce_unicode_scalar_boundaries() {
        assert_eq!(integer_to_char(0), Ok(0));
        assert_eq!(integer_to_char(0xd7ff), Ok(0xd7ff));
        assert_eq!(integer_to_char(0xd800), Err(Fault::InvalidCast));
        assert_eq!(integer_to_char(0xdfff), Err(Fault::InvalidCast));
        assert_eq!(integer_to_char(0xe000), Ok(0xe000));
        assert_eq!(integer_to_char(0x10ffff), Ok(0x10ffff));
        assert_eq!(integer_to_char(0x110000), Err(Fault::InvalidCast));
        assert_eq!(
            char_to_integer(IntegerType::U16, 0x10ffff),
            Err(Fault::InvalidCast)
        );
        assert_eq!(char_to_integer(IntegerType::U32, 0x10ffff), Ok(0x10ffff));
    }

    #[test]
    fn identity_table_has_twelve_accepted_and_twelve_rejected_judgments() {
        let cases = identity_cases();
        assert_eq!(cases.iter().filter(|case| case.expected).count(), 12);
        assert_eq!(cases.iter().filter(|case| !case.expected).count(), 12);
        for case in cases {
            assert_eq!(case.lhs == case.rhs, case.expected, "{}", case.name);
        }
    }

    #[test]
    fn coercion_table_has_twelve_accepted_and_fourteen_rejected_judgments() {
        let cases = coercion_cases();
        assert_eq!(
            cases.iter().filter(|case| case.expected.is_some()).count(),
            12
        );
        assert_eq!(
            cases.iter().filter(|case| case.expected.is_none()).count(),
            14
        );
        for case in cases {
            assert_eq!(
                coercion_rule(case.source, case.target),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn required_layout_rows_match_windows_x64_contract() {
        assert_eq!(
            array_layout(UNIT_LAYOUT, 0),
            Some(Layout {
                size: 0,
                alignment: 1
            })
        );
        assert_eq!(
            array_layout(UNIT_LAYOUT, 4),
            Some(Layout {
                size: 0,
                alignment: 1
            })
        );
        assert_eq!(
            array_layout(BYTE_LAYOUT, 0),
            Some(Layout {
                size: 0,
                alignment: 1
            })
        );
        let aligned_empty_array = Layout {
            size: 0,
            alignment: 8,
        };
        assert_eq!(array_layout(WORD64_LAYOUT, 0), Some(aligned_empty_array));
        assert_eq!(
            tuple_layout(&[aligned_empty_array]),
            Some((aligned_empty_array, vec![0]))
        );
        assert_eq!(
            tuple_layout(&[BYTE_LAYOUT, WORD64_LAYOUT, WORD16_LAYOUT]),
            Some((
                Layout {
                    size: 24,
                    alignment: 8
                },
                vec![0, 8, 16]
            ))
        );
        assert_eq!(
            STRING_LAYOUT,
            Layout {
                size: 24,
                alignment: 8
            }
        );
        assert_eq!(
            SLICE_REFERENCE_LAYOUT,
            Layout {
                size: 16,
                alignment: 8
            }
        );
    }

    #[test]
    fn diagnostics_and_evidence_rows_have_deterministic_order() {
        let ordered = deterministic_diagnostics(vec![
            Diagnostic {
                case_id: "b",
                code: "E2",
            },
            Diagnostic {
                case_id: "a",
                code: "E2",
            },
            Diagnostic {
                case_id: "a",
                code: "E1",
            },
        ]);
        assert_eq!(
            ordered,
            vec![
                Diagnostic {
                    case_id: "a",
                    code: "E1"
                },
                Diagnostic {
                    case_id: "a",
                    code: "E2"
                },
                Diagnostic {
                    case_id: "b",
                    code: "E2"
                },
            ]
        );
        let rows = semantic_inventory();
        assert!(rows.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn stable_digest_output_has_exactly_two_bounded_lines() {
        let rendered = output();
        let lines: Vec<_> = rendered.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("RLM-0002 semantic digest: fnv1a64:"));
        assert_eq!(lines[1], EVIDENCE_STATEMENT);
        assert!(rendered.len() < 160);
        assert_eq!(
            fnv1a64(&semantic_inventory()),
            fnv1a64(&semantic_inventory())
        );
    }
}
