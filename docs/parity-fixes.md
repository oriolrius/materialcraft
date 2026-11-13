# MaterialCraft parity fixes

Rolling record of fixes made while working through the [SBS root-node inventory](sbs-root-nodes.md).
Parity means exact pixel or typed-value agreement for recorded cases; pending validation is stated explicitly.
The first pass classifies all 546 roots: 166 Parity and 380 Too hard. These measured cases do not establish complete functional parity.

- **2026-11-13 — SBS swizzle defaults (orientation corrected):** Removed the importer's zero-channel overrides for omitted two-, three-, and four-component float/integer swizzle settings. Imports now use the existing identity-channel defaults from the function catalog. The first affected node, `3d_gradient.sbs/3d_linear_gradient`, previously repeated X instead of preserving XYZ, producing a horizontal gradient instead of the reference diagonal gradient. Compared both point-position modes with generated position maps at 256 and 512 pixels. Remaining float differences are recorded below; full node parity is not claimed.
- **2026-11-13 — Filtered function sampling (integer probes exact):** Added cooked-image storage precision to function samplers. Bilinear reads use eight fractional bits for filter weights. Normalized integer textures return 16-bit filtered values (including 8-bit sources); floating-point textures retain floating-point values. Input bridges retain the integer/floating-point distinction, and cache keys include it. Direct reference measurements on a generated position ramp showed a 16-bit half-step rounding to the next integer, explaining the remaining UV-gradient offset after the swizzle fix. All six measured integer-source sampler cases match all 65,536 float pixels exactly. Two of three floating-source cases are exact; one differs by 5.96e-8. Full-gradient rounding differences remain below.

- **2026-11-13 - SBS format inheritance (measured improvement):** Read and save the declared native graph output format instead of dropping it. An omitted format retains inheritance; comparison cases explicitly align the parent format to the reference preview (eight bits). Pixel Processors with omitted format settings now inherit the parent format instead of an image input format. This was exposed by `3d_perlin_noise.sbs/3D_perlin_noise`: the reference processor produces eight-bit output even though its position and lookup inputs are sixteen-bit. The Perlin comparison improved from 63,461/65,536 to 65,533/65,536 matching 8-bit pixels at 256 pixels; three additional 128-pixel cases match exactly. All four native format codes are serialized explicitly on save.

- **2026-11-13 - Floored floating-point modulo (randomized jump flood exact):** Replaced signed remainder with `a - b * floor(a / b)`. Direct reference probes confirm all sign combinations, including `-1 mod 4 = 3`, `1 mod -4 = -3`, and `-5.5 mod 4 = 2.5`. This corrects negative coordinate wrapping used by 3D texture jump flood. Both randomized 128x128 jump-flood cases (iterations 0 and 1, with disabled seed points) now match all 32,768 RGBA pixels exactly; the three ramp cases also match. All 131,072 RGBA pixels across five cases are exact.

- **2026-11-13 - Bound input storage and resizing:** Bound image inputs now store their declared output precision instead of passing unconverted floats downstream. Resizing clamps resource edges and proceeds through factor-two stored levels. A generated HDR 16x8 ramp resized to 32x32 now matches every RGBA8 input-bridge pixel; previously its center sample was 1.0 instead of 0.9843137. A randomized 64x32-to-32x32 downsample also matches every pixel. Three harder randomized multilevel cases retain single-level differences (4/1,024, 51/4,096, and 6/256 pixels), so general resampling parity is not claimed.

- **2026-11-13 - Half-float output storage:** Sixteen-bit floating-point outputs now convert to half precision, truncate toward zero, and saturate at +/-65,504. Previously they retained full 32-bit precision. All 2,304 pixels across nine direct reference probes match exactly, covering positive/negative values, rounding boundaries, a subnormal, maximum finite values, and positive/negative overflow. Uses the existing workspace half dependency (MIT OR Apache-2.0).

- **2026-11-13 - Volume-light function expansion:** Raised the bounded expanded-function allowance from 65,536 to 131,072 nodes after measuring the legitimate volume-light processor at 72,973 nodes and four call levels. The individual 4,096-node graph limit, 16-level call limit, cycle checks, and execution budget remain enforced. Error messages now report the actual limits. Three previously rejected light-render cases cook successfully; 760/768 RGBA16F pixels match, with the remaining differences recorded below.

- **2026-11-13 - Omitted function sampler settings:** Corrected Sample Color/Gray defaults to image zero with nearest filtering. Explicit bilinear settings are retained. Both original omitted-setting probes match all 32,768 float pixels. This removes the shifted edges of the 3D volume mask: default outputs at 128x128 and 256x256 and the changed-scale 128x128 case now match every sixteen-bit pixel.

- **2026-11-13 - Finite results after undefined pixel intermediates:** Pixel Processor arithmetic and vector operations now retain IEEE intermediate values so a later min/max can clamp division by zero to a finite result. Non-finite final pixels still return an actionable error. Parameter and pattern functions are checked at their public evaluation boundary. Four original clamp probes (positive infinity, negative infinity, NaN, and finite division) match all 1,024 float pixels. Box-mask cases previously failed at the exact sampled center; they now cook successfully, with the residual conversion differences recorded below.

- **2026-11-13 - Independent transform wrapping:** Transformation 2D now uses horizontal and vertical tiling flags independently, with matching per-axis image sampling. Previously a horizontal-only panorama lost horizontal wrapping because the two flags collapsed to one boolean. All 131,072 RGBA8 pixels across eight original transform probes match exactly, covering both one-axis modes, positive/negative offsets, matte edges, and varying alpha. The panorama helper improves, but its remaining minification/filtering differences are recorded below.

- **2026-11-13 - Blend alpha, copy, and HDR:** Corrected the Switch stages used by Copy and other graphs to interpolate RGBA without weighting foreground alpha twice. Copy, Add, and other color modes now combine alpha coverage when alpha blending is enabled; disabling alpha blending preserves background alpha while RGB still uses foreground coverage. Integer clipping happens after opacity; floating-point formats retain HDR output. All 19 independent blend graphs are being measured with opaque/varying-alpha inputs, masks, opacity changes, and alpha-channel blending disabled. Direct Add/Copy HDR cases match all 512 RGBA32F pixels. Fourteen blend graphs now match every pixel across four RGBA8 cases each (1,605,632 pixels); Add and Copy also pass their HDR cases. The blend properties label and all four alpha-interpretation menu entries were checked visually in the running native app.

- **2026-11-13 - Function validation at result boundaries:** Original clamp probes confirm that node-parameter and pattern functions also accept undefined intermediates when later operations produce a finite result. Deferred those intermediate checks and retained explicit final-result validation. The original four node-parameter and four pattern probes match all 2,048 output pixels; unclamped pixel and node-parameter probes still return clear non-finite-output errors. Ordered NaN comparisons now return false, as measured for all four relational operators. All 5,120 float pixels across the twenty NaN/infinity/finite/zero comparison cases match exactly.

- **2026-11-13 - Grayscale default precision:** Default grayscale rendering now uses one third per RGB channel instead of 0.333, which previously darkened equal channels. This makes Add Sub exact in all four RGBA8 cases and reduces Luminosity from 8,407 mismatching pixels to two. Original float probes establish the fused red/green/blue/alpha dot-product order; all 65,536 float pixels across default, custom, sum-three, and zero-weight cases match exactly. Explicit channel weights retain their authored values.

- **2026-11-13 - Declared image types:** Imported single-output filters with an omitted color-mode parameter now retain the color/grayscale type declared in their SBS output. The bevel debug helper previously dropped RGBA coordinate data into a grayscale pattern buffer. It now cooks in the correct four-channel format. Explicit settings and functions remain authoritative.

- **2026-11-13 - Pattern alpha and empty backgrounds:** Color pattern maps now start transparent. Scalar luminosity/opacity affects alpha as well as RGB, without multiplying RGB twice. Eight original empty/scalar/RGBA/HDR pattern probes match all 2,048 RGBA32F pixels across additive and both maximum blend modes. The existing White Noise result still matches every one of its 65,536 sixteen-bit pixels.

- **2026-11-13 - Color resources on grayscale input pins:** Bound color images now use equal RGB weights at grayscale input bridges. The general display-luminance utility retains its documented behavior. All nine eight-bit grayscale switch cases match 294,912 pixels; two additional float switch cases match all 32,768 pixels exactly, including varying source alpha. Previously only about one percent of the eight-bit pixels matched.

- **2026-11-13 - Transform matrix layout:** Transformation 2D now reads consecutive columns in the stored matrix and uses the same convention when selecting nearest mip levels. Previously rotations ran in the opposite direction and shears used the wrong axis. All 131,072 RGBA8 pixels across eight original shear/quarter-turn/diagonal/offset probes match exactly. Both unscaled rotated-checker cases also become exact. Scaled filtering remains open.

- **2026-11-13 - Legacy blend alpha interpretation:** Imported and implemented the older source-alpha/ignore-alpha/straight/premultiplied setting instead of dropping it. Direct runtime enum inspection confirms the four modes. Original Copy/Add/Max probes cover opacity changes, alpha-channel blending disabled, and 128x128/256x256 outputs. Ignore-alpha controls RGB weighting independently of output coverage; straight compositing includes background coverage, while premultiplied Copy consumes premultiplied foreground RGB. All four Clamp cases now match every one of 114,688 RGBA8 pixels. All 48 direct Copy/Add/Max cases cook; seven of the twelve mode/interpretation combinations match all 114,688 pixels, while the other five retain 11 one-level differences in total. Straight HDR output remains unclipped; its float differences reach 3.577e-7.

- **2026-11-13 - Integer options read as Boolean:** A legacy color-temperature function reads an integer option through Get Boolean. Original probes confirm that zero is false and positive/negative integers, including the minimum 32-bit value, are true. Added that conversion for user integer/enum variables while retaining strict system-variable and other getter types. All 1,280 float pixels across the five direct cases match exactly. Color temperature now cooks in all four cases; only two one-level pixels remain different.

- **2026-11-13 - Internal images down to one texel:** Removed the evaluator's 16-pixel floor for internal node sizes. The graph editor retains its own graph-size setting range. Internal reductions and lookup images can now use their declared 1..8,192 dimensions. Direct 1x1, 2x4, and 8x2 RGBA32F probes match all 25 pixels. The curvature helper's four small cases now use the correct 2x2/4x4 outputs and match all 28 pixels; two larger nonconstant cases also match all 16,640 RGBA32F pixels. All 655,360 pixels across 22 earlier Perlin-fractal, simplex, jump-flood, angle-vector, checker, and live White Noise cases still match exactly.

- **2026-11-13 - Integer Pixel Processor storage:** Original eight- and sixteen-bit probes confirm NaN converts to black, positive infinity to white, and negative infinity to black. Deferred Pixel Processor finite checks to integer storage for those formats, retaining final floating-point output validation. Integer storage now saturates infinities instead of treating every non-finite value as zero. All 2,560 pixels across ten direct integer probes match exactly. The zero-softness color mask and radial gradient now cook in all four cases each, with only one-level residual differences. Unclamped floating-point Pixel Processors still return a non-finite-output error.

- **2026-11-13 - Swizzle padding:** Vector swizzles now read components outside the source dimension as zero, including scalar inputs. Arithmetic scalar broadcasting remains available to arithmetic operations. All 1,792 RGBA32F pixels across seven original float/integer probes match exactly. Histogram computation now cooks in all four cases; its remaining numerical differences are recorded below.

- **2026-11-13 - Grayscale resources on color input pins:** Color input bridges now promote grayscale resources to RGB with opaque alpha. Both original 128x128 and 256x256 probes match all 81,920 RGBA8 pixels exactly.

- **2026-11-13 - Unassigned user variables:** Numeric user-variable getters now return typed zero, and Boolean getters return false, before the first assignment. Previously they raised an undefined-variable error. Original probes add a nonzero value after each getter so a failed black render cannot look like a match. All 2,304 float/RGBA32F pixels across Boolean, integer1..4, and float1..4 cases match exactly. System-variable availability, existing-variable type checks, and string errors remain strict. All four affected Voronoi graphs now cook across four 16x16/32x32 cases each; numerical differences remain. The 128x128 fractal case also cooks and matches 15,457/16,384 pixels. The previously failing matte and shadow helper now cook in all four cases each, with remaining output differences.

- **2026-11-13 - RGBA results in grayscale Pixel Processors:** Grayscale processors now convert a four-component result with equal RGB weights instead of taking red alone. Alpha does not contribute. Original HDR and negative-value probes establish the fused green/red/blue accumulation order; all 9,216 LUM32F pixels across 36 cases match exactly. The three affected grayscale rendering templates now match all 344,064 LUM8 pixels across twelve cases; the color template retains all 114,688 exact RGBA8 pixels.

- **2026-11-13 - Float-to-integer conversion:** Finite scalar and vector conversion now rounds down, correcting negative coordinates that previously truncated toward zero. All 6,144 float/RGBA32F pixels and 18 typed integer outputs match the original reference probes. Non-finite and out-of-range integer conversions retain actionable errors; parity for those inputs is not claimed. The 22 previously verified cases still match all 655,360 pixels after this change.

- **2026-11-13 - Unconnected output images:** Unconnected output bridges now produce transparent RGBA instead of grayscale. The importer separately retains each output's declared image type so saving an unconnected output preserves the graph interface. Both material templates match all 1,179,648 RGBA8 pixels across six recorded cases. All 2,560 direct grayscale/color declaration probe pixels also match, and both declared image types survive save/reload.

