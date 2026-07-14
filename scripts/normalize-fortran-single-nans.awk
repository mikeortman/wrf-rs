# Canonicalize only IEEE-754 binary32 NaN encodings in the final column.
# Finite values, signed zero, and infinities remain byte-for-byte significant.
{
    bits = toupper($NF)
    if (bits ~ /^(7F|FF)[89ABCDEF][0-9A-F][0-9A-F][0-9A-F][0-9A-F][0-9A-F]$/ && \
        bits != "7F800000" && bits != "FF800000") {
        $NF = "7FC00000"
    }
    print
}
