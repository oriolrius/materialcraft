# SBS root graphs

Inventory of the SBS processing graphs in MaterialCraft that do not instantiate another
SBS processing graph. Reusable function calls are pure math and **do not count as dependencies**
for this inventory.

## Scope and method

- Scanned all **549 `.sbs` packages** under [`assets/sbs`](../assets/sbs), including nested package folders.
- Examined **1,025 processing graphs** (`<graph>` elements). The **954 reusable function definitions** (`<function>` elements) are outside this processing-node inventory.
- A root has **zero `<compInstance>` elements** in its graph body. Instances of graphs in the same package and in other packages both disqualify a graph, including disconnected instances.
- Native primitives, embedded Pixel Processor functions, FX-Maps, reusable math function calls, exposed parameters, and resource references do not disqualify a graph.
- Package-level dependency declarations alone do not disqualify a graph: another graph or a math function in that package may need them.
- Included hidden helpers, nested graphs, templates, and work-in-progress graphs. Visibility and category do not affect the criterion.
- Input pins are reported for context: image inputs (type `1` or `2`) and value inputs with `isConnectable=1`. Ordinary exposed parameters are not input pins.
- Paths retain package folders and exact graph identifiers; each link opens the containing SBS package. Paths are relative to `assets/sbs/`.

This is a structural inventory for MaterialCraft's clean-room implementation. It does not
assert that graphs cook successfully or are independent of bitmap resources.

## Results

| Measure | Count |
|---|---:|
| Root processing graphs | 546 |
| Packages containing root graphs | 320 |
| Roots with no input pins | 176 |
| Roots with one or more input pins | 370 |
| Processing graphs excluded because they instantiate another graph | 479 |

Both `white_noise.sbs/white_noise` and `white_noise.sbs/white_noise_fast` qualify.
A graph may accept input pins and still be a root under the dependency criterion.

## Parity tracking

Statuses: **Not checked**, **In progress**, **Parity** (exact pixels or typed values for the linked measured cases), or **Too hard** (reason recorded). Function calls do not count as regular graph dependencies.

The first pass covers all **546 roots**: **166 Parity** and **380 Too hard**. Parity applies only to the recorded cases; it is not a claim of complete functional parity.

## Complete inventory