## Remaining differences

- **`3d_gradient.sbs/3d_linear_gradient` - Too hard for this pass:** Corrected the wrong channel mapping and normalized-integer bilinear sampling offset. Compared generated position maps in UV and world modes, with zero and nonzero contrast, at 256 and 512 pixels (eight cases). Remaining 32-bit differences reach 7.153e-7; some values straddle a 16-bit rounding boundary by one level. The default world-mode case matches every pixel after 16-bit conversion at both sizes, but full float parity is not established. Further work needs matching the reference shader arithmetic rounding. An unconnected Position input yields NaNs in the reference app; MaterialCraft retains its actionable non-finite arithmetic error. Moving to the next node.

- **`3d_perlin_noise.sbs/3D_perlin_noise` - Too hard for this pass:** Exact at 128x128 for defaults, seed 42, and scale 6. At 256x256, 3 of 65,536 eight-bit pixels remain one level different: (x=246,y=155), (25,179), and (154,227). Input maps are generated linear XY ramps with Z=128/255 and alpha=1. The output-format error is fixed; residual arithmetic/conversion rounding still prevents a parity mark.

- **`3d_perlin_noise.sbs/3D_ridged_noise_fractal` - Too hard for this pass:** Default and roughness-0.7 cases at 128x128 are exact. The default 256x256 case has two differing eight-bit pixels; seed 42 at 128x128 has one. Maximum difference is one level. Full parity is not claimed.

- **`3d_planar_projection.sbs/3d_planar_projection` - Too hard for this pass:** With generated position, normal, and high-frequency grayscale texture inputs, 16,240/16,384 sixteen-bit pixels match at 128x128 and 65,048/65,536 at 256x256. Maximum differences are 125 and 128 levels respectively, concentrated at texture sampling boundaries. Independent sampler half-step and nearest-read probes agree; remaining projection arithmetic is a likely contributor.

- **`3d_planar_projection.sbs/3d_planar_projection_color` - Too hard for this pass:** With generated position, normal, and RGBA texture inputs, 16,370/16,384 pixels match at 128x128 and 65,498/65,536 at 256x256. All four eight-bit channels, including alpha, were compared. Remaining differences are at most one level.

- **`3d_texture_render.sbs/3d_texture_surface_render_irradiance` - Too hard for this pass:** Eight 16x16 RGBA32F cases use original position/normal maps, constant positive HDR distance fields, directional sample tables, and a colored environment ramp. Four nonzero lighting cases (iteration 0, 1, 16, and 128) differ by at most 8.941e-8 in RGB; alpha is exact. Four opposite-hemisphere cases match exactly but produce black lighting. All cooks succeed. Full float parity needs matching shader arithmetic rounding; black-image agreement alone is not used for a parity mark.

- **`3d_texture_render.sbs/3d_texture_volume_render_ibl_pass` - Too hard for this pass:** Six 16x16 RGBA16F cases use original position/noise/environment/sample maps and constant negative distance fields, with dense and thin media, iteration changes, and absorption changes. Thin-media cases produce visible nonzero lighting but only 9/256, 10/256, and 4/256 pixels match exactly; maximum component differences are 0.006348, 0.005372, and 0.011719. Three additional thin-media cases bind an original 256x256 noise map at the required input size; they still differ, reaching 0.012208. All cooks succeed. The volume-integration discrepancy needs further isolation; half-float storage is separately verified.

- **`3d_texture_render.sbs/3d_texture_volume_render_light_pass` - Too hard for this pass:** Fixed the function-expansion rejection. All three 16x16 thin-media cases now cook successfully, including changed light position/intensity and changed albedo. Exact RGBA16F pixels: 252/256, 254/256, and 254/256. Maximum component differences are 6.104e-5, 3.052e-5, and 1.526e-5. Eight pixels in total prevent a parity mark.

- **`3d_texture_render.sbs/3d_texture_surface_render_position` - Too hard for this pass:** All four initial 16x16 RGBA8 cases match exactly (solid/positive fields, ground plane, changed camera). Solid-field 64x64 and ground-plane 128x128 cases also match exactly. A 64x64 case with a changed camera direction and 65-degree field of view has four differing pixels, at most two levels apart. All cooks succeed. Full camera/geometry parity is not claimed.

- **`3d_texture_render.sbs/position_debug` - Too hard for this pass:** Initial zero/positive/midpoint fields and checker fields produce constant images and are insufficient for a parity mark. A generated signed sphere-distance atlas and original position map produce nonconstant 256x256 RGBA32F outputs. Center and changed debug-selection cases each match 65,519/65,536 pixels; 17 pixels per case differ by up to 1.0. Both internal 4096x4096 processors cook at full native size.

- **`3d_volume_mask.sbs/3d_volume_mask` - Too hard for this pass:** Corrected omitted nearest sampling and finite clamping after division by zero. Default 128x128, default 256x256, and changed-scale 128x128 outputs are exact (98,304 sixteen-bit pixels). Box mode has four one-level differences; rotated, resized, narrow-feather box mode has 58 one-level differences. All five cases cook successfully.

- **`3d_worley_noise.sbs/3D_worley_noise` - Too hard for this pass:** All cooks succeed, but the noise field differs substantially. Defaults at 128x128/256x256, seed 42, scale 6, and border style have only 1, 3, 1, 2, and 1 matching sixteen-bit pixels respectively; maximum difference is 44,494 levels. Changing omitted sampler defaults does not resolve this.

- **Pixel Processor random stream - Unresolved:** Three original scalar-random probes (16x16 seeds 0/42 and 32x32 seed 0) disagree at all 1,536 float pixels. The reference stream also depends on output resolution. No guessed replacement sequence was introduced. A causal link to the remaining noise or volume-lighting differences is not established.

- **`alveolus.sbs/alveolus` - Too hard for this pass:** Defaults at 128x128/256x256 and seed 42 have no exactly matching sixteen-bit pixels. Gradient-filled cells with interstice width 0.1 match 13,880/16,384 pixels, but differ by up to 65,535 levels. All four cooks succeed.

- **`anisotropic_blur.sbs/anisotropic_blur` and `anisotropic_blur_grayscale` - Too hard for this pass:** Compared default 128x128, rotated high-anisotropy 128x128, and quality/intensity changes at 256x256. All cooks succeed. Color matches 224/16,384, 109/16,384, and 977/65,536 RGBA8 pixels; grayscale matches 252/16,384, 125/16,384, and 1,024/65,536 pixels. Maximum difference reaches 87 eight-bit levels. Blur sampling/kernel parity remains open.

- **`atlas_spliter.sbs/crop_and_transform_sub` - Too hard for this pass:** Its two typed value pins are outside the reference graph's exposed-parameter API. The direct comparison harness cannot set shape count/table size through that API. A future comparison needs an original wrapper that supplies these typed value inputs; the regular image-pin bindings are available.

- **`3d_viewer.sbs/ibl_specular_contrib_anisotropic/panoramic_lod` - Too hard for this pass:** Corrected one-axis wrapping. For 128x128 and 256x256 opaque color inputs, exact RGBA8 pixels improve from 576/16,384 and 4,391/65,536 to 719/16,384 and 5,015/65,536. Maximum component differences drop from 252/254 to 128/127. A varying-alpha 128x128 case still has no fully matching pixels and a maximum difference of 239. All cooks succeed. Input resizing and transform minification/filtering remain open.

- **`bevel_smooth.sbs/blur_gaussian_grayscale` - Too hard for this pass:** Defaults at 128x128 and 256x256 are exact but perform no visible blur at these settings. Intensity 5 at 128x128 matches only 158/16,384 eight-bit pixels and differs by up to 23 levels. All cooks succeed; nonzero-kernel parity remains open.

- **`bevel_smooth.sbs/jump_flood_segment_iteration` - Too hard for this pass:** All cooks succeed. Default 128x128, iteration 1, changed distance mode/scale, and default 256x256 match 16,366/16,384, 16,376/16,384, 16,379/16,384, and 65,466/65,536 RGBA8 pixels. A total of 101 segment selections differ; maximum component difference is 192. Input segment endpoints are original synthetic color fields.

- **`bevel_smooth.sbs/debug/bevel_smooth_debug_position` - Too hard for this pass:** Corrected undefined function intermediates, ordered NaN comparisons, omitted color-mode imports, and pattern alpha. All four cases now cook successfully in RGBA32F. Default bounded-position maps at 128x128/256x256 match 16,359/16,384 and 65,415/65,536 pixels. Offset and pixel-selection cases match 14,326/16,384 and 15,362/16,384. Remaining stamp placement/coverage differences reach 19.165; a parity mark is withheld.

- **`blackbody.sbs/blackbody` - Too hard for this pass:** Compared the default 3,000 K at 128x128 and 1,000/6,500/12,000 K at 16x16, retaining RGBA32F HDR output. Alpha is exact, but no complete RGBA pixel matches; RGB differences reach 4.769e-7. All cooks succeed.

- **Remaining blend cases:** All four RGBA8 cases cook for every graph. `blend.sbs/color` matches 114,111/114,688 pixels, with differences up to nine levels. `color_burn` and `color_dodge` each have seven differing pixels, reaching 16 and 144 levels; these include degenerate black/white endpoints and one-level arithmetic differences. `dissolve_2` matches 75,035/114,688 pixels, with stochastic differences up to 254 levels. After the grayscale default fix, `luminosity` has two one-level differences; the fused-dot recheck retains those two differences. These graphs are not marked parity.

- **`blur_hq.sbs/blur_hq` and `blur_hq_grayscale` - Too hard for this pass:** All four cases cook. Intensity 1 at 128x128 is exact for both output types but has no visible blur at that scale. Default 128x128/256x256 and high-quality intensity-5 color cases match 2/16,384, 0/65,536, and 12/16,384 RGBA8 pixels; grayscale matches 125/16,384, 15,168/65,536, and 144/16,384 pixels. Differences reach 52 color levels and 30 grayscale levels. Directional blur kernel/sampling remains open.

- **`bnw_spots_1.sbs/bnw_spots_1` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and changed parameters: 0/16384, 5/65536, 0/16384, 0/16384. Maximum component difference is 62677 integer levels. Pattern/noise behavior remains open.

- **`bnw_spots_3.sbs/bnw_spots_3` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and changed parameters: 0/16384, 0/65536, 0/16384, 0/16384. Maximum component difference is 63995 integer levels. Pattern/noise behavior remains open.

- **`brick_1.sbs/brick_1` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and changed parameters: 14992/16384, 59968/65536, 14992/16384, 8352/16384. Maximum component difference is 65535 integer levels. Pattern/noise behavior remains open.

- **`brick_2.sbs/brick_2` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and changed parameters: 15264/16384, 61360/65536, 15264/16384, 8896/16384. Maximum component difference is 65535 integer levels. Pattern/noise behavior remains open.

- **`brick_3.sbs/brick_3` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and changed parameters: 4616/16384, 24922/65536, 5408/16384, 2880/16384. Maximum component difference is 65535 integer levels. Pattern/noise behavior remains open.

- **`cartesian_polar_transformation.sbs/cartesian_to_polar` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and a changed input: 14130/16384, 52081/65536, 14130/16384, 14081/16384. Maximum component difference is 4 integer levels. Polar sampling/coordinate precision remains open.

- **`cartesian_polar_transformation.sbs/polar_to_cartesian` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and a changed input: 15330/16384, 61564/65536, 15330/16384, 15842/16384. Maximum component difference is 1 integer levels. Polar sampling/coordinate precision remains open.

- **`cartesian_polar_transformation.sbs/cartesian_to_polar_grayscale` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and a changed input: 15216/16384, 58474/65536, 15216/16384, 15281/16384. Maximum component difference is 4 integer levels. Polar sampling/coordinate precision remains open.

- **`cartesian_polar_transformation.sbs/polar_to_cartesian_grayscale` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and a changed input: 15804/16384, 63310/65536, 15804/16384, 15794/16384. Maximum component difference is 1 integer levels. Polar sampling/coordinate precision remains open.

- **`cells_1.sbs/cells_1` - Too hard for this pass:** All four cases cook. Exact pixels for defaults at 128x128/256x256, seed 42, and changed parameters: 5/16384, 12/65536, 1/16384, 2/16384. Maximum component difference is 64385 integer levels. Pattern/noise behavior remains open.

- **`channel_mixer.sbs/channel_mixer` - Too hard for this pass:** Default 128x128/256x256 and seed-42 cases fail final validation of the dynamic `in_high` parameter. The changed-channel, monochrome case cooks but matches 0/16384 RGBA16 pixels, with differences up to 62259 levels. Parameter endpoint semantics and channel mixing remain open.

- **`chrominance_extract.sbs/chrominance_extract` - Too hard for this pass:** All cases cook. Exact pixels in case order: 10557/16384, 42010/65536, 10557/16384, 10515/16384; maximum component difference 1 integer levels.

- **`clone.sbs/clone` - Too hard for this pass:** All cases cook. Exact pixels in case order: 16384/16384, 65536/65536, 16384/16384, 16155/16384; maximum component difference 1 integer levels.

- **`clouds_1.sbs/clouds_1` - Too hard for this pass:** All cases cook. Exact pixels in case order: 1/16384, 12/65536, 4/16384, 3/16384; maximum component difference 51526 integer levels.

- **`clouds_2.sbs/clouds_2` - Too hard for this pass:** All cases cook. Exact pixels in case order: 1/16384, 2/65536, 1/16384; maximum component difference 42679 integer levels.

