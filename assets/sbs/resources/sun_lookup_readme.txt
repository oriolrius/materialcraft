Solar coefficient lookup

sun.exr is a user-supplied 512 by 16 RGBA32F representation of numeric solar
coefficients. Its metadata identifies the BSD-3-Clause data source as:
https://cgg.mff.cuni.cz/projects/SkylightModelling/HosekWilkie_SkylightModel_C_Source.1.4a.zip

All four channels are numeric data. Alpha is data, not opacity. Disable
color conversions and alpha premultiplication when sampling this lookup.
See LICENSE_HOSEK_WILKIE.txt for the original copyright and license notice.