| SBS package / graph path | Input pins | Hidden in library | Parity status |
|---|---:|---|---|
| [3d_gradient.sbs/3d_linear_gradient](../assets/sbs/3d_gradient.sbs) | 1 | No | [Too hard: remaining float rounding](parity-fixes.md#remaining-differences) |
| [3d_perlin_noise.sbs/3D_perlin_noise](../assets/sbs/3d_perlin_noise.sbs) | 1 | No | [Too hard: 3 pixels remain](parity-fixes.md#remaining-differences) |
| [3d_perlin_noise.sbs/3D_perlin_noise_fractal](../assets/sbs/3d_perlin_noise.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [3d_perlin_noise.sbs/3D_ridged_noise_fractal](../assets/sbs/3d_perlin_noise.sbs) | 1 | No | [Too hard: 3 pixels remain](parity-fixes.md#remaining-differences) |
| [3d_planar_projection.sbs/3d_planar_projection](../assets/sbs/3d_planar_projection.sbs) | 3 | No | [Too hard: sampling boundaries](parity-fixes.md#remaining-differences) |
| [3d_planar_projection.sbs/3d_planar_projection_color](../assets/sbs/3d_planar_projection.sbs) | 3 | No | [Too hard: sampling boundaries](parity-fixes.md#remaining-differences) |
| [3d_simplex_noise.sbs/3d_simplex_noise](../assets/sbs/3d_simplex_noise.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [3d_texture_jump_flood.sbs/3d_texture_jump_flood_iteration](../assets/sbs/3d_texture_jump_flood.sbs) | 1 | Yes | [Parity: 5 cases](parity-fixes.md#verified-cases) |
| [3d_texture_position.sbs/3d_texture_position](../assets/sbs/3d_texture_position.sbs) | 0 | No | [Parity: native 4K atlas](parity-fixes.md#verified-cases) |
| [3d_texture_render.sbs/3d_texture_surface_render_irradiance](../assets/sbs/3d_texture_render.sbs) | 6 | Yes | [Too hard: float rounding](parity-fixes.md#remaining-differences) |
| [3d_texture_render.sbs/3d_texture_surface_render_position](../assets/sbs/3d_texture_render.sbs) | 1 | Yes | [Too hard: 4 camera pixels](parity-fixes.md#remaining-differences) |
| [3d_texture_render.sbs/3d_texture_volume_render_ibl_pass](../assets/sbs/3d_texture_render.sbs) | 6 | Yes | [Too hard: volume integration](parity-fixes.md#remaining-differences) |
| [3d_texture_render.sbs/3d_texture_volume_render_light_pass](../assets/sbs/3d_texture_render.sbs) | 4 | Yes | [Too hard: 8 pixels remain](parity-fixes.md#remaining-differences) |
| [3d_texture_render.sbs/position_debug](../assets/sbs/3d_texture_render.sbs) | 2 | Yes | [Too hard: 34 traced pixels](parity-fixes.md#remaining-differences) |
| [3d_texture_transform.sbs/3d_texture_offset_color](../assets/sbs/3d_texture_transform.sbs) | 1 | No | [Parity: 5 cases](parity-fixes.md#verified-cases) |
| [3d_texture_transform.sbs/3d_texture_offset_grayscale](../assets/sbs/3d_texture_transform.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [3d_viewer.sbs/ibl_specular_contrib_anisotropic/panoramic_lod](../assets/sbs/3d_viewer.sbs) | 1 | Yes | [Too hard: transform filtering](parity-fixes.md#remaining-differences) |
| [3d_volume_mask.sbs/3d_volume_mask](../assets/sbs/3d_volume_mask.sbs) | 1 | No | [Too hard: 62 pixels remain](parity-fixes.md#remaining-differences) |
| [3d_worley_noise.sbs/3D_worley_noise](../assets/sbs/3d_worley_noise.sbs) | 1 | No | [Too hard: noise field mismatch](parity-fixes.md#remaining-differences) |
| [alveolus.sbs/alveolus](../assets/sbs/alveolus.sbs) | 0 | Yes | [Too hard: cell field mismatch](parity-fixes.md#remaining-differences) |
| [angle_to_vector.sbs/angle_to_vector](../assets/sbs/angle_to_vector.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [anisotropic_blur.sbs/anisotropic_blur](../assets/sbs/anisotropic_blur.sbs) | 1 | No | [Too hard: blur kernel](parity-fixes.md#remaining-differences) |
| [anisotropic_blur.sbs/anisotropic_blur_grayscale](../assets/sbs/anisotropic_blur.sbs) | 1 | No | [Too hard: blur kernel](parity-fixes.md#remaining-differences) |
| [atlas_spliter.sbs/crop_and_transform_sub](../assets/sbs/atlas_spliter.sbs) | 7 | Yes | [Too hard: typed-input comparison](parity-fixes.md#remaining-differences) |
| [atlas_spliter.sbs/max_ba](../assets/sbs/atlas_spliter.sbs) | 1 | Yes | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [auto_crop.sbs/min_max](../assets/sbs/auto_crop.sbs) | 1 | Yes | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [auto_levels.sbs/auto_levels](../assets/sbs/auto_levels.sbs) | 1 | No | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [auto_levels.sbs/min_max](../assets/sbs/auto_levels.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [bevel_smooth.sbs/blur_gaussian_grayscale](../assets/sbs/bevel_smooth.sbs) | 1 | Yes | [Too hard: nonzero blur](parity-fixes.md#remaining-differences) |
| [bevel_smooth.sbs/debug/bevel_smooth_debug_position](../assets/sbs/bevel_smooth.sbs) | 1 | Yes | [Too hard: stamp coverage](parity-fixes.md#remaining-differences) |
| [bevel_smooth.sbs/jump_flood_segment_iteration](../assets/sbs/bevel_smooth.sbs) | 1 | Yes | [Too hard: segment selection](parity-fixes.md#remaining-differences) |
| [blackbody.sbs/blackbody](../assets/sbs/blackbody.sbs) | 0 | No | [Too hard: float rounding](parity-fixes.md#remaining-differences) |
| [blend.sbs/add](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 5 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/add_sub](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/color](../assets/sbs/blend.sbs) | 3 | No | [Too hard: color conversion](parity-fixes.md#remaining-differences) |
| [blend.sbs/color_burn](../assets/sbs/blend.sbs) | 3 | No | [Too hard: 7 pixels remain](parity-fixes.md#remaining-differences) |
| [blend.sbs/color_dodge](../assets/sbs/blend.sbs) | 3 | No | [Too hard: 7 pixels remain](parity-fixes.md#remaining-differences) |
| [blend.sbs/copy](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 5 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/difference](../assets/sbs/blend.sbs) | 3 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/dissolve_2](../assets/sbs/blend.sbs) | 3 | No | [Too hard: stochastic pixels](parity-fixes.md#remaining-differences) |
| [blend.sbs/exclusion](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/hard_light](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/linear_burn](../assets/sbs/blend.sbs) | 3 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/luminosity](../assets/sbs/blend.sbs) | 3 | No | [Too hard: 2 pixels remain](parity-fixes.md#remaining-differences) |
| [blend.sbs/max](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/min](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/multiply](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/overlay](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/screen](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/soft_light](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend.sbs/subtract](../assets/sbs/blend.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend_switch.sbs/multi_switch](../assets/sbs/blend_switch.sbs) | 20 | No | [Parity: 5 cases](parity-fixes.md#verified-cases) |
| [blend_switch.sbs/multi_switch_grayscale](../assets/sbs/blend_switch.sbs) | 20 | No | [Parity: 5 cases](parity-fixes.md#verified-cases) |
| [blend_switch.sbs/switch](../assets/sbs/blend_switch.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [blend_switch.sbs/switch_grayscale](../assets/sbs/blend_switch.sbs) | 2 | No | [Parity: 6 cases](parity-fixes.md#verified-cases) |
| [blur_hq.sbs/blur_hq](../assets/sbs/blur_hq.sbs) | 1 | No | [Too hard: blur kernel](parity-fixes.md#remaining-differences) |
| [blur_hq.sbs/blur_hq_grayscale](../assets/sbs/blur_hq.sbs) | 1 | No | [Too hard: blur kernel](parity-fixes.md#remaining-differences) |
| [bnw_spots_1.sbs/bnw_spots_1](../assets/sbs/bnw_spots_1.sbs) | 0 | Yes | [Too hard: noise field](parity-fixes.md#remaining-differences) |
| [bnw_spots_3.sbs/bnw_spots_3](../assets/sbs/bnw_spots_3.sbs) | 0 | Yes | [Too hard: noise field](parity-fixes.md#remaining-differences) |
| [brick_1.sbs/brick_1](../assets/sbs/brick_1.sbs) | 0 | Yes | [Too hard: pattern placement](parity-fixes.md#remaining-differences) |
| [brick_2.sbs/brick_2](../assets/sbs/brick_2.sbs) | 0 | Yes | [Too hard: pattern placement](parity-fixes.md#remaining-differences) |
| [brick_3.sbs/brick_3](../assets/sbs/brick_3.sbs) | 0 | Yes | [Too hard: pattern placement](parity-fixes.md#remaining-differences) |
| [brick_generator.sbs/resources/Tile](../assets/sbs/brick_generator.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [cartesian_polar_transformation.sbs/cartesian_to_polar](../assets/sbs/cartesian_polar_transformation.sbs) | 1 | No | [Too hard: sampling/rounding](parity-fixes.md#remaining-differences) |
| [cartesian_polar_transformation.sbs/cartesian_to_polar_grayscale](../assets/sbs/cartesian_polar_transformation.sbs) | 1 | No | [Too hard: sampling/rounding](parity-fixes.md#remaining-differences) |
| [cartesian_polar_transformation.sbs/polar_to_cartesian](../assets/sbs/cartesian_polar_transformation.sbs) | 1 | No | [Too hard: sampling/rounding](parity-fixes.md#remaining-differences) |
| [cartesian_polar_transformation.sbs/polar_to_cartesian_grayscale](../assets/sbs/cartesian_polar_transformation.sbs) | 1 | No | [Too hard: sampling/rounding](parity-fixes.md#remaining-differences) |
| [cells_1.sbs/cells_1](../assets/sbs/cells_1.sbs) | 0 | Yes | [Too hard: noise field](parity-fixes.md#remaining-differences) |
| [channel_mixer.sbs/channel_mixer](../assets/sbs/channel_mixer.sbs) | 1 | No | [Too hard: function parameter output](parity-fixes.md#remaining-differences) |
| [checker_1.sbs/checker_1](../assets/sbs/checker_1.sbs) | 0 | Yes | [Too hard: scaled filtering](parity-fixes.md#remaining-differences) |
| [checker_2.sbs/checker_2](../assets/sbs/checker_2.sbs) | 0 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [chrominance_extract.sbs/chrominance_extract](../assets/sbs/chrominance_extract.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [clamp.sbs/clamp](../assets/sbs/clamp.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [clamp.sbs/clamp_grayscale](../assets/sbs/clamp.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [clone.sbs/clone](../assets/sbs/clone.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [clone_patch.sbs/transform_from_center](../assets/sbs/clone_patch.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [clone_patch.sbs/transform_to_center](../assets/sbs/clone_patch.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [clone_patch.sbs/transform_to_center_grayscale](../assets/sbs/clone_patch.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [clouds_1.sbs/clouds_1](../assets/sbs/clouds_1.sbs) | 0 | Yes | [Too hard: noise field](parity-fixes.md#remaining-differences) |
| [clouds_2.sbs/clouds_2](../assets/sbs/clouds_2.sbs) | 0 | Yes | [Too hard: noise field](parity-fixes.md#remaining-differences) |
| [color_space_conversion.sbs/acescg_to_linear_srgb](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [color_space_conversion.sbs/acescg_to_srgb](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [color_space_conversion.sbs/linear_srgb_to_acescg](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [color_space_conversion.sbs/linear_srgb_to_srgb](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [color_space_conversion.sbs/linear_srgb_to_srgb_grayscale](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [color_space_conversion.sbs/srgb_to_acescg](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [color_space_conversion.sbs/srgb_to_linear_srgb](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [color_space_conversion.sbs/srgb_to_linear_srgb_grayscale](../assets/sbs/color_space_conversion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [color_temperature_adjustment.sbs/color_temperature_adjustment](../assets/sbs/color_temperature_adjustment.sbs) | 1 | No | [Too hard: 2 pixels remain](parity-fixes.md#remaining-differences) |
| [color_to_mask.sbs/color_to_mask](../assets/sbs/color_to_mask.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [color_to_mask_v2.sbs/color_to_mask](../assets/sbs/color_to_mask_v2.sbs) | 2 | No | [Too hard: 1 pixel remains](parity-fixes.md#remaining-differences) |
| [composite.sbs/composite/background/uniform_color_float3](../assets/sbs/composite.sbs) | 0 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/blends/blend_content](../assets/sbs/composite.sbs) | 3 | Yes | [Too hard: pixel differences](parity-fixes.md#remaining-differences) |
| [composite.sbs/composite/blends/blend_content_grayscale](../assets/sbs/composite.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/blends/blend_content_pre_post_composite](../assets/sbs/composite.sbs) | 2 | Yes | [Too hard: pixel differences](parity-fixes.md#remaining-differences) |
| [composite.sbs/composite/blends/blend_content_pre_post_composite_grayscale](../assets/sbs/composite.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/blends/blend_matte](../assets/sbs/composite.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [composite.sbs/composite/blends/blend_matte_bg_passthrough](../assets/sbs/composite.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/blends/blend_matte_fg_passthrough](../assets/sbs/composite.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/filters/filter_mask](../assets/sbs/composite.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/filters/filter_split_rgb_a](../assets/sbs/composite.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [composite.sbs/composite/filters/filter_transformation](../assets/sbs/composite.sbs) | 1 | Yes | [Too hard: pixel differences](parity-fixes.md#remaining-differences) |
| [composite.sbs/composite/filters/filter_transformation_grayscale](../assets/sbs/composite.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [constant_values.sbs/constant_boolean](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/float/constant_float](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/float/constant_float2](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/float/constant_float3](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/float/constant_float4](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/integer/constant_integer](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/integer/constant_integer2](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/integer/constant_integer3](../assets/sbs/constant_values.sbs) | 0 | No | [Parity: 3 typed values](parity-fixes.md#verified-cases) |
| [constant_values.sbs/integer/constant_integer4](../assets/sbs/constant_values.sbs) | 0 | No | [Too hard: integer limit](parity-fixes.md#remaining-differences) |
| [contrast_luminosity.sbs/contrast_luminosity](../assets/sbs/contrast_luminosity.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [contrast_luminosity.sbs/contrast_luminosity_grayscale](../assets/sbs/contrast_luminosity.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [creased.sbs/creased](../assets/sbs/creased.sbs) | 0 | Yes | [Too hard: noise field](parity-fixes.md#remaining-differences) |
| [crop.sbs/crop](../assets/sbs/crop.sbs) | 1 | No | [Too hard: pixel differences](parity-fixes.md#remaining-differences) |
| [crop.sbs/crop_grayscale](../assets/sbs/crop.sbs) | 1 | No | [Too hard: pixel differences](parity-fixes.md#remaining-differences) |
| [crystal_1.sbs/crystal_1](../assets/sbs/crystal_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [crystal_2.sbs/crystal_2](../assets/sbs/crystal_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [curvature.sbs/curvature](../assets/sbs/curvature.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [curvature_smooth.sbs/curvature_smooth](../assets/sbs/curvature_smooth.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [curvature_smooth_v2.sbs/compute_curvature_v2](../assets/sbs/curvature_smooth_v2.sbs) | 2 | Yes | [Parity: 6 cases](parity-fixes.md#verified-cases) |
| [diffusion.sbs/impl/pass_RGBA](../assets/sbs/diffusion.sbs) | 4 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [diffusion.sbs/impl/pass_UV](../assets/sbs/diffusion.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [diffusion.sbs/impl/pass_gray](../assets/sbs/diffusion.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [diffusion.sbs/impl/pass_normal](../assets/sbs/diffusion.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [directional_distance.sbs/directional_distance](../assets/sbs/directional_distance.sbs) | 3 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [directionnal_noise.sbs/directionnal_noise](../assets/sbs/directionnal_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [dirt_1.sbs/dirt_1](../assets/sbs/dirt_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [dirt_3.sbs/dirt_3](../assets/sbs/dirt_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [dirt_5.sbs/dirt_5](../assets/sbs/dirt_5.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [dirt_6.sbs/dirt_6](../assets/sbs/dirt_6.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [dirt_gradient.sbs/dirt_gradient](../assets/sbs/dirt_gradient.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [exposure.sbs/exposure](../assets/sbs/exposure.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [extend_shape.sbs/extend_shape_color](../assets/sbs/extend_shape.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [extend_shape.sbs/extend_shape_grayscale](../assets/sbs/extend_shape.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [facing_normal.sbs/facing_normal](../assets/sbs/facing_normal.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [fibers_1.sbs/fibers_1](../assets/sbs/fibers_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [flood_fill.sbs/flood_fill_step](../assets/sbs/flood_fill.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill.sbs/flood_fill_to_bbox_size](../assets/sbs/flood_fill.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill.sbs/flood_fill_to_position](../assets/sbs/flood_fill.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill.sbs/flood_fill_to_random_color](../assets/sbs/flood_fill.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [flood_fill.sbs/flood_fill_to_random_grayscale](../assets/sbs/flood_fill.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [flood_fill_2.sbs/flood_fill_impl/flood_fill_step_A](../assets/sbs/flood_fill_2.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill_2.sbs/flood_fill_impl/flood_fill_step_A_switch](../assets/sbs/flood_fill_2.sbs) | 4 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill_2.sbs/flood_fill_impl/flood_fill_step_B](../assets/sbs/flood_fill_2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill_2.sbs/flood_fill_to_bbox_size](../assets/sbs/flood_fill_2.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill_2.sbs/flood_fill_to_color](../assets/sbs/flood_fill_2.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [flood_fill_2.sbs/flood_fill_to_grayscale](../assets/sbs/flood_fill_2.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [flood_fill_2.sbs/flood_fill_to_position](../assets/sbs/flood_fill_2.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [flood_fill_2.sbs/flood_fill_to_random_color](../assets/sbs/flood_fill_2.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [flood_fill_2.sbs/flood_fill_to_random_grayscale](../assets/sbs/flood_fill_2.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [floodfill_to_index.sbs/pass](../assets/sbs/floodfill_to_index.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [fluid.sbs/fluid](../assets/sbs/fluid.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [fractal_sum_base.sbs/fractal_sum_base](../assets/sbs/fractal_sum_base.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [fur_1.sbs/fur_1](../assets/sbs/fur_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [fur_2.sbs/fur_2](../assets/sbs/fur_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [fur_3.sbs/fur_3](../assets/sbs/fur_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [fur_4.sbs/fur_4](../assets/sbs/fur_4.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [fxaa.sbs/fxaa_grayscale](../assets/sbs/fxaa.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [gaussian_1.sbs/gaussian_1](../assets/sbs/gaussian_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gaussian_2.sbs/gaussian_2](../assets/sbs/gaussian_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gaussian_spots_1.sbs/gaussian_spots_1](../assets/sbs/gaussian_spots_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gaussian_spots_2.sbs/gaussian_spots_2](../assets/sbs/gaussian_spots_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [glow.sbs/clamp](../assets/sbs/glow.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [gradient_axial.sbs/gradient_axial](../assets/sbs/gradient_axial.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gradient_axial.sbs/gradient_axial_reflected](../assets/sbs/gradient_axial.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gradient_circular.sbs/gradient_circular](../assets/sbs/gradient_circular.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gradient_linear_1.sbs/gradient_linear_1](../assets/sbs/gradient_linear_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gradient_linear_2.sbs/gradient_linear_2](../assets/sbs/gradient_linear_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gradient_linear_3.sbs/gradient_linear_3](../assets/sbs/gradient_linear_3.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [gradient_radial.sbs/gradient_radial](../assets/sbs/gradient_radial.sbs) | 0 | No | [Too hard: 64 one-level pixels](parity-fixes.md#remaining-differences) |
| [grid_atlas.sbs/grid_atlas_color](../assets/sbs/grid_atlas.sbs) | 16 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [grid_atlas.sbs/grid_atlas_grayscale](../assets/sbs/grid_atlas.sbs) | 16 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [hald_clut.sbs/hald_clut](../assets/sbs/hald_clut.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [hbao.sbs/hbao_mip](../assets/sbs/hbao.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [hbao_2.sbs/hbao_mip](../assets/sbs/hbao_2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [hbao_2.sbs/hbao_subsample](../assets/sbs/hbao_2.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [hdr_exposure_preview.sbs/hdr_exposure_preview](../assets/sbs/hdr_exposure_preview.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [hdr_merge.sbs/average_luminance_merge](../assets/sbs/hdr_merge.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [hdr_range_viewer.sbs/hdr_range_viewer_grayscale](../assets/sbs/hdr_range_viewer.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [height_blend.sbs/height_blend](../assets/sbs/height_blend.sbs) | 3 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [height_extrude.sbs/impl/acceleration_struct_impl](../assets/sbs/height_extrude.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [height_to_normal_world_units.sbs/height_to_normal_world_units](../assets/sbs/height_to_normal_world_units.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [height_to_normal_world_units.sbs/height_to_normal_world_units_2](../assets/sbs/height_to_normal_world_units.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [highpass.sbs/highpass](../assets/sbs/highpass.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [highpass.sbs/highpass_grayscale](../assets/sbs/highpass.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [histogram_range.sbs/histogram_range](../assets/sbs/histogram_range.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [histogram_scan.sbs/histogram_scan](../assets/sbs/histogram_scan.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [histogram_scan_non_uniform.sbs/histogram_scan_non_uniform](../assets/sbs/histogram_scan_non_uniform.sbs) | 3 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [histogram_select.sbs/histogram_select](../assets/sbs/histogram_select.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [histogram_shift.sbs/histogram_shift](../assets/sbs/histogram_shift.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [histogram_tools.sbs/histogram_compute](../assets/sbs/histogram_tools.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [histogram_tools.sbs/horizontal_max](../assets/sbs/histogram_tools.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [id_to_mask.sbs/id_to_mask_color](../assets/sbs/id_to_mask.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [id_to_mask.sbs/id_to_mask_grayscale](../assets/sbs/id_to_mask.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [invert.sbs/invert](../assets/sbs/invert.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [invert.sbs/invert_grayscale](../assets/sbs/invert.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [linear_srgb_conversion.sbs/convert_to_linear](../assets/sbs/linear_srgb_conversion.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [linear_srgb_conversion.sbs/convert_to_linear_grayscale](../assets/sbs/linear_srgb_conversion.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [linear_srgb_conversion.sbs/convert_to_srgb](../assets/sbs/linear_srgb_conversion.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [linear_srgb_conversion.sbs/convert_to_srgb_grayscale](../assets/sbs/linear_srgb_conversion.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [make_it_tile_patch.sbs/make_it_tile_patch](../assets/sbs/make_it_tile_patch.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [make_it_tile_patch_grayscale.sbs/make_it_tile_patch_grayscale](../assets/sbs/make_it_tile_patch_grayscale.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [material_switch.sbs/material_switch](../assets/sbs/material_switch.sbs) | 24 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [material_transforms.sbs/material_transforms](../assets/sbs/material_transforms.sbs) | 5 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [me_water_level.sbs/scratches_2](../assets/sbs/me_water_level.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mean_of_least_variance.sbs/mlv_color_iteration](../assets/sbs/mean_of_least_variance.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mean_of_least_variance.sbs/mlv_grayscale_iteration](../assets/sbs/mean_of_least_variance.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [median_filter.sbs/median_filter_color](../assets/sbs/median_filter.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [median_filter.sbs/median_filter_grayscale](../assets/sbs/median_filter.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [mesh_1.sbs/mesh_1](../assets/sbs/mesh_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mesh_2.sbs/mesh_2](../assets/sbs/mesh_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mesh_adaptive_base_color.sbs/base_color_2_inputs](../assets/sbs/mesh_adaptive_base_color.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [mesh_adaptive_base_color.sbs/base_color_4_inputs](../assets/sbs/mesh_adaptive_base_color.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [mesh_adaptive_base_color.sbs/base_color_8_inputs](../assets/sbs/mesh_adaptive_base_color.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [mesh_adaptive_material_blender.sbs/mesh_material_blender_2_inputs](../assets/sbs/mesh_adaptive_material_blender.sbs) | 7 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mesh_adaptive_material_blender.sbs/mesh_material_blender_4_inputs](../assets/sbs/mesh_adaptive_material_blender.sbs) | 13 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mesh_adaptive_material_blender.sbs/mesh_material_blender_8_inputs](../assets/sbs/mesh_adaptive_material_blender.sbs) | 25 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mesh_data_combiner.sbs/mesh_data_combiner](../assets/sbs/mesh_data_combiner.sbs) | 11 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [messy_fibers_1.sbs/messy_fibers_1](../assets/sbs/messy_fibers_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [messy_fibers_2.sbs/messy_fibers_2](../assets/sbs/messy_fibers_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [messy_fibers_3.sbs/messy_fibers_3](../assets/sbs/messy_fibers_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [messy_fibers_4.sbs/messy_fibers_4](../assets/sbs/messy_fibers_4.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [microscope_view.sbs/microscope_view](../assets/sbs/microscope_view.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [min_max.sbs/min_max](../assets/sbs/min_max.sbs) | 1 | No | [Too hard: remaining typed-value difference](parity-fixes.md#remaining-differences) |
| [moisture_noise.sbs/moisture_noise](../assets/sbs/moisture_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mosaic.sbs/mosaic](../assets/sbs/mosaic.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [mosaic.sbs/mosaic_grayscale](../assets/sbs/mosaic.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [multi_directional_warp.sbs/multi_directional_warp_color](../assets/sbs/multi_directional_warp.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [multi_directional_warp.sbs/multi_directional_warp_grayscale](../assets/sbs/multi_directional_warp.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [multiangle_to_albedo.sbs/multiangle_to_albedo](../assets/sbs/multiangle_to_albedo.sbs) | 8 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [multiangle_to_normal.sbs/angle_to_normal](../assets/sbs/multiangle_to_normal.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [multiangle_to_normal.sbs/angle_weight](../assets/sbs/multiangle_to_normal.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [noise_anisotropic_noise.sbs/anisotropic_noise](../assets/sbs/noise_anisotropic_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_anisotropic_noise_v2.sbs/anisotropic_noise_v2](../assets/sbs/noise_anisotropic_noise_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_bnw_spots_1.sbs/bnw_spots_1](../assets/sbs/noise_bnw_spots_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_bnw_spots_1_v2.sbs/bnw_spots_1](../assets/sbs/noise_bnw_spots_1_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_bnw_spots_3.sbs/bnw_spots_3](../assets/sbs/noise_bnw_spots_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_bnw_spots_3_v2.sbs/bnw_spots_3](../assets/sbs/noise_bnw_spots_3_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_cells_1.sbs/cells_1](../assets/sbs/noise_cells_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_cells_1_v2.sbs/cells_1](../assets/sbs/noise_cells_1_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_cells_4.sbs/cells_4](../assets/sbs/noise_cells_4.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_cells_4_v2.sbs/cells_4](../assets/sbs/noise_cells_4_v2.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_clouds_1.sbs/clouds_1](../assets/sbs/noise_clouds_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_clouds_1_v2.sbs/clouds_1](../assets/sbs/noise_clouds_1_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_clouds_2.sbs/clouds_2](../assets/sbs/noise_clouds_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_clouds_2_v2.sbs/clouds_2](../assets/sbs/noise_clouds_2_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_creased.sbs/creased](../assets/sbs/noise_creased.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_crystal_1.sbs/crystal_1](../assets/sbs/noise_crystal_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_crystal_2.sbs/crystal_2](../assets/sbs/noise_crystal_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_1.sbs/directional_noise_1](../assets/sbs/noise_directional_noise_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_1_v2.sbs/directional_noise_1](../assets/sbs/noise_directional_noise_1_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_2.sbs/directional_noise_2](../assets/sbs/noise_directional_noise_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_2_v2.sbs/directional_noise_2](../assets/sbs/noise_directional_noise_2_v2.sbs) | 0 | No | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_3.sbs/directional_noise_3](../assets/sbs/noise_directional_noise_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_3_v2.sbs/directional_noise_3](../assets/sbs/noise_directional_noise_3_v2.sbs) | 0 | No | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_4.sbs/directional_noise_4](../assets/sbs/noise_directional_noise_4.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_noise_4_v2.sbs/directional_noise_4](../assets/sbs/noise_directional_noise_4_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_scratches.sbs/directional_scratches](../assets/sbs/noise_directional_scratches.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_directional_scratches_v2.sbs/directional_scratches](../assets/sbs/noise_directional_scratches_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_1.sbs/dirt_1](../assets/sbs/noise_dirt_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_1_v2.sbs/dirt_1](../assets/sbs/noise_dirt_1_v2.sbs) | 0 | No | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [noise_dirt_2.sbs/dirt_2](../assets/sbs/noise_dirt_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_3.sbs/dirt_3](../assets/sbs/noise_dirt_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_3_v2.sbs/dirt_3](../assets/sbs/noise_dirt_3_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_4.sbs/dirt_4](../assets/sbs/noise_dirt_4.sbs) | 0 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [noise_dirt_4_v2.sbs/dirt_4](../assets/sbs/noise_dirt_4_v2.sbs) | 0 | No | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [noise_dirt_5.sbs/dirt_5](../assets/sbs/noise_dirt_5.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_5_v2.sbs/dirt_5](../assets/sbs/noise_dirt_5_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_gradient.sbs/dirt_gradient](../assets/sbs/noise_dirt_gradient.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_dirt_gradient_v2.sbs/dirt_gradient](../assets/sbs/noise_dirt_gradient_v2.sbs) | 0 | No | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [noise_fluid.sbs/fluid](../assets/sbs/noise_fluid.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fractal_sum_base.sbs/fractal_sum_base](../assets/sbs/noise_fractal_sum_base.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fractal_sum_base.sbs/fractal_sum_base_2](../assets/sbs/noise_fractal_sum_base.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fractal_sum_base_v2.sbs/noise_blend](../assets/sbs/noise_fractal_sum_base_v2.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fractal_sum_base_v2.sbs/noise_disorder](../assets/sbs/noise_fractal_sum_base_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [noise_fractal_sum_base_v2.sbs/noise_upscale](../assets/sbs/noise_fractal_sum_base_v2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fur_1.sbs/fur_1](../assets/sbs/noise_fur_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fur_2.sbs/fur_2](../assets/sbs/noise_fur_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_fur_3.sbs/fur_3](../assets/sbs/noise_fur_3.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_gaussian_noise.sbs/gaussian_noise](../assets/sbs/noise_gaussian_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_gaussian_noise_v2.sbs/gaussian_noise](../assets/sbs/noise_gaussian_noise_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_gaussian_spots_1.sbs/gaussian_spots_1](../assets/sbs/noise_gaussian_spots_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_gaussian_spots_1_v2.sbs/gaussian_spots_1](../assets/sbs/noise_gaussian_spots_1_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_gaussian_spots_2.sbs/gaussian_spots_2](../assets/sbs/noise_gaussian_spots_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_gaussian_spots_2_v2.sbs/gaussian_spots_2](../assets/sbs/noise_gaussian_spots_2_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_liquid.sbs/liquid](../assets/sbs/noise_liquid.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_messy_fibers_1.sbs/messy_fibers_1](../assets/sbs/noise_messy_fibers_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_messy_fibers_1_v2.sbs/messy_fibers_1](../assets/sbs/noise_messy_fibers_1_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_messy_fibers_2.sbs/messy_fibers_2](../assets/sbs/noise_messy_fibers_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_messy_fibers_2_v2.sbs/messy_fibers_2](../assets/sbs/noise_messy_fibers_2_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_messy_fibers_3.sbs/messy_fibers_3](../assets/sbs/noise_messy_fibers_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_messy_fibers_3_v2.sbs/messy_fibers_3](../assets/sbs/noise_messy_fibers_3_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_microscope_view.sbs/microscope_view](../assets/sbs/noise_microscope_view.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_moisture_noise.sbs/moisture_noise](../assets/sbs/noise_moisture_noise.sbs) | 0 | Yes | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [noise_moisture_noise_1_v2.sbs/moisture_noise_1](../assets/sbs/noise_moisture_noise_1_v2.sbs) | 0 | No | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [noise_moisture_noise_2.sbs/moisture_noise_2](../assets/sbs/noise_moisture_noise_2.sbs) | 0 | No | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [noise_perlin_noise.sbs/perlin_noise](../assets/sbs/noise_perlin_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_perlin_noise_v2.sbs/perlin_noise](../assets/sbs/noise_perlin_noise_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_plasma.sbs/plasma](../assets/sbs/noise_plasma.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_upscale_1.sbs/noise_upscale_1](../assets/sbs/noise_upscale_1.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_upscale_2.sbs/noise_upscale_2](../assets/sbs/noise_upscale_2.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_upscale_3.sbs/noise_upscale_3](../assets/sbs/noise_upscale_3.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_voronoi.sbs/3d_voronoi](../assets/sbs/noise_voronoi.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_voronoi.sbs/3d_voronoi_fractal](../assets/sbs/noise_voronoi.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_voronoi.sbs/voronoi](../assets/sbs/noise_voronoi.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [noise_voronoi.sbs/voronoi_fractal](../assets/sbs/noise_voronoi.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [non_square_transform.sbs/non_square_transform](../assets/sbs/non_square_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [non_square_transform.sbs/non_square_transform_greyscale](../assets/sbs/non_square_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [non_uniform_blur.sbs/non_uniform_blur_grayscale_sample](../assets/sbs/non_uniform_blur.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [non_uniform_blur.sbs/non_uniform_blur_sample](../assets/sbs/non_uniform_blur.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [non_uniform_rotation.sbs/non_uniform_rotation_color](../assets/sbs/non_uniform_rotation.sbs) | 3 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [non_uniform_rotation.sbs/non_uniform_rotation_grayscale](../assets/sbs/non_uniform_rotation.sbs) | 3 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_color.sbs/normal_color](../assets/sbs/normal_color.sbs) | 0 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [normal_intensity.sbs/normal_intensity](../assets/sbs/normal_intensity.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [normal_invert.sbs/normal_invert](../assets/sbs/normal_invert.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [normal_mapper.sbs/normal_mapper](../assets/sbs/normal_mapper.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [normal_normalise.sbs/normal_normalise](../assets/sbs/normal_normalise.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_sobel.sbs/normal_sobel](../assets/sbs/normal_sobel.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_sobel.sbs/normal_sobel_2](../assets/sbs/normal_sobel.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_to_height_2.sbs/normal_to_height](../assets/sbs/normal_to_height_2.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_to_height_hq.sbs/base_algorithm_height_distance](../assets/sbs/normal_to_height_hq.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_transform.sbs/normal_transform](../assets/sbs/normal_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [normal_uncombine.sbs/get_intensity_iteration](../assets/sbs/normal_uncombine.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [normal_vector_rotation.sbs/normal_vector_rotation](../assets/sbs/normal_vector_rotation.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [panorama_gradients.sbs/panorama_gradient_2_points](../assets/sbs/panorama_gradients.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_gradients.sbs/panorama_gradient_linear](../assets/sbs/panorama_gradients.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_nadir_extract.sbs/nadir_extract](../assets/sbs/panorama_nadir_extract.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_nadir_patch.sbs/panorama_nadir_patch](../assets/sbs/panorama_nadir_patch.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_physical_sun_sky.sbs/Impl/GetOneParam](../assets/sbs/panorama_physical_sun_sky.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_position.sbs/panorama_3d_position](../assets/sbs/panorama_position.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_rotation.sbs/panorama_rotation](../assets/sbs/panorama_rotation.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_shape.sbs/panorama_line_light](../assets/sbs/panorama_shape.sbs) | 3 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_shape.sbs/panorama_plane_light](../assets/sbs/panorama_shape.sbs) | 3 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_shape.sbs/panorama_shape](../assets/sbs/panorama_shape.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_shape.sbs/panorama_shape_2](../assets/sbs/panorama_shape.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_shape.sbs/panorama_sphere_light](../assets/sbs/panorama_shape.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_shape.sbs/panorama_spot](../assets/sbs/panorama_shape.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [panorama_straighten_horizon.sbs/panorama_straighten_horizon](../assets/sbs/panorama_straighten_horizon.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paper_path.sbs/paper_path](../assets/sbs/paper_path.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [parquet_1.sbs/parquet_1](../assets/sbs/parquet_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/Impl/compute_paths_lengths](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/Impl/facets_to_paths_impl_link_template](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/Impl/facets_to_paths_impl_stitch_template](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/Impl/get_number_of_valid_paths](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [paths_tools.sbs/Impl/mask_to_facets](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/Impl/sort_by_blue](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/Utils/RelativeToInputPP](../assets/sbs/paths_tools.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [paths_tools.sbs/paths_polygon](../assets/sbs/paths_tools.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/paths_trace](../assets/sbs/paths_tools.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/vertex_processor](../assets/sbs/paths_tools.sbs) | 9 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [paths_tools.sbs/vertex_processor_simple](../assets/sbs/paths_tools.sbs) | 5 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_alveolus.sbs/alveolus](../assets/sbs/pattern_alveolus.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_arc_pavement.sbs/arc_pavement](../assets/sbs/pattern_arc_pavement.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_brick_1.sbs/brick_1](../assets/sbs/pattern_brick_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_brick_2.sbs/brick_2](../assets/sbs/pattern_brick_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_brick_generator.sbs/resources/Tile](../assets/sbs/pattern_brick_generator.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pattern_checker_1.sbs/checker_1](../assets/sbs/pattern_checker_1.sbs) | 0 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pattern_cube_3d.sbs/cube_3d](../assets/sbs/pattern_cube_3d.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_cube_3d.sbs/cube_3d_gbuffers](../assets/sbs/pattern_cube_3d.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_fibers_1.sbs/fibers_1](../assets/sbs/pattern_fibers_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_fibers_2.sbs/fibers_2](../assets/sbs/pattern_fibers_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_gaussian_1.sbs/gaussian_1](../assets/sbs/pattern_gaussian_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_gaussian_2.sbs/gaussian_2](../assets/sbs/pattern_gaussian_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_mesh_1.sbs/mesh_1](../assets/sbs/pattern_mesh_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_mesh_2.sbs/mesh_2](../assets/sbs/pattern_mesh_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_polygon_2.sbs/polygon_2](../assets/sbs/pattern_polygon_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_shape.sbs/hemisphere](../assets/sbs/pattern_shape.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_shape.sbs/shape_base](../assets/sbs/pattern_shape.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_splatter_circular.sbs/splatter_circular](../assets/sbs/pattern_splatter_circular.sbs) | 7 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_splatter_circular.sbs/splatter_circular_color](../assets/sbs/pattern_splatter_circular.sbs) | 7 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_stripes.sbs/stripes](../assets/sbs/pattern_stripes.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_tile_generator.sbs/tile_generator](../assets/sbs/pattern_tile_generator.sbs) | 7 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_tile_generator.sbs/tile_generator_color](../assets/sbs/pattern_tile_generator.sbs) | 7 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_tile_random.sbs/tile_random](../assets/sbs/pattern_tile_random.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_tile_random.sbs/tile_random_color](../assets/sbs/pattern_tile_random.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_tile_random_2.sbs/functions/debug_find_cell](../assets/sbs/pattern_tile_random_2.sbs) | 0 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pattern_tile_random_2.sbs/tile_random_2](../assets/sbs/pattern_tile_random_2.sbs) | 5 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_tile_sampler.sbs/arrow](../assets/sbs/pattern_tile_sampler.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_triangle_grid.sbs/triangle_grid_base](../assets/sbs/pattern_triangle_grid.sbs) | 3 | Yes | [Too hard: measurement timeout](parity-fixes.md#remaining-differences) |
| [pattern_weave_1.sbs/weave_1](../assets/sbs/pattern_weave_1.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pattern_weave_2.sbs/weave_2](../assets/sbs/pattern_weave_2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pavement_1.sbs/pavement_1](../assets/sbs/pavement_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_albedo_safe_color.sbs/pbr_albedo_safe_color](../assets/sbs/pbr_albedo_safe_color.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pbr_dielectric_F0.sbs/pbr_dielectric_F0](../assets/sbs/pbr_dielectric_F0.sbs) | 0 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pbr_metal_reflectance.sbs/pbr_metal_reflectance](../assets/sbs/pbr_metal_reflectance.sbs) | 0 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pbr_render.sbs/PBR_render_mapping_color](../assets/sbs/pbr_render.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/PBR_render_mapping_grayscale](../assets/sbs/pbr_render.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/chroma_shift](../assets/sbs/pbr_render.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/environment_map_rescale](../assets/sbs/pbr_render.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/get_max_coords/get_max_coord](../assets/sbs/pbr_render.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/ibl_specular_contrib_anisotropic/panoramic_lod](../assets/sbs/pbr_render.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/position_to_normal](../assets/sbs/pbr_render.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/raytrace_position](../assets/sbs/pbr_render.sbs) | 4 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/raytraced_irradiance_pass_8](../assets/sbs/pbr_render.sbs) | 10 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render.sbs/raytraced_specular_pass_8](../assets/sbs/pbr_render.sbs) | 9 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/PBR_render_mapping_color](../assets/sbs/pbr_render2.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/PBR_render_mapping_grayscale](../assets/sbs/pbr_render2.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/chroma_shift](../assets/sbs/pbr_render2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/environment_map_rescale](../assets/sbs/pbr_render2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/get_max_coords/get_max_coord](../assets/sbs/pbr_render2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/height_adjust](../assets/sbs/pbr_render2.sbs) | 3 | Yes | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [pbr_render2.sbs/ibl_specular_contrib_anisotropic/panoramic_lod](../assets/sbs/pbr_render2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/lod](../assets/sbs/pbr_render2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/position_to_normal](../assets/sbs/pbr_render2.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/raytrace_position](../assets/sbs/pbr_render2.sbs) | 4 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/raytraced_irradiance_is_pass_8](../assets/sbs/pbr_render2.sbs) | 9 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/raytraced_specular_pass_8](../assets/sbs/pbr_render2.sbs) | 9 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render2.sbs/sat/sat_iteration](../assets/sbs/pbr_render2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pbr_render_ortho.sbs/blue_noise](../assets/sbs/pbr_render_ortho.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render_ortho.sbs/environment_map_rescale](../assets/sbs/pbr_render_ortho.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render_ortho.sbs/get_max_coord](../assets/sbs/pbr_render_ortho.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pbr_render_ortho.sbs/ibl_shadow_pass](../assets/sbs/pbr_render_ortho.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render_ortho.sbs/mipmap_generator](../assets/sbs/pbr_render_ortho.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render_ortho.sbs/rotate_environment](../assets/sbs/pbr_render_ortho.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pbr_render_ortho.sbs/sphere_displace](../assets/sbs/pbr_render_ortho.sbs) | 1 | Yes | [Too hard: cook timeout](parity-fixes.md#remaining-differences) |
| [pbr_render_ortho.sbs/vector_shadow_pass](../assets/sbs/pbr_render_ortho.sbs) | 4 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [perlin_noise_zoom.sbs/perlin_noise_zoom](../assets/sbs/perlin_noise_zoom.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [pow.sbs/pow](../assets/sbs/pow.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pow.sbs/pow_grayscale](../assets/sbs/pow.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [pre_multiply_straight.sbs/premult_to_straight](../assets/sbs/pre_multiply_straight.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quad_transform.sbs/quad_transform](../assets/sbs/quad_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quad_transform.sbs/quad_transform_grayscale](../assets/sbs/quad_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quantize.sbs/quantize](../assets/sbs/quantize.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize.sbs/quantize_grayscale](../assets/sbs/quantize.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_2.sbs/quantize_grayscale](../assets/sbs/quantize_2.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/apply_color_palette_sub](../assets/sbs/quantize_color.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/bitonic_sort/graph/bitonic_sort_iteration](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/bitonic_sort/graph/initial_iteration](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/bitonic_sort/graph/remap_sorting](../assets/sbs/quantize_color.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/bitonic_sort/graph/remap_sorting_256x1](../assets/sbs/quantize_color.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/color_conversions/convert_Lab_to_RGB](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quantize_color.sbs/color_conversions/convert_RGB_to_Lab](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quantize_color.sbs/create_color_palette_16colors](../assets/sbs/quantize_color.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/get_min_max/GetMaxFromAlpha_v2](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quantize_color.sbs/get_min_max/GetMaxFromAlpha_v3](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/helpers/cdf_pass](../assets/sbs/quantize_color.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [quantize_color.sbs/kmeans_iteration](../assets/sbs/quantize_color.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quantize_color.sbs/modify_color_palette_sub](../assets/sbs/quantize_color.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [quantize_color.sbs/view_color_palette_sub](../assets/sbs/quantize_color.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [radial_blur.sbs/radial_blur_grayscale](../assets/sbs/radial_blur.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [replace_color.sbs/replace_color](../assets/sbs/replace_color.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [rgb-a_merge.sbs/rgb-a_merge](../assets/sbs/rgb-a_merge.sbs) | 2 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rgb-a_split.sbs/rgb-a_split](../assets/sbs/rgb-a_split.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rgba_merge.sbs/rgba_merge](../assets/sbs/rgba_merge.sbs) | 4 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rgba_split.sbs/rgba_split](../assets/sbs/rgba_split.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_ao.sbs/pp_template/pp_template_64g](../assets/sbs/rt_ao.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_ao_v2.sbs/mip_max_step](../assets/sbs/rt_ao_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_bent_normal.sbs/pp_template/pp_template_64c](../assets/sbs/rt_bent_normal.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_bent_normal_v2.sbs/mip_max_step](../assets/sbs/rt_bent_normal_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_caustics.sbs/rt_caustics_base](../assets/sbs/rt_caustics.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [rt_irradiance.sbs/compute_irradiance](../assets/sbs/rt_irradiance.sbs) | 5 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [rt_irradiance.sbs/compute_ray_direction](../assets/sbs/rt_irradiance.sbs) | 4 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [rt_irradiance.sbs/debug_hit_uv](../assets/sbs/rt_irradiance.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [rt_irradiance.sbs/mip_max](../assets/sbs/rt_irradiance.sbs) | 1 | Yes | [Too hard: output size differs](parity-fixes.md#remaining-differences) |
| [rt_irradiance.sbs/panoramic_lod](../assets/sbs/rt_irradiance.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [rt_irradiance.sbs/template/pp_template](../assets/sbs/rt_irradiance.sbs) | 3 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_shadows.sbs/mip_max](../assets/sbs/rt_shadows.sbs) | 1 | Yes | [Too hard: output size differs](parity-fixes.md#remaining-differences) |
| [rt_shadows.sbs/template/pp_template](../assets/sbs/rt_shadows.sbs) | 4 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [rt_shadows_v2.sbs/mip_max_step](../assets/sbs/rt_shadows_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [safe_transform.sbs/safe_transform](../assets/sbs/safe_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [safe_transform.sbs/safe_transform_grayscale](../assets/sbs/safe_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [scratches_1.sbs/scratches_1](../assets/sbs/scratches_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [scratches_2.sbs/scratches_2](../assets/sbs/scratches_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [scratches_4.sbs/scratches_4](../assets/sbs/scratches_4.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape.sbs/shape_base](../assets/sbs/shape.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_mapper.sbs/shape_mapper](../assets/sbs/shape_mapper.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_splatter.sbs/min_max](../assets/sbs/shape_splatter.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter.sbs/shape_splatter_blend](../assets/sbs/shape_splatter.sbs) | 12 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_splatter.sbs/shape_splatter_blend_color](../assets/sbs/shape_splatter.sbs) | 12 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_splatter.sbs/shape_splatter_data_extract](../assets/sbs/shape_splatter.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_splatter.sbs/shape_splatter_to_mask](../assets/sbs/shape_splatter.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/bitonic_sort/graph/bitonic_sort_iteration](../assets/sbs/shape_splatter_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/bitonic_sort/graph/initial_iteration](../assets/sbs/shape_splatter_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/bitonic_sort/graph/remap_sorting](../assets/sbs/shape_splatter_v2.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/grid_to_scanline](../assets/sbs/shape_splatter_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/mipmap_tools/Mipmap_N_from_O_color](../assets/sbs/shape_splatter_v2.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_splatter_v2.sbs/poisson_disc/poisson_disc_iteration](../assets/sbs/shape_splatter_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/scanline_to_z_order](../assets/sbs/shape_splatter_v2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [shape_splatter_v2.sbs/shape_splatter_compute_normal](../assets/sbs/shape_splatter_v2.sbs) | 10 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [shape_splatter_v2_companion.sbs/shape_splatter_to_mask](../assets/sbs/shape_splatter_v2_companion.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [skew.sbs/skew](../assets/sbs/skew.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [skew.sbs/skew_grayscale](../assets/sbs/skew.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [slope_blur.sbs/slope_blur](../assets/sbs/slope_blur.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [slope_blur.sbs/slope_blur_grayscale](../assets/sbs/slope_blur.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [slope_blur.sbs/slope_blur_grayscale_2](../assets/sbs/slope_blur.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [smart_auto_tile.sbs/pattern_find_edges_step](../assets/sbs/smart_auto_tile.sbs) | 4 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [splatter.sbs/splatter](../assets/sbs/splatter.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [splatter_circular.sbs/splatter_circular](../assets/sbs/splatter_circular.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [splatter_circular.sbs/splatter_circular_color](../assets/sbs/splatter_circular.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [splatter_color.sbs/splatter_color](../assets/sbs/splatter_color.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/-----WIP-----/circular_array_wip](../assets/sbs/spline_tools.sbs) | 2 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [spline_tools.sbs/-----WIP-----/displace_spline_by_spline_wip](../assets/sbs/spline_tools.sbs) | 4 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [spline_tools.sbs/internal_tools/2dtransform_spline_sub](../assets/sbs/spline_tools.sbs) | 3 | Yes | [Too hard: value-input case not verified](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/append_splines_sub](../assets/sbs/spline_tools.sbs) | 6 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [spline_tools.sbs/internal_tools/compute_bridge_distance](../assets/sbs/spline_tools.sbs) | 3 | Yes | [Too hard: value-input case not verified](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/compute_spline_distance_iteration](../assets/sbs/spline_tools.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [spline_tools.sbs/internal_tools/compute_spline_length_init](../assets/sbs/spline_tools.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/compute_spline_tangents](../assets/sbs/spline_tools.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/get_spline_point_data](../assets/sbs/spline_tools.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [spline_tools.sbs/internal_tools/modify_spline_property_sub](../assets/sbs/spline_tools.sbs) | 4 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/normalize_spline_distance](../assets/sbs/spline_tools.sbs) | 1 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/points_preview](../assets/sbs/spline_tools.sbs) | 3 | Yes | [Too hard: value-input case not verified](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/resample_spline_data](../assets/sbs/spline_tools.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/scatter_splines_on_splines_preprocess](../assets/sbs/spline_tools.sbs) | 5 | Yes | [Too hard: value-input case not verified](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/select_spline_sub](../assets/sbs/spline_tools.sbs) | 3 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spline_tools.sbs/internal_tools/spline_warp_sub](../assets/sbs/spline_tools.sbs) | 6 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [spot_emboss.sbs/spot_emboss](../assets/sbs/spot_emboss.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [starburst.sbs/starburst](../assets/sbs/starburst.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [stripes.sbs/stripes](../assets/sbs/stripes.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [summed_area_table.sbs/summed_area_table_iteration](../assets/sbs/summed_area_table.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [summed_area_table_2.sbs/summed_area_table_iteration](../assets/sbs/summed_area_table_2.sbs) | 1 | Yes | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [swirl.sbs/swirl](../assets/sbs/swirl.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [swirl.sbs/swirl_grayscale](../assets/sbs/swirl.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [symmetry.sbs/symmetry](../assets/sbs/symmetry.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [symmetry_slice.sbs/symmetry_slice](../assets/sbs/symmetry_slice.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [templates.sbs/Templates/Standard_Materials/Extended_Material](../assets/sbs/templates.sbs) | 0 | Yes | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [templates.sbs/Templates/Standard_Materials/Simple_Material](../assets/sbs/templates.sbs) | 0 | Yes | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [threshold.sbs/threshold](../assets/sbs/threshold.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [tile_generator.sbs/tile_generator](../assets/sbs/tile_generator.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [tile_generator_color.sbs/tile_generator_color](../assets/sbs/tile_generator_color.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [tile_random.sbs/tile_random](../assets/sbs/tile_random.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [tile_random.sbs/tile_random_color](../assets/sbs/tile_random.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [tile_sampler.sbs/arrow](../assets/sbs/tile_sampler.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [tileable_material_to_mesh_material.sbs/tileable_material_to_mesh_material](../assets/sbs/tileable_material_to_mesh_material.sbs) | 5 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [trapezoid_transform.sbs/trapezoid_transform](../assets/sbs/trapezoid_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [trapezoid_transform.sbs/trapezoid_transform_grayscale](../assets/sbs/trapezoid_transform.sbs) | 1 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [uv_mapper.sbs/uv_mapper_color_sub](../assets/sbs/uv_mapper.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [uv_mapper.sbs/uv_mapper_grayscale_sub](../assets/sbs/uv_mapper.sbs) | 2 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [value_get_size.sbs/get_size_color](../assets/sbs/value_get_size.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [value_get_size.sbs/get_size_grayscale](../assets/sbs/value_get_size.sbs) | 1 | No | [Parity: 4 cases](parity-fixes.md#verified-cases) |
| [vector_morph.sbs/vector_morph](../assets/sbs/vector_morph.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [vector_morph.sbs/vector_morph_grayscale](../assets/sbs/vector_morph.sbs) | 2 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [vertical_noise.sbs/vertical_noise](../assets/sbs/vertical_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [waveform_01_v2.sbs/waveform_01_v2](../assets/sbs/waveform_01_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [waveform_1.sbs/waveform_1](../assets/sbs/waveform_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [weave_1.sbs/weave_1](../assets/sbs/weave_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [weave_2.sbs/weave_2](../assets/sbs/weave_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [weave_3.sbs/weave_3](../assets/sbs/weave_3.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [weave_4.sbs/weave_4](../assets/sbs/weave_4.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [white_noise.sbs/white_noise](../assets/sbs/white_noise.sbs) | 0 | Yes | [Parity: 3 cases](parity-fixes.md#verified-cases) |
| [white_noise.sbs/white_noise_fast](../assets/sbs/white_noise.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [white_noise_v2.sbs/white_noise_v2](../assets/sbs/white_noise_v2.sbs) | 0 | No | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [wood_fibers_1.sbs/wood_fibers_1](../assets/sbs/wood_fibers_1.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
| [wood_fibers_2.sbs/wood_fibers_2](../assets/sbs/wood_fibers_2.sbs) | 0 | Yes | [Too hard: remaining pixel differences](parity-fixes.md#remaining-differences) |