- **`color_space_conversion.sbs/acescg_to_linear_srgb` - Too hard for this pass:** All cases cook. Exact pixels in case order: 16384/16384, 65536/65536, 16384/16384, 16383/16384; maximum component difference 1 integer levels.

- **`color_space_conversion.sbs/srgb_to_acescg` - Too hard for this pass:** All cases cook. Exact pixels in case order: 16383/16384, 65534/65536, 16383/16384, 16382/16384; maximum component difference 1 integer levels.

- **`color_to_mask.sbs/color_to_mask` - Too hard for this pass:** All cases cook. Exact pixels in case order: 16384/16384, 65536/65536, 16384/16384, 16293/16384; maximum component difference 255 integer levels.

- **`color_to_mask_v2.sbs/color_to_mask` - Too hard for this pass:** Corrected integer storage after undefined final pixel arithmetic. All four cases now cook. Defaults at 128x128/256x256 and seed 42 match exactly. Changed input-color selection has one one-level difference (16,383/16,384 pixels match).

- **`checker_1.sbs/checker_1` - Too hard for this pass:** Corrected the transposed transform matrix. Defaults at 128x128/256x256, seed 42, and rotated tiling 1/2 now match exactly. Tiling 3 still has minification differences: 14,400/16,384 pixels without rotation and 13,476/16,384 with rotation; maxima 56 and 119 eight-bit levels. A parity mark is withheld.

- **`constant_values.sbs/integer/constant_integer4` - Too hard for this pass:** Zero and mixed signed-vector cases match. At the positive 32-bit integer limit, the reference returns -2,147,483,648 for a component set to 2,147,483,647; MaterialCraft retains 2,147,483,647. No overflow behavior was introduced to imitate this boundary result.

- **`color_temperature_adjustment.sbs/color_temperature_adjustment` - Too hard for this pass:** Fixed the integer option read that previously rejected every cook. Default 128x128/256x256 and seed-42 cases are exact. Temperature 0.25, magenta/green -0.2, and alternate color space at 128x128 have two one-level RGBA8 differences. All four cases cook.

- **`composite.sbs/composite/filters/filter_transformation` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16320/16384, 65280/65536, 16320/16384, 16318/16384; maximum component difference 255 integer levels.

- **`composite.sbs/composite/blends/blend_content` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16288/16384, 65152/65536, 16288/16384, 16288/16384; maximum component difference 255 integer levels.

- **`composite.sbs/composite/blends/blend_content_pre_post_composite` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16288/16384, 65152/65536, 16288/16384, 16287/16384; maximum component difference 255 integer levels.

- **`creased.sbs/creased` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum component difference 55711 integer levels.

- **`crop.sbs/crop` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16340/16384, 16384/16384, 65350/65536, 65536/65536, 16340/16384, 16384/16384, 16384/16384, 16384/16384; maximum component difference 1 integer levels.

- **`crop.sbs/crop_grayscale` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16340/16384, 16384/16384, 65350/65536, 65536/65536, 16340/16384, 16384/16384, 16384/16384, 16384/16384; maximum component difference 1 integer levels.

- **`crystal_1.sbs/crystal_1` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 2/16384, 5/65536, 2/16384, 3/16384; maximum component difference 64433 integer levels.

- **`crystal_2.sbs/crystal_2` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 1/16384, 2/65536, 0/16384, 1/16384; maximum component difference 55976 integer levels.

- **`curvature.sbs/curvature` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 31/16384, 660/65536, 31/16384, 20/16384; maximum component difference 962 integer levels. Inputs include an original curved, normalized normal field.

- **`curvature_smooth.sbs/curvature_smooth` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 23/16384, 167/65536, 23/16384, 25/16384; maximum component difference 1602 integer levels. Inputs include an original curved, normalized normal field.

- **`diffusion.sbs/impl/pass_normal` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 16383/16384, 65527/65536, 16383/16384, 1304/16384; maximum component difference 94 integer levels.

- **`diffusion.sbs/impl/pass_RGBA` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 15919/16384, 63936/65536, 15919/16384, 458/16384; maximum component difference 154 integer levels.

- **`diffusion.sbs/impl/pass_gray` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 16178/16384, 64736/65536, 16178/16384, 2192/16384; maximum component difference 104 integer levels.

- **`diffusion.sbs/impl/pass_UV` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 9704/16384, 40270/65536, 9720/16384, 9704/16384; maximum component difference 192 integer levels.

- **`directional_distance.sbs/directional_distance` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 16378/16384, 16384/16384, 65510/65536, 65536/65536, 16378/16384, 16384/16384, 16384/16384, 16384/16384; maximum component difference 1 integer levels. All output bridges were compared.

- **`directionnal_noise.sbs/directionnal_noise` - Too hard for this pass:** All 3 cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384; maximum component difference 60592 integer levels.

- **`dirt_1.sbs/dirt_1` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 79/16384, 334/65536, 55/16384, 100/16384; maximum component difference 65535 integer levels.

- **`dirt_3.sbs/dirt_3` - Too hard for this pass:** All 4 cases cook. Exact pixels by case/output: 6/16384, 546/65536, 2/16384, 5/16384; maximum component difference 64872 integer levels.

- **`dirt_5.sbs/dirt_5` - Too hard for this pass:** All 3 cases cook. Exact pixels by case/output: 2514/16384, 9716/65536, 2587/16384; maximum component difference 65535 integer levels.

- **`dirt_6.sbs/dirt_6` - Too hard for this pass:** All 3 cases cook. Exact pixels by case/output: 51/16384, 188/65536, 60/16384; maximum component difference 64915 integer levels.

- **`dirt_gradient.sbs/dirt_gradient` - Too hard for this pass:** All 3 cases cook. Exact pixels by case/output: 0/16384, 1/65536, 1/16384; maximum component difference 65399 integer levels.

- **`fibers_1.sbs/fibers_1` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum component difference 48873 integer levels.

- **`flood_fill.sbs/flood_fill_to_random_color` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 65071 integer levels.

- **`flood_fill.sbs/flood_fill_to_random_grayscale` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 8/65536, 0/16384, 0/16384; maximum component difference 64844 integer levels.

- **`flood_fill_2.sbs/flood_fill_to_random_color` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 65071 integer levels.

- **`flood_fill_2.sbs/flood_fill_to_random_grayscale` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 65169 integer levels.

- **`flood_fill_2.sbs/flood_fill_to_color` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 0/16384; maximum component difference 61956 integer levels.

- **`flood_fill_2.sbs/flood_fill_to_grayscale` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 2/16384; maximum component difference 5503 integer levels.

- **`fluid.sbs/fluid` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 65454 integer levels.

- **`fractal_sum_base.sbs/fractal_sum_base` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 3/16384, 8/65536, 5/16384, 5/16384; maximum component difference 10237 integer levels.

- **`fur_1.sbs/fur_1` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 23/16384, 193/65536, 30/16384, 25/16384; maximum component difference 65535 integer levels.

- **`fur_2.sbs/fur_2` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 65523 integer levels.

- **`fur_3.sbs/fur_3` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum component difference 65535 integer levels.

- **`fur_4.sbs/fur_4` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 1134/16384, 4416/65536, 1205/16384, 1200/16384; maximum component difference 65535 integer levels.

- **`gaussian_1.sbs/gaussian_1` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 40180 integer levels.

- **`gaussian_2.sbs/gaussian_2` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum component difference 23969 integer levels.

- **`gaussian_spots_1.sbs/gaussian_spots_1` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 4/16384, 15/65536, 7/16384, 4/16384; maximum component difference 63216 integer levels.

- **`gaussian_spots_2.sbs/gaussian_spots_2` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 3/16384, 16/65536, 2/16384, 3/16384; maximum component difference 65535 integer levels.

- **`gradient_axial.sbs/gradient_axial` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16379/16384; maximum component difference 1 integer levels.

- **`gradient_axial.sbs/gradient_axial_reflected` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16366/16384; maximum component difference 1 integer levels.

- **`gradient_circular.sbs/gradient_circular` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 5531/16384, 21951/65536, 5531/16384, 5591/16384; maximum component difference 4 integer levels.

- **`gradient_linear_1.sbs/gradient_linear_1` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 15456/16384; maximum component difference 24576 integer levels.

- **`gradient_linear_2.sbs/gradient_linear_2` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 0/16384; maximum component difference 1984 integer levels.

- **`gradient_linear_3.sbs/gradient_linear_3` - Too hard for this pass:** All four cases cook. Exact pixels by case/output: 15495/16384, 61975/65536, 15495/16384, 4096/16384; maximum component difference 1830 integer levels.

- **`gradient_radial.sbs/gradient_radial` - Too hard for this pass:** Corrected integer storage so the changed shape/points case cooks instead of failing on a non-finite pixel. All four cases now cook and match 114,624/114,688 sixteen-bit pixels in total; 64 one-level differences remain.

- **`hald_clut.sbs/hald_clut` - Too hard for this pass:** Exact pixels by case/output: 16118/16384, 64478/65536, 16118/16384, 15905/16384; maximum stored-component difference 1.

- **`hbao.sbs/hbao_mip` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 22/16384; maximum stored-component difference 24094.

- **`hbao_2.sbs/hbao_subsample` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16377/16384; maximum stored-component difference 32.

- **`hbao_2.sbs/hbao_mip` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 143/16384; maximum stored-component difference 94.

- **`hdr_merge.sbs/average_luminance_merge` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 1/16384; maximum stored-component difference 26228.

- **`hdr_range_viewer.sbs/hdr_range_viewer_grayscale` - Too hard for this pass:** Exact pixels by case/output: 14532/16384, 58173/65536, 14532/16384, 11670/16384; maximum stored-component difference 1.

- **`height_blend.sbs/height_blend` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 16384/16384, 65536/65536, 65536/65536, 16384/16384, 16384/16384, 7573/16384, 8768/16384; maximum stored-component difference 23426.

- **`height_to_normal_world_units.sbs/height_to_normal_world_units` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 10198/16384; maximum stored-component difference 50.

- **`height_to_normal_world_units.sbs/height_to_normal_world_units_2` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 10198/16384; maximum stored-component difference 49.

- **`highpass.sbs/highpass` - Too hard for this pass:** Exact pixels by case/output: 223/16384, 556/65536, 223/16384, 6967/16384; maximum stored-component difference 59.

- **`highpass.sbs/highpass_grayscale` - Too hard for this pass:** Exact pixels by case/output: 223/16384, 3712/65536, 223/16384, 9926/16384; maximum stored-component difference 66.

- **`histogram_scan.sbs/histogram_scan` - Too hard for this pass:** Exact pixels by case/output: 16319/16384, 65280/65536, 16319/16384, 16384/16384; maximum stored-component difference 255.

- **`histogram_shift.sbs/histogram_shift` - Too hard for this pass:** Exact pixels by case/output: 16319/16384, 65280/65536, 16319/16384, 15935/16384; maximum stored-component difference 1.

- **`histogram_tools.sbs/histogram_compute` - Too hard for this pass:** Exact pixels by case/output: 4/256, 54/256, 1/256, 256/256, 4/256, 54/256, 1/128, 0/128; maximum stored-component difference 7.

- **`material_transforms.sbs/material_transforms` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 65536/65536, 65536/65536, 65536/65536, 65536/65536, 65536/65536, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 14559/16384, 13711/16384, 15603/16384, 15493/16384, 12192/16384; maximum stored-component difference 1.

- **`me_water_level.sbs/scratches_2` - Too hard for this pass:** Exact pixels by case/output: 3/16384, 29/65536, 2/16384, 9/16384; maximum stored-component difference 103.

- **`mean_of_least_variance.sbs/mlv_color_iteration` - Too hard for this pass:** Exact pixels by case/output: 4/16384, 0/65536, 4/16384, 658/16384; maximum stored-component difference 167.

- **`mean_of_least_variance.sbs/mlv_grayscale_iteration` - Too hard for this pass:** Exact pixels by case/output: 6292/16384, 33605/65536, 6292/16384, 1762/16384; maximum stored-component difference 88.

- **`median_filter.sbs/median_filter_color` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 12077/16384; maximum stored-component difference 236.

- **`mesh_1.sbs/mesh_1` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 16/16384; maximum stored-component difference 65535.

