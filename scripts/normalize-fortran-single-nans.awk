# Canonicalize every IEEE-754 binary32 NaN token in an oracle output line.
# Finite values, signed zero, and infinities remain byte-for-byte significant.
function is_binary32_nan(bits) {
    bits = toupper(bits)
    return bits ~ /^(7F|FF)[89ABCDEF][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]$/ && \
        bits != "7F800000" && bits != "FF800000"
}

{
    for (field_index = 1; field_index <= NF; field_index++) {
        if (is_binary32_nan($field_index)) {
            $field_index = "7FC00000"
        }
    }
    print
}