- **`mesh_2.sbs/mesh_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 160/16384; maximum stored-component difference 65535.

- **`mesh_adaptive_material_blender.sbs/mesh_material_blender_8_inputs` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 16384/16384, 89/16384, 65536/65536, 65536/65536, 345/65536, 16384/16384, 16384/16384, 89/16384, 3157/16384, 14385/16384, 76/16384; maximum stored-component difference 255.

- **`mesh_adaptive_material_blender.sbs/mesh_material_blender_4_inputs` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 16384/16384, 86/16384, 65536/65536, 65536/65536, 358/65536, 16384/16384, 16384/16384, 86/16384, 1595/16384, 14388/16384, 74/16384; maximum stored-component difference 255.

- **`mesh_adaptive_material_blender.sbs/mesh_material_blender_2_inputs` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 16384/16384, 20/16384, 65536/65536, 65536/65536, 81/65536, 16384/16384, 16384/16384, 20/16384, 231/16384, 2761/16384, 12/16384; maximum stored-component difference 255.

- **`messy_fibers_1.sbs/messy_fibers_1` - Too hard for this pass:** Exact pixels by case/output: 173/16384, 1893/65536, 145/16384, 104/16384; maximum stored-component difference 65535.

- **`messy_fibers_2.sbs/messy_fibers_2` - Too hard for this pass:** Exact pixels by case/output: 34/16384, 165/65536, 53/16384, 46/16384; maximum stored-component difference 65535.

- **`messy_fibers_3.sbs/messy_fibers_3` - Too hard for this pass:** Exact pixels by case/output: 6383/16384, 25656/65536, 6157/16384, 6471/16384; maximum stored-component difference 255.

- **`messy_fibers_4.sbs/messy_fibers_4` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384; maximum stored-component difference 64037.

- **`microscope_view.sbs/microscope_view` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 23/65536, 0/16384, 1/16384; maximum stored-component difference 65535.

- **`moisture_noise.sbs/moisture_noise` - Too hard for this pass:** Exact pixels by case/output: 7/16384, 34/65536, 3/16384, 10/16384; maximum stored-component difference 45639.

- **`mosaic.sbs/mosaic` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 190.

- **`mosaic.sbs/mosaic_grayscale` - Too hard for this pass:** Exact pixels by case/output: 198/16384, 630/65536, 198/16384, 258/16384; maximum stored-component difference 143.

- **`multi_directional_warp.sbs/multi_directional_warp_grayscale` - Too hard for this pass:** Exact pixels by case/output: 121/16384, 3395/65536, 121/16384, 9394/16384; maximum stored-component difference 158.

- **`multi_directional_warp.sbs/multi_directional_warp_color` - Too hard for this pass:** Exact pixels by case/output: 66/16384, 286/65536, 66/16384, 1215/16384; maximum stored-component difference 250.

- **`multiangle_to_albedo.sbs/multiangle_to_albedo` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65534/65536, 16384/16384, 16384/16384; maximum stored-component difference 1.

- **`noise_anisotropic_noise.sbs/anisotropic_noise` - Too hard for this pass:** Exact pixels by case/output: 5/16384, 25/65536, 8/16384, 3612/16384; maximum stored-component difference 65214.

- **`noise_anisotropic_noise_v2.sbs/anisotropic_noise_v2` - Too hard for this pass:** Exact pixels by case/output: 5422/16384, 14285/65536, 5547/16384, 13676/16384; maximum stored-component difference 15.

- **`min_max.sbs/min_max` - Too hard for this pass:** All four cases cook and seven of eight scalar outputs match exactly. The alternate-input minimum differs by 5.9604645e-8 (0.1254901885986328 versus 0.12549012899398804).

- **`make_it_tile_patch.sbs/make_it_tile_patch` - Too hard for this pass:** Exact pixels by case/output: 596/16384, 2423/65536, 596/16384, 41/16384; maximum stored-component difference 255.

- **`make_it_tile_patch_grayscale.sbs/make_it_tile_patch_grayscale` - Too hard for this pass:** Exact pixels by case/output: 937/16384, 5840/65536, 937/16384, 90/16384; maximum stored-component difference 252.

- **`non_square_transform.sbs/non_square_transform` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 13939/16384; maximum stored-component difference 2.

- **`non_square_transform.sbs/non_square_transform_greyscale` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 15078/16384; maximum stored-component difference 2.

- **`non_uniform_blur.sbs/non_uniform_blur_sample` - Too hard for this pass:** Exact pixels by case/output: 8397/16384, 25549/65536, 8397/16384, 3980/16384; maximum stored-component difference 4.

- **`non_uniform_blur.sbs/non_uniform_blur_grayscale_sample` - Too hard for this pass:** Exact pixels by case/output: 13146/16384, 52479/65536, 13146/16384, 8120/16384; maximum stored-component difference 4.

- **`non_uniform_rotation.sbs/non_uniform_rotation_grayscale` - Too hard for this pass:** Exact pixels by case/output: 15829/16384, 63348/65536, 15829/16384, 15865/16384; maximum stored-component difference 1.

- **`non_uniform_rotation.sbs/non_uniform_rotation_color` - Too hard for this pass:** Exact pixels by case/output: 15888/16384, 63498/65536, 15888/16384, 15885/16384; maximum stored-component difference 2.

- **`normal_normalise.sbs/normal_normalise` - Too hard for this pass:** Exact pixels by case/output: 16356/16384, 65420/65536, 16356/16384, 16356/16384; maximum stored-component difference 1.

- **`normal_sobel.sbs/normal_sobel` - Too hard for this pass:** Exact pixels by case/output: 10198/16384, 40960/65536, 10198/16384, 10198/16384; maximum stored-component difference 2399.

- **`normal_sobel.sbs/normal_sobel_2` - Too hard for this pass:** Exact pixels by case/output: 10198/16384, 40960/65536, 10198/16384, 10198/16384; maximum stored-component difference 9.

- **`normal_to_height_2.sbs/normal_to_height` - Too hard for this pass:** Exact pixels by case/output: 10956/16384, 44049/65536, 10956/16384, 11389/16384; maximum stored-component difference 19614.

- **`normal_to_height_hq.sbs/base_algorithm_height_distance` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 14374/16384; maximum stored-component difference 347.

- **`normal_transform.sbs/normal_transform` - Too hard for this pass:** Exact pixels by case/output: 16356/16384, 65420/65536, 16356/16384, 16356/16384; maximum stored-component difference 1.

- **`panorama_gradients.sbs/panorama_gradient_linear` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 2.384186e-07.

- **`panorama_gradients.sbs/panorama_gradient_2_points` - Too hard for this pass:** Exact pixels by case/output: 445/16384, 1667/65536, 445/16384, 137/16384; maximum stored-component difference 4.172325e-07.

- **`panorama_nadir_extract.sbs/nadir_extract` - Too hard for this pass:** Exact pixels by case/output: 354/16384, 337/65536, 354/16384, 3312/16384; maximum stored-component difference 0.07189722.

- **`panorama_nadir_patch.sbs/panorama_nadir_patch` - Too hard for this pass:** Exact pixels by case/output: 15109/16384, 59874/65536, 15109/16384, 15569/16384; maximum stored-component difference 0.01915002.

- **`panorama_position.sbs/panorama_3d_position` - Too hard for this pass:** Exact pixels by case/output: 16183/16384, 64379/65536, 16183/16384, 16183/16384; maximum stored-component difference 1.

- **`panorama_rotation.sbs/panorama_rotation` - Too hard for this pass:** Exact pixels by case/output: 13885/16384, 48478/65536, 13885/16384, 11046/16384; maximum stored-component difference 7.

- **`panorama_shape.sbs/panorama_shape` - Too hard for this pass:** Exact pixels by case/output: 15802/16384, 63195/65536, 15802/16384, 15177/16384; maximum stored-component difference 9.202957e-05.

- **`panorama_shape.sbs/panorama_shape_2` - Too hard for this pass:** Exact pixels by case/output: 15374/16384, 61494/65536, 15374/16384, 10926/16384; maximum stored-component difference 6.747246e-05.

- **`panorama_shape.sbs/panorama_sphere_light` - Too hard for this pass:** Exact pixels by case/output: 16356/16384, 65425/65536, 16356/16384, 16269/16384; maximum stored-component difference 0.001916409.

- **`panorama_shape.sbs/panorama_plane_light` - Too hard for this pass:** Exact pixels by case/output: 15918/16384, 63690/65536, 15918/16384, 16384/16384; maximum stored-component difference 4.464388e-05.

- **`panorama_shape.sbs/panorama_spot` - Too hard for this pass:** Exact pixels by case/output: 16378/16384, 65508/65536, 16378/16384, 11303/16384; maximum stored-component difference 0.1226807.

- **`panorama_straighten_horizon.sbs/panorama_straighten_horizon` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 0/16384; maximum stored-component difference 250.

- **`paper_path.sbs/paper_path` - Too hard for this pass:** Exact pixels by case/output: 7664/16384, 27377/65536, 7766/16384, 6530/16384; maximum stored-component difference 65535.

- **`parquet_1.sbs/parquet_1` - Too hard for this pass:** Exact pixels by case/output: 16240/16384, 64996/65536, 16288/16384, 16096/16384; maximum stored-component difference 255.

- **`panorama_physical_sun_sky.sbs/Impl/GetOneParam` - Too hard for this pass:** Typed outputs match in 0/4 comparisons. output: [3.0352942943573, 3.1215686798095703, 3.1450982093811035] versus [3.0352942943573, 3.1215689182281494, 3.1450982093811035]; output: [2.8705883026123047, 3.3490195274353027, 2.462745428085327] versus [2.870588541030884, 3.349020004272461, 2.462745428085327]; output: [3.0352942943573, 3.1215686798095703, 3.1450982093811035] versus [3.0352942943573, 3.1215689182281494, 3.1450982093811035]; output: [0.9883579015731812, 0.8544117212295532, 1.0778186321258545] versus [0.9883579015731812, 0.8544118404388428, 1.0778186321258545].

- **`panorama_shape.sbs/panorama_line_light` - Too hard for this pass:** Exact pixels by case/output: 15172/16384, 60708/65536, 15172/16384, 14428/16384; maximum stored-component difference 0.0001544952.

- **`noise_bnw_spots_1.sbs/bnw_spots_1` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 1/16384, 0/65536, 1/16384; maximum stored-component difference 64390.

- **`noise_bnw_spots_1_v2.sbs/bnw_spots_1` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 131/16384, 541/65536, 141/16384; maximum stored-component difference 205.

- **`noise_bnw_spots_3.sbs/bnw_spots_3` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 0/16384, 1/65536, 0/16384; maximum stored-component difference 62911.

- **`noise_bnw_spots_3_v2.sbs/bnw_spots_3` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 49/16384, 162/65536, 48/16384; maximum stored-component difference 227.

- **`noise_cells_1.sbs/cells_1` - Too hard for this pass:** Exact pixels by case/output: 2/16384, 5/65536, 0/16384, 1/16384; maximum stored-component difference 56592.

- **`noise_cells_1_v2.sbs/cells_1` - Too hard for this pass:** Exact pixels by case/output: 236/16384, 1029/65536, 261/16384, 0/16384; maximum stored-component difference 223.

- **`noise_cells_4.sbs/cells_4` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 23/16384; maximum stored-component difference 65462.

- **`noise_cells_4_v2.sbs/cells_4` - Too hard for this pass:** Exact pixels by case/output: 98/16384, 379/65536, 33/16384, 1970/16384; maximum stored-component difference 253.

- **`noise_clouds_1.sbs/clouds_1` - Too hard for this pass:** Exact pixels by case/output: 2/16384, 5/65536, 0/16384, 0/16384; maximum stored-component difference 48017.

- **`noise_clouds_1_v2.sbs/clouds_1` - Too hard for this pass:** Exact pixels by case/output: 229/16384, 792/65536, 186/16384, 242/16384; maximum stored-component difference 156.

- **`noise_clouds_2.sbs/clouds_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 4/65536, 2/16384, 0/16384; maximum stored-component difference 44350.

- **`noise_clouds_2_v2.sbs/clouds_2` - Too hard for this pass:** Exact pixels by case/output: 201/16384, 832/65536, 211/16384, 245/16384; maximum stored-component difference 123.

- **`noise_creased.sbs/creased` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 0/65536, 2/16384, 1/16384; maximum stored-component difference 42493.

- **`noise_crystal_1.sbs/crystal_1` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 2/65536, 2/16384, 1/16384; maximum stored-component difference 61334.

- **`noise_crystal_2.sbs/crystal_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 3/65536, 0/16384, 1/16384; maximum stored-component difference 51894.

- **`noise_directional_noise_1.sbs/directional_noise_1` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 64802.

- **`noise_directional_noise_1_v2.sbs/directional_noise_1` - Too hard for this pass:** Exact pixels by case/output: 8/16384, 54/65536, 0/16384, 2/16384; maximum stored-component difference 240.

- **`noise_directional_noise_2.sbs/directional_noise_2` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 0/16384, 0/65536, 0/16384; maximum stored-component difference 65535.

- **`noise_directional_noise_3.sbs/directional_noise_3` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. 

- **`noise_directional_noise_4.sbs/directional_noise_4` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum stored-component difference 47884.

- **`noise_directional_noise_4_v2.sbs/directional_noise_4` - Too hard for this pass:** Exact pixels by case/output: 161/16384, 713/65536, 161/16384, 148/16384; maximum stored-component difference 207.

- **`noise_directional_scratches.sbs/directional_scratches` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 5334/16384; maximum stored-component difference 62981.

- **`noise_directional_scratches_v2.sbs/directional_scratches` - Too hard for this pass:** Exact pixels by case/output: 62/16384, 250/65536, 74/16384, 4096/16384; maximum stored-component difference 239.

- **`noise_dirt_1.sbs/dirt_1` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 1037/16384, 4038/65536, 992/16384; maximum stored-component difference 65535.

- **`noise_fluid.sbs/fluid` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 63791.

- **`noise_fractal_sum_base.sbs/fractal_sum_base` - Too hard for this pass:** Exact pixels by case/output: 2/16384, 6/65536, 1/16384, 0/16384; maximum stored-component difference 21118.

- **`noise_fractal_sum_base.sbs/fractal_sum_base_2` - Too hard for this pass:** Exact pixels by case/output: 2/16384, 6/65536, 1/16384, 0/16384; maximum stored-component difference 20137.

- **`noise_fractal_sum_base_v2.sbs/noise_upscale` - Too hard for this pass:** Exact pixels by case/output: 389/16384, 1536/65536, 389/16384, 279/16384; maximum stored-component difference 15.

- **`noise_fractal_sum_base_v2.sbs/noise_blend` - Too hard for this pass:** Exact pixels by case/output: 134/16384, 668/65536, 134/16384, 75/16384; maximum stored-component difference 100.

- **`noise_fur_1.sbs/fur_1` - Too hard for this pass:** Exact pixels by case/output: 13/16384, 75/65536, 16/16384, 10/16384; maximum stored-component difference 65535.

- **`noise_fur_2.sbs/fur_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 65530.

- **`noise_fur_3.sbs/fur_3` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 65529.

- **`noise_gaussian_noise.sbs/gaussian_noise` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 64187.

- **`noise_gaussian_noise_v2.sbs/gaussian_noise` - Too hard for this pass:** Exact pixels by case/output: 4/16384, 16/65536, 6/16384, 0/16384; maximum stored-component difference 251.

- **`noise_gaussian_spots_1.sbs/gaussian_spots_1` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 1/65536, 1/16384, 0/16384; maximum stored-component difference 58938.

- **`noise_gaussian_spots_1_v2.sbs/gaussian_spots_1` - Too hard for this pass:** Exact pixels by case/output: 99/16384, 434/65536, 118/16384, 121/16384; maximum stored-component difference 235.

- **`noise_gaussian_spots_2.sbs/gaussian_spots_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum stored-component difference 45111.

- **`noise_gaussian_spots_2_v2.sbs/gaussian_spots_2` - Too hard for this pass:** Exact pixels by case/output: 21/16384, 257/65536, 33/16384, 62/16384; maximum stored-component difference 255.

- **`noise_liquid.sbs/liquid` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 61102.

- **`noise_messy_fibers_1.sbs/messy_fibers_1` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum stored-component difference 65295.

- **`noise_messy_fibers_1_v2.sbs/messy_fibers_1` - Too hard for this pass:** Exact pixels by case/output: 37/16384, 153/65536, 58/16384, 46/16384; maximum stored-component difference 255.

- **`noise_messy_fibers_2.sbs/messy_fibers_2` - Too hard for this pass:** Exact pixels by case/output: 55/16384, 282/65536, 116/16384, 87/16384; maximum stored-component difference 65535.

- **`noise_messy_fibers_2_v2.sbs/messy_fibers_2` - Too hard for this pass:** Exact pixels by case/output: 163/16384, 686/65536, 95/16384, 213/16384; maximum stored-component difference 255.

- **`noise_messy_fibers_3.sbs/messy_fibers_3` - Too hard for this pass:** Exact pixels by case/output: 78/16384, 304/65536, 77/16384, 63/16384; maximum stored-component difference 65535.

- **`noise_messy_fibers_3_v2.sbs/messy_fibers_3` - Too hard for this pass:** Exact pixels by case/output: 128/16384, 515/65536, 138/16384, 3/16384; maximum stored-component difference 255.

- **`noise_microscope_view.sbs/microscope_view` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 65528.

- **`noise_perlin_noise.sbs/perlin_noise` - Too hard for this pass:** Exact pixels by case/output: 2/16384, 6/65536, 4/16384, 4/16384; maximum stored-component difference 44958.

- **`noise_perlin_noise_v2.sbs/perlin_noise` - Too hard for this pass:** Exact pixels by case/output: 107/16384, 448/65536, 130/16384, 168/16384; maximum stored-component difference 215.

- **`noise_plasma.sbs/plasma` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 3/65536, 1/16384, 0/16384; maximum stored-component difference 41315.

- **`noise_upscale_1.sbs/noise_upscale_1` - Too hard for this pass:** Exact pixels by case/output: 47496/65536, 189800/262144, 47496/65536, 46630/65536; maximum stored-component difference 42.

- **`noise_upscale_2.sbs/noise_upscale_2` - Too hard for this pass:** Exact pixels by case/output: 44458/65536, 177124/262144, 44458/65536, 44586/65536; maximum stored-component difference 23.

- **`noise_upscale_3.sbs/noise_upscale_3` - Too hard for this pass:** Exact pixels by case/output: 54720/65536, 223780/262144, 54720/65536, 55992/65536; maximum stored-component difference 2.

- **`pattern_alveolus.sbs/alveolus` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 13568/16384; maximum stored-component difference 65535.

- **`pattern_arc_pavement.sbs/arc_pavement` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65532/65536, 16384/16384, 12457/16384; maximum stored-component difference 65535.

- **`pattern_brick_1.sbs/brick_1` - Too hard for this pass:** Exact pixels by case/output: 14592/16384, 58368/65536, 14592/16384, 6528/16384; maximum stored-component difference 65535.

- **`pattern_brick_2.sbs/brick_2` - Too hard for this pass:** Exact pixels by case/output: 15040/16384, 61056/65536, 15040/16384, 6656/16384; maximum stored-component difference 65535.

- **`pattern_cube_3d.sbs/cube_3d` - Too hard for this pass:** Exact pixels by case/output: 16375/16384, 65369/65536, 16375/16384, 16328/16384; maximum stored-component difference 1.

- **`pattern_cube_3d.sbs/cube_3d_gbuffers` - Too hard for this pass:** Exact pixels by case/output: 16350/16384, 16384/16384, 16375/16384, 16352/16384, 65244/65536, 65536/65536, 65369/65536, 65322/65536, 16350/16384, 16384/16384, 16375/16384, 16352/16384, 16107/16384, 16384/16384, 16328/16384, 16253/16384; maximum stored-component difference 1.

- **`pattern_fibers_1.sbs/fibers_1` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum stored-component difference 48873.

- **`pattern_fibers_2.sbs/fibers_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 2/65536, 0/16384, 0/16384; maximum stored-component difference 62612.

- **`pattern_gaussian_1.sbs/gaussian_1` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 40180.

- **`pattern_gaussian_2.sbs/gaussian_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 23969.

- **`pattern_mesh_1.sbs/mesh_1` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 16/16384; maximum stored-component difference 65535.

- **`pattern_mesh_2.sbs/mesh_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 160/16384; maximum stored-component difference 65535.

- **`pattern_polygon_2.sbs/polygon_2` - Too hard for this pass:** Exact pixels by case/output: 14203/16384, 56174/65536, 14203/16384, 13032/16384; maximum stored-component difference 4.

- **`pattern_shape.sbs/shape_base` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16217/16384; maximum stored-component difference 1.

- **`pattern_shape.sbs/hemisphere` - Too hard for this pass:** Exact pixels by case/output: 16368/16384, 65536/65536, 16368/16384, 16361/16384; maximum stored-component difference 1.

- **`pattern_splatter_circular.sbs/splatter_circular` - Too hard for this pass:** Exact pixels by case/output: 15941/16384, 63805/65536, 15941/16384, 15401/16384; maximum stored-component difference 57311.

- **`pattern_splatter_circular.sbs/splatter_circular_color` - Too hard for this pass:** Exact pixels by case/output: 15529/16384, 62023/65536, 15529/16384, 15401/16384; maximum stored-component difference 65535.

- **`pattern_stripes.sbs/stripes` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 45056/65536, 16384/16384, 16384/16384; maximum stored-component difference 9557.

- **`pattern_tile_generator.sbs/tile_generator` - Too hard for this pass:** Exact pixels by case/output: 13097/16384, 52436/65536, 13097/16384, 16384/16384; maximum stored-component difference 18088.

- **`pattern_tile_generator.sbs/tile_generator_color` - Too hard for this pass:** Exact pixels by case/output: 10819/16384, 41641/65536, 10819/16384, 16384/16384; maximum stored-component difference 23978.

- **`pattern_tile_random.sbs/tile_random` - Too hard for this pass:** Exact pixels by case/output: 13532/16384, 53254/65536, 13610/16384, 11762/16384; maximum stored-component difference 57311.

- **`pattern_tile_random.sbs/tile_random_color` - Too hard for this pass:** Exact pixels by case/output: 13532/16384, 53254/65536, 13610/16384, 11762/16384; maximum stored-component difference 65535.

- **`pattern_tile_random_2.sbs/tile_random_2` - Too hard for this pass:** Exact pixels by case/output: 16378/16384, 4788/16384, 65497/65536, 18310/65536, 16368/16384, 5261/16384, 16353/16384, 12087/16384; maximum stored-component difference 1.

- **`pattern_weave_1.sbs/weave_1` - Too hard for this pass:** Exact pixels by case/output: 928/16384, 3872/65536, 928/16384, 848/16384; maximum stored-component difference 65535.

- **`pattern_weave_2.sbs/weave_2` - Too hard for this pass:** Exact pixels by case/output: 256/16384, 2304/65536, 256/16384, 39/16384; maximum stored-component difference 65535.

- **`pbr_render.sbs/PBR_render_mapping_color` - Too hard for this pass:** Exact pixels by case/output: 16223/16384, 65081/65536, 16223/16384, 16123/16384; maximum stored-component difference 1.

- **`pbr_render.sbs/PBR_render_mapping_grayscale` - Too hard for this pass:** Exact pixels by case/output: 15835/16384, 63262/65536, 15835/16384, 16170/16384; maximum stored-component difference 1.

- **`pbr_render.sbs/environment_map_rescale` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 2/65536, 1/16384, 0/16384; maximum stored-component difference 2.551079e-05.

- **`pbr_render.sbs/position_to_normal` - Too hard for this pass:** Exact pixels by case/output: 3119/16384, 4234/65536, 3119/16384, 1153/16384; maximum stored-component difference 0.1367188.

- **`pbr_render.sbs/chroma_shift` - Too hard for this pass:** Exact pixels by case/output: 16083/16384, 64304/65536, 16083/16384, 15570/16384; maximum stored-component difference 1.

- **`pbr_render.sbs/get_max_coords/get_max_coord` - Too hard for this pass:** Cooks report: Get Float2 (1379828382): undefined variable `$pos` in this context. 

- **`pbr_render.sbs/ibl_specular_contrib_anisotropic/panoramic_lod` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 0.98973.

- **`pbr_render2.sbs/PBR_render_mapping_color` - Too hard for this pass:** Exact pixels by case/output: 16223/16384, 65081/65536, 16223/16384, 16123/16384; maximum stored-component difference 1.

- **`pbr_render2.sbs/PBR_render_mapping_grayscale` - Too hard for this pass:** Exact pixels by case/output: 15835/16384, 63262/65536, 15835/16384, 16170/16384; maximum stored-component difference 1.

- **`pbr_render2.sbs/environment_map_rescale` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 2/65536, 1/16384, 0/16384; maximum stored-component difference 2.551079e-05.

- **`pbr_render2.sbs/position_to_normal` - Too hard for this pass:** Exact pixels by case/output: 8615/16384, 31600/65536, 8615/16384, 6737/16384; maximum stored-component difference 0.0004882812.

- **`pbr_render2.sbs/chroma_shift` - Too hard for this pass:** Exact pixels by case/output: 16083/16384, 64304/65536, 16083/16384, 15570/16384; maximum stored-component difference 1.

- **`pbr_render2.sbs/get_max_coords/get_max_coord` - Too hard for this pass:** Cooks report: Get Float2 (1379828382): undefined variable `$pos` in this context. 

- **`pbr_render2.sbs/ibl_specular_contrib_anisotropic/panoramic_lod` - Too hard for this pass:** Exact pixels by case/output: 42/16384, 1690/65536, 42/16384, 741/16384; maximum stored-component difference 155.

- **`pbr_render2.sbs/lod` - Too hard for this pass:** Exact pixels by case/output: 12328/16384, 49408/65536, 12328/16384, 12632/16384; maximum stored-component difference 127.

- **`pbr_render_ortho.sbs/mipmap_generator` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 2.687117.

- **`pbr_render_ortho.sbs/rotate_environment` - Too hard for this pass:** Exact pixels by case/output: 13885/16384, 48478/65536, 13885/16384, 11791/16384; maximum stored-component difference 7.

- **`pbr_render_ortho.sbs/blue_noise` - Too hard for this pass:** Exact pixels by case/output: 133/16384, 489/65536, 133/16384, 144/16384; maximum stored-component difference 3.874302e-07.

- **`pbr_render_ortho.sbs/environment_map_rescale` - Too hard for this pass:** Exact pixels by case/output: 1/16384, 2/65536, 1/16384, 0/16384; maximum stored-component difference 2.551079e-05.

- **`pbr_render_ortho.sbs/vector_shadow_pass` - Too hard for this pass:** Exact pixels by case/output: 14758/16384, 60408/65536, 14758/16384, 14762/16384; maximum stored-component difference 1.149848.

- **`noise_directional_noise_2_v2.sbs/directional_noise_2` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`noise_directional_noise_3_v2.sbs/directional_noise_3` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`noise_dirt_1_v2.sbs/dirt_1` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`noise_moisture_noise.sbs/moisture_noise` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`noise_moisture_noise_1_v2.sbs/moisture_noise_1` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`noise_moisture_noise_2.sbs/moisture_noise_2` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`pattern_triangle_grid.sbs/triangle_grid_base` - Too hard for this pass:** The release CPU comparison did not finish its four-case batch within the 180-second per-graph allowance. Reference captures completed; no pixel-parity claim is made for this timed-out run.

- **`noise_voronoi.sbs/3d_voronoi_fractal` - Too hard for this pass:** All four 16x16/32x32 cases now cook after the unassigned-counter fix. Exact pixels by case/output: 232/256, 950/1024, 245/256, 253/256; maximum stored-component difference 75.

- **`noise_voronoi.sbs/3d_voronoi` - Too hard for this pass:** All four 16x16/32x32 cases now cook after the unassigned-counter fix. Exact pixels by case/output: 238/256, 968/1024, 240/256, 255/256; maximum stored-component difference 1.

- **`noise_voronoi.sbs/voronoi` - Too hard for this pass:** All four 16x16/32x32 cases now cook after the unassigned-counter fix. Exact pixels by case/output: 225/256, 888/1024, 175/256, 185/256; maximum stored-component difference 178.

- **`noise_voronoi.sbs/voronoi_fractal` - Too hard for this pass:** All four 16x16/32x32 cases now cook after the unassigned-counter fix. Exact pixels by case/output: 124/256, 502/1024, 87/256, 179/256; maximum stored-component difference 36916.

- **`pattern_tile_sampler.sbs/arrow` - Too hard for this pass:** The importer drops the explicit 512x512 graph-level output size. Manually aligning both graph sizes still leaves rendering differences. These three captures use 512x512 output. Exact pixels by case/output: 259850/262144, 259850/262144, 259850/262144; maximum stored-component difference 155.

- **`pbr_render_ortho.sbs/ibl_shadow_pass` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 31.45882.

- **`composite.sbs/composite/blends/blend_matte` - Too hard for this pass:** Exact pixels by case/output: 90/16384, 359/65536, 90/16384, 118/16384; maximum stored-component difference 196.

- **`noise_dirt_2.sbs/dirt_2` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 14450/16384, 57939/65536, 14517/16384; maximum stored-component difference 65535.

- **`noise_dirt_3.sbs/dirt_3` - Too hard for this pass:** Exact pixels by case/output: 375/16384, 0/65536, 326/16384, 259/16384; maximum stored-component difference 65535.

- **`noise_dirt_3_v2.sbs/dirt_3` - Too hard for this pass:** Exact pixels by case/output: 363/16384, 0/65536, 427/16384, 94/16384; maximum stored-component difference 255.

- **`noise_dirt_5.sbs/dirt_5` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 2184/16384, 6519/65536, 2184/16384; maximum stored-component difference 61798.

- **`noise_dirt_5_v2.sbs/dirt_5` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. Exact pixels by case/output: 1988/16384, 5879/65536, 1997/16384; maximum stored-component difference 218.

- **`noise_dirt_gradient.sbs/dirt_gradient` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 2/65536, 0/16384, 0/16384; maximum stored-component difference 62840.

- **`perlin_noise_zoom.sbs/perlin_noise_zoom` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 62227.

- **`pre_multiply_straight.sbs/premult_to_straight` - Too hard for this pass:** Exact pixels by case/output: 16376/16384, 65504/65536, 16376/16384, 16381/16384; maximum stored-component difference 255.

- **`quad_transform.sbs/quad_transform` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 15477/16384; maximum stored-component difference 1.

- **`quad_transform.sbs/quad_transform_grayscale` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 15846/16384; maximum stored-component difference 1.

- **`quantize_color.sbs/modify_color_palette_sub` - Too hard for this pass:** Exact pixels by case/output: 240/256, 240/256, 240/256, 240/256; maximum stored-component difference 0.7294117.

- **`quantize_color.sbs/kmeans_iteration` - Too hard for this pass:** Exact pixels by case/output: 4080/4096, 4080/4096, 4080/4096, 4080/4096; maximum stored-component difference 6.023682.

- **`quantize_color.sbs/get_min_max/GetMaxFromAlpha_v2` - Too hard for this pass:** Exact pixels by case/output: 0/1, 1/1, 0/1, 0/1; maximum stored-component difference 128.

- **`quantize_color.sbs/color_conversions/convert_RGB_to_Lab` - Too hard for this pass:** Exact pixels by case/output: 16163/16384, 64702/65536, 16163/16384, 16163/16384; maximum stored-component difference 1.

- **`quantize_color.sbs/color_conversions/convert_Lab_to_RGB` - Too hard for this pass:** Exact pixels by case/output: 16377/16384, 65504/65536, 16377/16384, 16383/16384; maximum stored-component difference 255.

- **`radial_blur.sbs/radial_blur_grayscale` - Too hard for this pass:** Exact pixels by case/output: 48/16384, 200/65536, 48/16384, 19/16384; maximum stored-component difference 2355.

- **`replace_color.sbs/replace_color` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 1/16384; maximum stored-component difference 64.

- **`rt_caustics.sbs/rt_caustics_base` - Too hard for this pass:** Exact pixels by case/output: 282/16384, 1219/65536, 282/16384, 185/16384; maximum stored-component difference 242.

- **`rt_irradiance.sbs/panoramic_lod` - Too hard for this pass:** Exact pixels by case/output: 4979/16384, 20210/65536, 4979/16384, 5976/16384; maximum stored-component difference 155.

- **`safe_transform.sbs/safe_transform` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 3649/16384; maximum stored-component difference 145.

- **`safe_transform.sbs/safe_transform_grayscale` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 6679/16384; maximum stored-component difference 132.

- **`scratches_1.sbs/scratches_1` - Too hard for this pass:** Exact pixels by case/output: 3025/16384, 12465/65536, 2995/16384, 2178/16384; maximum stored-component difference 64929.

- **`scratches_2.sbs/scratches_2` - Too hard for this pass:** Exact pixels by case/output: 431/16384, 567/65536, 461/16384, 291/16384; maximum stored-component difference 65534.

- **`scratches_4.sbs/scratches_4` - Too hard for this pass:** Exact pixels by case/output: 97/16384, 1946/65536, 96/16384; maximum stored-component difference 65535.

- **`shape.sbs/shape_base` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16217/16384; maximum stored-component difference 1.

- **`shape_mapper.sbs/shape_mapper` - Too hard for this pass:** Exact pixels by case/output: 7688/16384, 28865/65536, 7688/16384, 5078/16384; maximum stored-component difference 30712.

- **`shape_splatter.sbs/shape_splatter_blend_color` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16383/16384; maximum stored-component difference 1.

- **`shape_splatter.sbs/shape_splatter_blend` - Too hard for this pass:** Exact pixels by case/output: 16379/16384, 65532/65536, 16379/16384, 16319/16384; maximum stored-component difference 36.

- **`shape_splatter.sbs/shape_splatter_data_extract` - Too hard for this pass:** Exact pixels by case/output: 12131/16384, 4638/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 48527/65536, 19321/65536, 65536/65536, 65536/65536, 65536/65536, 65536/65536, 65536/65536, 12131/16384, 4638/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 12151/16384, 4862/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384, 16384/16384; maximum stored-component difference 1.192093e-07.

- **`shape_splatter_v2.sbs/mipmap_tools/Mipmap_N_from_O_color` - Too hard for this pass:** Exact pixels by case/output: 28672/32768, 114688/131072, 28672/32768, 15872/16384; maximum stored-component difference 255.

- **`shape_splatter_v2.sbs/shape_splatter_compute_normal` - Too hard for this pass:** Exact pixels by case/output: 128/16384, 256/65536, 128/16384, 128/16384; maximum stored-component difference 255.

- **`skew.sbs/skew` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 14625/16384; maximum stored-component difference 1.

- **`skew.sbs/skew_grayscale` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 15263/16384; maximum stored-component difference 1.

- **`slope_blur.sbs/slope_blur` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 0/16384; maximum stored-component difference 160.

- **`slope_blur.sbs/slope_blur_grayscale` - Too hard for this pass:** Exact pixels by case/output: 296/16384, 2588/65536, 296/16384, 217/16384; maximum stored-component difference 124.

- **`slope_blur.sbs/slope_blur_grayscale_2` - Too hard for this pass:** Exact pixels by case/output: 296/16384, 2588/65536, 296/16384, 217/16384; maximum stored-component difference 124.

- **`smart_auto_tile.sbs/pattern_find_edges_step` - Too hard for this pass:** Exact pixels by case/output: 1048574/1048576, 1048575/1048576, 1048574/1048576, 1024/1024; maximum stored-component difference 255.

- **`splatter_circular.sbs/splatter_circular` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 15750/16384; maximum stored-component difference 57311.

- **`splatter_circular.sbs/splatter_circular_color` - Too hard for this pass:** Exact pixels by case/output: 14564/16384, 57648/65536, 14564/16384, 13974/16384; maximum stored-component difference 65535.

- **`spot_emboss.sbs/spot_emboss` - Too hard for this pass:** Exact pixels by case/output: 460/16384, 2680/65536, 460/16384, 163/16384; maximum stored-component difference 255.

- **`starburst.sbs/starburst` - Too hard for this pass:** Exact pixels by case/output: 8952/16384, 35696/65536, 8952/16384, 12412/16384; maximum stored-component difference 58561.

- **`stripes.sbs/stripes` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 45056/65536, 16384/16384, 16350/16384; maximum stored-component difference 65535.

- **`swirl.sbs/swirl` - Too hard for this pass:** Exact pixels by case/output: 16186/16384, 64854/65536, 16186/16384, 15722/16384; maximum stored-component difference 2.

- **`swirl.sbs/swirl_grayscale` - Too hard for this pass:** Exact pixels by case/output: 16274/16384, 65132/65536, 16274/16384, 16007/16384; maximum stored-component difference 2.

- **`symmetry_slice.sbs/symmetry_slice` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 11619/16384; maximum stored-component difference 7.

- **`tile_generator.sbs/tile_generator` - Too hard for this pass:** Exact pixels by case/output: 12821/16384, 51231/65536, 12821/16384, 14592/16384; maximum stored-component difference 18088.

- **`tile_generator_color.sbs/tile_generator_color` - Too hard for this pass:** Exact pixels by case/output: 10819/16384, 41641/65536, 10819/16384, 13363/16384; maximum stored-component difference 23978.

- **`tile_random.sbs/tile_random` - Too hard for this pass:** Exact pixels by case/output: 13932/16384, 53844/65536, 13189/16384, 11424/16384; maximum stored-component difference 57311.

- **`tile_random.sbs/tile_random_color` - Too hard for this pass:** Exact pixels by case/output: 13932/16384, 53844/65536, 13189/16384, 11424/16384; maximum stored-component difference 65535.

- **`tile_sampler.sbs/arrow` - Too hard for this pass:** Exact pixels by case/output: 259850/262144, 259850/262144, 259850/262144; maximum stored-component difference 155.

- **`tileable_material_to_mesh_material.sbs/tileable_material_to_mesh_material` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 16384/16384, 65536/65536, 65536/65536, 16384/16384, 16384/16384, 16384/16384, 4460/16384; maximum stored-component difference 4.

- **`trapezoid_transform.sbs/trapezoid_transform` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16382/16384; maximum stored-component difference 1.

- **`trapezoid_transform.sbs/trapezoid_transform_grayscale` - Too hard for this pass:** Exact pixels by case/output: 16384/16384, 65536/65536, 16384/16384, 16382/16384; maximum stored-component difference 1.

- **`uv_mapper.sbs/uv_mapper_grayscale_sub` - Too hard for this pass:** Exact pixels by case/output: 15835/16384, 63262/65536, 15835/16384, 16384/16384; maximum stored-component difference 1.

- **`uv_mapper.sbs/uv_mapper_color_sub` - Too hard for this pass:** Exact pixels by case/output: 16223/16384, 65081/65536, 16223/16384, 16384/16384; maximum stored-component difference 1.

- **`vector_morph.sbs/vector_morph` - Too hard for this pass:** Exact pixels by case/output: 2840/16384, 15375/65536, 2840/16384, 3262/16384; maximum stored-component difference 250.

- **`vector_morph.sbs/vector_morph_grayscale` - Too hard for this pass:** Exact pixels by case/output: 1277/16384, 4578/65536, 1277/16384, 1540/16384; maximum stored-component difference 146.

- **`vertical_noise.sbs/vertical_noise` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 1/65536, 0/16384, 0/16384; maximum stored-component difference 51100.

- **`waveform_01_v2.sbs/waveform_01_v2` - Too hard for this pass:** Exact pixels by case/output: 12757/16384, 57538/65536, 12257/16384, 15190/16384; maximum stored-component difference 255.

- **`waveform_1.sbs/waveform_1` - Too hard for this pass:** Exact pixels by case/output: 10166/16384, 44500/65536, 10112/16384, 16046/16384; maximum stored-component difference 65535.

- **`weave_1.sbs/weave_1` - Too hard for this pass:** Exact pixels by case/output: 928/16384, 3872/65536, 928/16384, 848/16384; maximum stored-component difference 65535.

- **`weave_2.sbs/weave_2` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 2/16384; maximum stored-component difference 64614.

- **`weave_3.sbs/weave_3` - Too hard for this pass:** Exact pixels by case/output: 15760/16384, 63040/65536, 15760/16384, 14816/16384; maximum stored-component difference 64255.

- **`weave_4.sbs/weave_4` - Too hard for this pass:** Exact pixels by case/output: 10240/16384, 40960/65536, 10240/16384, 9728/16384; maximum stored-component difference 64511.

- **`white_noise.sbs/white_noise_fast` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 2/65536, 1/16384; maximum stored-component difference 65348.

- **`white_noise_v2.sbs/white_noise_v2` - Too hard for this pass:** Exact pixels by case/output: 68/16384, 298/65536, 70/16384, 153/16384; maximum stored-component difference 255.

- **`wood_fibers_1.sbs/wood_fibers_1` - Too hard for this pass:** Exact pixels by case/output: 3/16384, 10/65536, 0/16384, 4/16384; maximum stored-component difference 65495.

- **`wood_fibers_2.sbs/wood_fibers_2` - Too hard for this pass:** Cooks report: pattern graph exceeded its execution budget. 

- **`paths_tools.sbs/paths_polygon` - Too hard for this pass:** Exact pixels by case/output: 252/256, 252/256, 252/256, 250/256; maximum stored-component difference 5.602837e-06.

- **`paths_tools.sbs/paths_trace` - Too hard for this pass:** Exact pixels by case/output: 184/256, 182/256, 184/256, 138/256; maximum stored-component difference 145.

- **`paths_tools.sbs/vertex_processor` - Too hard for this pass:** Exact pixels by case/output: 250/256, 248/256, 250/256, 248/256; maximum stored-component difference 1.

- **`paths_tools.sbs/vertex_processor_simple` - Too hard for this pass:** Exact pixels by case/output: 250/256, 248/256, 250/256, 248/256; maximum stored-component difference 1.

- **`paths_tools.sbs/Impl/compute_paths_lengths` - Too hard for this pass:** Cooks report: function returned bool, expected native type 256. 

- **`paths_tools.sbs/Impl/sort_by_blue` - Too hard for this pass:** Cooks report: function returned bool, expected native type 256. Exact pixels by case/output: 256/256, 256/256, 256/256; maximum stored-component difference 0.

- **`paths_tools.sbs/Impl/mask_to_facets` - Too hard for this pass:** Exact pixels by case/output: 256/256, 240/256, 256/256, 247/256; maximum stored-component difference 2.980232e-08.

- **`spline_tools.sbs/internal_tools/2dtransform_spline_sub` - Too hard for this pass:** Its connectable spline-count input defaults to zero. Nonempty image buffers are available, but a nonzero value-pin case is not verified through the reference API in this pass.

- **`spline_tools.sbs/internal_tools/compute_bridge_distance` - Too hard for this pass:** Its connectable spline-count input defaults to zero. A meaningful nonempty case needs a configured value input; that case is not verified in this pass.

- **`spline_tools.sbs/internal_tools/points_preview` - Too hard for this pass:** Its connectable point-count input defaults to zero. Empty-preview agreement cannot establish parity for drawing a nonempty point list.

- **`spline_tools.sbs/internal_tools/scatter_splines_on_splines_preprocess` - Too hard for this pass:** The reference compute did not finish after more than six minutes with the default zero input-spline count and original curve data. The comparison session was recovered; nonzero value-input behavior is not verified.

- **`spline_tools.sbs/internal_tools/compute_spline_tangents` - Too hard for this pass:** Exact pixels by case/output: 247/256, 249/256, 247/256, 248/256; maximum stored-component difference 0.1875924.

- **`spline_tools.sbs/internal_tools/normalize_spline_distance` - Too hard for this pass:** Exact pixels by case/output: 193/256, 193/256, 193/256, 193/256; maximum stored-component difference 1.907349e-06.

- **`spline_tools.sbs/internal_tools/compute_spline_length_init` - Too hard for this pass:** Exact pixels by case/output: 251/256, 250/256, 251/256, 250/256; maximum stored-component difference 1.490116e-08.

- **`spline_tools.sbs/internal_tools/resample_spline_data` - Too hard for this pass:** Exact pixels by case/output: 220/256, 208/256, 220/256, 208/256, 220/256, 208/256, 220/256, 208/256; maximum stored-component difference 0.5.

- **`spline_tools.sbs/internal_tools/spline_warp_sub` - Too hard for this pass:** Exact pixels by case/output: 253/256, 253/256, 253/256, 240/256; maximum stored-component difference 0.1294118.

- **`spline_tools.sbs/internal_tools/select_spline_sub` - Too hard for this pass:** Exact pixels by case/output: 30720/32768, 30720/32768, 30720/32768, 30720/32768; maximum stored-component difference 0.5240468.

- **`spline_tools.sbs/internal_tools/modify_spline_property_sub` - Too hard for this pass:** Exact pixels by case/output: 15/256, 15/256, 15/256, 240/256; maximum stored-component difference 0.2784314.

- **`paths_tools.sbs/Impl/facets_to_paths_impl_link_template` - Too hard for this pass:** Exact pixels by case/output: 0/256, 0/256, 0/256, 256/256; maximum stored-component difference 1.

- **`paths_tools.sbs/Impl/facets_to_paths_impl_stitch_template` - Too hard for this pass:** Exact pixels by case/output: 0/256, 0/256, 0/256, 0/256; maximum stored-component difference 1.

- **`pavement_1.sbs/pavement_1` - Too hard for this pass:** Exact pixels by case/output: 2160/16384, 9712/65536, 2160/16384, 800/16384; maximum stored-component difference 65535.

- **`splatter.sbs/splatter` - Too hard for this pass:** Exact pixels by case/output: 2048/16384, 4096/65536, 2048/16384, 966/16384; maximum stored-component difference 44975.

- **`splatter_color.sbs/splatter_color` - Too hard for this pass:** Exact pixels by case/output: 0/16384, 0/65536, 0/16384, 924/16384; maximum stored-component difference 47536.

- **`noise_dirt_4.sbs/dirt_4` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`noise_dirt_4_v2.sbs/dirt_4` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`noise_dirt_gradient_v2.sbs/dirt_gradient` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`pbr_render.sbs/raytrace_position` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`pbr_render.sbs/raytraced_specular_pass_8` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`pbr_render.sbs/raytraced_irradiance_pass_8` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`pbr_render2.sbs/raytrace_position` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`pbr_render2.sbs/raytraced_specular_pass_8` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`pbr_render2.sbs/raytraced_irradiance_is_pass_8` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`rt_irradiance.sbs/mip_max` - Too hard for this pass:** The reference output is 128x128 while MaterialCraft produces 64x64 for the same 128x128 case. Pixel parity cannot be claimed with different output dimensions.

- **`rt_shadows.sbs/mip_max` - Too hard for this pass:** The reference output is 128x128 while MaterialCraft produces 64x64 for the same 128x128 case. Pixel parity cannot be claimed with different output dimensions.

- **`pbr_render_ortho.sbs/sphere_displace` - Too hard for this pass:** The MaterialCraft measurement exceeded the 180-second limit. Reference captures exist; exact pixel parity remains unverified.

- **`rt_irradiance.sbs/compute_ray_direction` - Too hard for this pass:** Exact pixels by case/output: 16368/16384, 65414/65536, 16368/16384; maximum stored-component difference 0.0004882812. Connectable value pins retain their defaults; nondefault values require a wired reference fixture and remain unverified.

- **`rt_irradiance.sbs/compute_irradiance` - Too hard for this pass:** Exact pixels by case/output: 7010/16384, 27432/65536, 7010/16384; maximum stored-component difference 0.001953125. Connectable value pins retain their defaults; nondefault values require a wired reference fixture and remain unverified.

- **`rt_irradiance.sbs/debug_hit_uv` - Too hard for this pass:** Exact pixels by case/output: 67/16384, 260/65536, 67/16384; maximum stored-component difference 159. Connectable value pins retain their defaults; nondefault values require a wired reference fixture and remain unverified.

## Verified cases

- **`3d_perlin_noise.sbs/3D_perlin_noise_fractal` - Parity:** All 114,688 eight-bit output pixels match across four cases: defaults at 128x128 and 256x256, seed 42 at 128x128, and roughness 0.7 at 128x128. Generated linear XY position ramps use Z=128/255 and alpha=1. Parent precision is eight bits. Other parameters retain defaults. This validates these cases, not every parameter combination.

- **`white_noise.sbs/white_noise` - Parity:** All 98,304 sixteen-bit pixels match across defaults at 128x128/256x256 and seed 42. This preserves the original result after the shared importer, sampler, and function changes.

- **`3d_simplex_noise.sbs/3d_simplex_noise` - Parity:** All 114,688 eight-bit pixels match for defaults at 128x128 and 256x256, seed 42 at 128x128, and scale 4 at 128x128. Generated position maps match the other 3D-noise comparisons.

- **`3d_texture_jump_flood.sbs/3d_texture_jump_flood_iteration` - Parity:** All 131,072 eight-bit RGBA pixels match across ramp inputs at 128x128 and 256x256, iteration 1 at 128x128, and two randomized 128x128 candidate-point cases (iterations 0 and 1, with disabled seeds). Floored modulo fixes the previously incorrect negative-coordinate wrapping.

- **`3d_texture_position.sbs/3d_texture_position` - Parity:** Both native 4096x4096 RGBA16 atlases match exactly: default grid 16x16 and grid 8x4. All 33,554,432 pixels (134,217,728 components) were compared at the graph's actual fixed output size, with no resizing.

- **`3d_texture_transform.sbs/3d_texture_offset_color` - Parity:** All 131,072 RGBA8 pixels match across five cases: default 128x128, fractional signed offsets at 128x128, voxel offsets at 256x256, changed grid 8x4 with offsets at 128x128, and varying input alpha with fractional offsets. All four channels were compared.

- **`3d_texture_transform.sbs/3d_texture_offset_grayscale` - Parity:** All 114,688 eight-bit pixels match across four corresponding default, fractional-offset, voxel-offset, and changed-grid cases at 128x128 and 256x256.

- **`angle_to_vector.sbs/angle_to_vector` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, signed-vector format, and angle offset 0.25. Inputs are original scalar angle maps; all four output channels were compared.

- **`atlas_spliter.sbs/max_ba` - Parity:** All 98,304 eight-bit output pixels match across 128x128/256x256 color inputs and a varying-alpha 128x128 case. The varying-alpha output is nonconstant.

- **`auto_crop.sbs/min_max` - Parity:** All 24,576 RGBA8 output pixels match for original rectangular masks at parent sizes 128 and 256 and a varying-alpha color case. Actual outputs are 64x64, 128x128, and 64x64 respectively.

- **`auto_levels.sbs/auto_levels` - Parity:** All 86,016 eight-bit pixels match for restricted-range inputs at 128x128/256x256 and a constant 64x64 image. The nonconstant inputs span levels 32 through 191; the flat input is level 96.

- **`auto_levels.sbs/min_max` - Parity:** All 102,400 eight-bit pixels match across minimum/maximum selection at 128x128, minimum selection at 256x256, and a flat 64x64 input.

- **Exact blend family - Parity:** `blend.sbs/add`, `add_sub`, `copy`, `difference`, `exclusion`, `hard_light`, `linear_burn`, `max`, `min`, `multiply`, `overlay`, `screen`, `soft_light`, and `subtract` each match all 114,688 RGBA8 pixels across four cases: opaque 128x128, varying foreground/background alpha at 128x128, opacity 0.37 with alpha-channel blending disabled, and 256x256 varying-alpha inputs with opacity 0.67 and seed 42. Original masks and all four output channels were compared. Add and Copy additionally match their 16x16 RGBA32F HDR cases (512 pixels total).

- **`blend_switch.sbs/multi_switch` - Parity:** All 131,072 RGBA8 pixels match across default selection, twentieth input, eleventh of twelve inputs, out-of-range selection with three active inputs, and second input at 256x256. Original twenty-input maps include varying alpha.

- **`blend_switch.sbs/switch` - Parity:** Both boolean selections at 128x128 and 256x256 match all 163,840 RGBA8 pixels, including varying alpha.

- **Grayscale switches - Parity:** `blend_switch.sbs/multi_switch_grayscale` matches all 131,072 eight-bit pixels in the same five selection cases as its color counterpart. `switch_grayscale` matches both boolean selections at 128x128/256x256 (163,840 eight-bit pixels) and both selections at 128x128 in float storage (32,768 pixels). Original color resources exercise the corrected bridge conversion.

- **`brick_generator.sbs/resources/Tile` - Parity:** Defaults at 128x128/256x256 and seed 42 are exact but constant. A seven-by-three grid with scale 0.8 and offset 0.5 produces a nonconstant result and also matches exactly. All 114,688 sixteen-bit pixels across four cases agree.

- **`checker_2.sbs/checker_2` - Parity:** All 114,688 eight-bit pixels match for defaults at 128x128/256x256, seed 42, and changed tiling/luminance variation/disorder at 128x128.

- **`clamp.sbs/clamp_grayscale` - Parity:** All 114,688 eight-bit pixels match across defaults at 128x128/256x256, seed 42, and changed clamp limits.

- **`clone_patch.sbs/transform_to_center` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, seed 42, and changed transform settings. Default all-one matrix settings produce black; the changed nonsingular matrix/offset produces a nonconstant output and is also exact.

- **`clone_patch.sbs/transform_to_center_grayscale` - Parity:** All 114,688 eight-bit pixels match across defaults at 128x128/256x256, seed 42, and changed transform settings. Default all-one matrix settings produce black; the changed nonsingular matrix/offset produces a nonconstant output and is also exact.

- **`clone_patch.sbs/transform_from_center` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, seed 42, and changed transform settings. Default all-one matrix settings produce black; the changed nonsingular matrix/offset produces a nonconstant output and is also exact.

- **`color_space_conversion.sbs/linear_srgb_to_acescg` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`color_space_conversion.sbs/acescg_to_srgb` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`color_space_conversion.sbs/linear_srgb_to_srgb` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`color_space_conversion.sbs/srgb_to_linear_srgb` - Parity:** All 114,688 RGBA8 pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`color_space_conversion.sbs/linear_srgb_to_srgb_grayscale` - Parity:** All 114,688 eight-bit pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`color_space_conversion.sbs/srgb_to_linear_srgb_grayscale` - Parity:** All 114,688 eight-bit pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **Constant typed values - Parity:** `constant_values.sbs` graphs `constant_float`, `constant_float2`, `constant_float3`, `constant_float4`, `constant_integer`, `constant_integer2`, `constant_integer3`, `constant_boolean` match all three recorded values each (24 typed output comparisons). Cases include zero, positive/negative scalar and vector values, HDR floats, and both Boolean states. These graphs publish values, so image pixel comparisons do not apply. The four-component integer boundary exception is recorded above.

- **`clamp.sbs/clamp` - Parity:** All 114,688 RGBA8 pixels match for defaults at 128x128/256x256, seed 42, and limits 0.2/0.7 with alpha clamping disabled. Original varying-alpha input maps exercise both the corrected legacy ignore-alpha stages and the final alpha-channel selection.

- **`composite.sbs/composite/filters/filter_transformation_grayscale` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed settings.

- **`composite.sbs/composite/filters/filter_mask` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`composite.sbs/composite/filters/filter_split_rgb_a` - Parity:** All 229,376 output pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map. All output bridges were compared.

- **`composite.sbs/composite/blends/blend_content_grayscale` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed settings.

- **`composite.sbs/composite/blends/blend_matte_fg_passthrough` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`composite.sbs/composite/blends/blend_matte_bg_passthrough` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`composite.sbs/composite/blends/blend_content_pre_post_composite_grayscale` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed settings.

- **`composite.sbs/composite/background/uniform_color_float3` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed settings.

- **`contrast_luminosity.sbs/contrast_luminosity` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed contrast/luminosity.

- **`contrast_luminosity.sbs/contrast_luminosity_grayscale` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed contrast/luminosity.

- **`exposure.sbs/exposure` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and exposure 1.5.

- **`extend_shape.sbs/extend_shape_grayscale` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed extension mode/endpoints with nearest sampling. Original varying-alpha color or nonconstant grayscale input maps were used.

- **`extend_shape.sbs/extend_shape_color` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed extension mode/endpoints with nearest sampling. Original varying-alpha color or nonconstant grayscale input maps were used.

- **`facing_normal.sbs/facing_normal` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case. Inputs are original curved normal maps.

- **`flood_fill.sbs/flood_fill_step` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared.

- **`flood_fill.sbs/flood_fill_to_bbox_size` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed exposed parameters. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared.

- **`flood_fill.sbs/flood_fill_to_position` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared.

- **`flood_fill_2.sbs/flood_fill_to_bbox_size` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed exposed parameters. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared.

- **`flood_fill_2.sbs/flood_fill_to_position` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared.

- **`flood_fill_2.sbs/flood_fill_impl/flood_fill_step_A` - Parity:** All 229,376 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared. Every output bridge was compared.

- **`flood_fill_2.sbs/flood_fill_impl/flood_fill_step_A_switch` - Parity:** All 229,376 output pixels match across defaults at 128x128/256x256, seed 42, and changed exposed parameters. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared. Every output bridge was compared.

- **`flood_fill_2.sbs/flood_fill_impl/flood_fill_step_B` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case. Inputs are original bounded RGBA coordinate/box fields; color and alpha were compared.

- **`floodfill_to_index.sbs/pass` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`fxaa.sbs/fxaa_grayscale` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and the additional input case.

- **`glow.sbs/clamp` - Parity:** All 114,688 output pixels match across defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`curvature_smooth_v2.sbs/compute_curvature_v2` - Parity:** All 16,668 RGBA32F pixels match in six cases after correcting internal size resolution. Actual outputs are three 2x2 images, one 4x4 image, one 16x16 image, and one 128x128 image. Larger cases change the level and both intensity components and use original nonconstant curved normal and previous-result maps.

- **`grid_atlas.sbs/grid_atlas_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`grid_atlas.sbs/grid_atlas_color` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`hdr_exposure_preview.sbs/hdr_exposure_preview` - Parity:** All 114,688 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters. The parent format is f32.

- **`height_extrude.sbs/impl/acceleration_struct_impl` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`histogram_range.sbs/histogram_range` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`histogram_scan_non_uniform.sbs/histogram_scan_non_uniform` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`histogram_select.sbs/histogram_select` - Parity:** All 114,688 output pixels (LUM16) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`histogram_tools.sbs/horizontal_max` - Parity:** All 4 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`id_to_mask.sbs/id_to_mask_color` - Parity:** All 114,688 output pixels (LUM16) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`id_to_mask.sbs/id_to_mask_grayscale` - Parity:** All 114,688 output pixels (LUM16) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`invert.sbs/invert` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`invert.sbs/invert_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`linear_srgb_conversion.sbs/convert_to_linear` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`linear_srgb_conversion.sbs/convert_to_linear_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`linear_srgb_conversion.sbs/convert_to_srgb` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`linear_srgb_conversion.sbs/convert_to_srgb_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`material_switch.sbs/material_switch` - Parity:** All 1,376,256 output pixels (LUM8, RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters. All output bridges were compared.

- **`median_filter.sbs/median_filter_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`mesh_adaptive_base_color.sbs/base_color_8_inputs` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`mesh_adaptive_base_color.sbs/base_color_4_inputs` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`mesh_adaptive_base_color.sbs/base_color_2_inputs` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`mesh_data_combiner.sbs/mesh_data_combiner` - Parity:** All 1,261,568 output pixels (LUM16, LUM8, RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters. All output bridges were compared.

- **`multiangle_to_normal.sbs/angle_to_normal` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`multiangle_to_normal.sbs/angle_weight` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`normal_color.sbs/normal_color` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`normal_intensity.sbs/normal_intensity` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`normal_invert.sbs/normal_invert` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`normal_mapper.sbs/normal_mapper` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`normal_vector_rotation.sbs/normal_vector_rotation` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`normal_uncombine.sbs/get_intensity_iteration` - Parity:** All 4 typed outputs match exactly across 4 recorded cases, including an alternate original input or changed exposed parameters.

- **`noise_fractal_sum_base_v2.sbs/noise_disorder` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pattern_brick_generator.sbs/resources/Tile` - Parity:** All 114,688 output pixels (LUM16) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pattern_checker_1.sbs/checker_1` - Parity:** All 114,688 output pixels (LUM16) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pattern_tile_random_2.sbs/functions/debug_find_cell` - Parity:** All 114,688 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pbr_albedo_safe_color.sbs/pbr_albedo_safe_color` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pbr_dielectric_F0.sbs/pbr_dielectric_F0` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pbr_metal_reflectance.sbs/pbr_metal_reflectance` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pbr_render2.sbs/sat/sat_iteration` - Parity:** All 114,688 output pixels (LUM32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pbr_render_ortho.sbs/get_max_coord` - Parity:** All 4 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`rt_ao.sbs/pp_template/pp_template_64g` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`rt_bent_normal.sbs/pp_template/pp_template_64c` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`rt_irradiance.sbs/template/pp_template` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`rt_shadows.sbs/template/pp_template` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pow.sbs/pow_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`pow.sbs/pow` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize.sbs/quantize_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize.sbs/quantize` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_2.sbs/quantize_grayscale` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_color.sbs/apply_color_palette_sub` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`quantize_color.sbs/get_min_max/GetMaxFromAlpha_v3` - Parity:** All 4 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`quantize_color.sbs/helpers/cdf_pass` - Parity:** All 114,688 output pixels (LUM32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_color.sbs/bitonic_sort/graph/remap_sorting` - Parity:** All 114,688 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_color.sbs/bitonic_sort/graph/remap_sorting_256x1` - Parity:** All 114,688 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_color.sbs/bitonic_sort/graph/bitonic_sort_iteration` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_color.sbs/bitonic_sort/graph/initial_iteration` - Parity:** All 57,344 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`rgb-a_merge.sbs/rgb-a_merge` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`rgb-a_split.sbs/rgb-a_split` - Parity:** All 229,376 output pixels (LUM8, RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map. All output bridges were compared.

- **`rgba_merge.sbs/rgba_merge` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`rgba_split.sbs/rgba_split` - Parity:** All 458,752 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map. All output bridges were compared.

- **`quantize_color.sbs/create_color_palette_16colors` - Parity:** All 1,024 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`quantize_color.sbs/view_color_palette_sub` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`rt_ao_v2.sbs/mip_max_step` - Parity:** All 25,600 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`rt_bent_normal_v2.sbs/mip_max_step` - Parity:** All 25,600 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`rt_shadows_v2.sbs/mip_max_step` - Parity:** All 25,600 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`shape_splatter.sbs/shape_splatter_to_mask` - Parity:** All 114,688 output pixels (LUM32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`shape_splatter.sbs/min_max` - Parity:** All 2,048 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map. All output bridges were compared.

- **`shape_splatter_v2.sbs/bitonic_sort/graph/remap_sorting` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`shape_splatter_v2.sbs/bitonic_sort/graph/bitonic_sort_iteration` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`shape_splatter_v2.sbs/bitonic_sort/graph/initial_iteration` - Parity:** All 57,344 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`shape_splatter_v2.sbs/poisson_disc/poisson_disc_iteration` - Parity:** All 800 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`shape_splatter_v2.sbs/scanline_to_z_order` - Parity:** All 114,688 output pixels (RGBA16) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and an alternate original input map.

- **`shape_splatter_v2.sbs/grid_to_scanline` - Parity:** All 114,688 output pixels (RGBA32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`shape_splatter_v2_companion.sbs/shape_splatter_to_mask` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`summed_area_table.sbs/summed_area_table_iteration` - Parity:** All 114,688 output pixels (LUM32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`summed_area_table_2.sbs/summed_area_table_iteration` - Parity:** All 114,688 output pixels (LUM32F) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`symmetry.sbs/symmetry` - Parity:** All 114,688 output pixels (RGBA8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`threshold.sbs/threshold` - Parity:** All 114,688 output pixels (LUM8) match across 4 recorded cases: defaults at 128x128/256x256, seed 42, and changed exposed parameters.

- **`value_get_size.sbs/get_size_color` - Parity:** All 4 typed outputs match exactly across 4 recorded cases, including an alternate original input or changed exposed parameters.

- **`value_get_size.sbs/get_size_grayscale` - Parity:** All 4 typed outputs match exactly across 4 recorded cases, including an alternate original input or changed exposed parameters.

- **`paths_tools.sbs/Impl/get_number_of_valid_paths` - Parity:** All 4 typed outputs match exactly across 4 recorded cases, including an alternate original input or changed exposed parameters.

- **`paths_tools.sbs/Utils/RelativeToInputPP` - Parity:** All 1,024 output pixels (RGBA32F) match across 4 recorded cases: original 16x16 curve/path buffers, seed 42, and changed exposed parameters. The parent format is f32.

- **`spline_tools.sbs/internal_tools/get_spline_point_data` - Parity:** All 4 typed outputs match exactly across 4 recorded cases, including an alternate original input or changed exposed parameters.

- **`spline_tools.sbs/internal_tools/compute_spline_distance_iteration` - Parity:** All 1,024 output pixels (LUM32F) match across 4 recorded cases: original 16x16 curve/path buffers, seed 42, and changed exposed parameters. The parent format is f32.

- **`spline_tools.sbs/internal_tools/append_splines_sub` - Parity:** All 262,144 output pixels (RGBA32F) match across 4 recorded cases: original 16x16 curve/path buffers, seed 42, and changed exposed parameters. The parent format is f32. All output bridges were compared. All four typed count outputs also match; connectable counts retain their default values.

- **`spline_tools.sbs/-----WIP-----/circular_array_wip` - Parity:** All 1,024 output pixels (RGBA32F) match across 4 recorded cases: original 16x16 curve/path buffers, seed 42, and changed exposed parameters. The parent format is f32. All four typed count outputs also match; connectable counts retain their default values.

- **`spline_tools.sbs/-----WIP-----/displace_spline_by_spline_wip` - Parity:** All 1,024 output pixels (RGBA32F) match across 4 recorded cases: original 16x16 curve/path buffers, seed 42, and changed exposed parameters. The parent format is f32. All four typed count outputs also match; connectable counts retain their default values.

- **`templates.sbs/Templates/Standard_Materials/Simple_Material` - Parity:** All 491,520 output pixels (RGBA8) match across 3 recorded cases: defaults at 128x128/256x256 and seed 42. All output bridges were compared.

- **`templates.sbs/Templates/Standard_Materials/Extended_Material` - Parity:** All 688,128 output pixels (RGBA8) match across 3 recorded cases: defaults at 128x128/256x256 and seed 42. All output bridges were compared.

- **`pbr_render2.sbs/height_adjust` - Parity:** All 98,304 output pixels (LUM8) match across 3 recorded cases: defaults at 128x128/256x256 and seed 42. Connectable value pins retain their defaults; nondefault values require a wired reference fixture and remain unverified.

## Validation

- Final release CLI and native desktop builds pass, including the unassigned-variable, grayscale Pixel Processor, integer-conversion, and output-bridge changes. Formatting, dependency layers, and all ten WebAssembly crate checks pass. An independent inventory rescan agrees with all 546 roots, and all 320 package links resolve.
- Workspace Clippy is blocked by an existing `chunks_exact_to_as_chunks` lint in the text primitive. Clippy for the four changed library crates passes with that existing lint suppressed.
- Native output-format save/reload checks preserve all four formats: u8, u16, f16, and f32. No repository tests were added or test suites run. Pixel and save/reload measurements use disposable files outside the repository. Reference captures use version 16.0.7 with the Direct3D 11 backend; exact-match claims apply to the recorded cases.
