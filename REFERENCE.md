# Parameter file reference

- `img_width` _(int)_ and `img_height` _(int)_: Set image width and height.

- `render`: Set render mode and render options. Available values are:

  - `Frame`: Render an image.

    - `zoom` _(float)_: Set zoom. A smaller number means a deeper zoom.

    - `center_x` _(float)_ and `center_y` _(float)_: Set the position of the center of the render area.

      > This corresponds to coordinates of the center of the render area in the complex plane: `z = center_x + i * center_y`

    - `fractal_kind`: Set the fractal you want to draw. Available options are:

      - `Mandelbrot`
      - `MandelbrotCustomExp(exp: float)`
      - `SecondDegreeRecWithGrowingExponent`
      - `SecondDegreeRecWithGrowingExponentParam(a_re: float, a_im: float)`
      - `SecondDegreeRecAlternating1WithGrowingExponent`
      - `ThirdDegreeRecWithGrowingExponent`
      - `NthDegreeRecWithGrowingExponent(n)`
      - `ThirdDegreeRecPairs`
      - `SecondDegreeThirtySevenBlend`
      - `ComplexLogisticMapLike(re: float, im: float)`

  - `Animation`: Render the frames of an animation.

    This mode uses `RenderStep` arrays to perform transitions between float values. `RenderStep` has three possible options: `Const(start_time, end_time, value)`, `Linear(start_time, end_time, start_value, end_value)` and `Smooth(start_time, end_time, start_value, end_value)`.
    See [gqwzzr.ron](./presets/gqwzzr.ron) for an example.

    - `zoom` _([RenderStep])_: Set zoom. A smaller number means a deeper zoom.

    - `center_x` _([RenderStep])_ and `center_y` _(RenderStep)_: Set the position of the center of the render area.

    - `fractal_kind`: Set the fractal you want to draw. Available options are:

      - `Mandelbrot`
      - `MandelbrotCustomExp ( exp: [RenderStep] )`
      - `SecondDegreeRecWithGrowingExponent`
      - `SecondDegreeRecWithGrowingExponentParam ( a_re: [RenderStep], a_im: [RenderStep] )`
      - `SecondDegreeRecAlternating1WithGrowingExponent`
      - `ThirdDegreeRecWithGrowingExponent`
      - `NthDegreeRecWithGrowingExponent(n)`
      - `ThirdDegreeRecPairs`
      - `SecondDegreeThirtySevenBlend`
      - `ComplexLogisticMapLike ( re: [RenderStep], im: [RenderStep] )`

    - `duration` _(float)_: The duration of the animation (in seconds).

    - `fps` _(float)_: The number of frames per second.

- `max_iter` _(int)_: Set the maximum iteration count (around 80000 recommended except for fractals with slow divergence parts such as Mandelbrot where you should settle for ~1000).

- `coloring_mode`: Set the way pixels are colored. Available options are:

  - `CumulativeHistogram(map)`: More information [here](https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Histogram_coloring).
  - `MaxNorm(max, map)`: Normalizes the value based on the provided (optional) max value or the highest iteration count reached while sampling.
  - `MinMaxNorm(min, max, map)`: Performs min-max normalization using the provided (optional) min and max or using the lowest and the highest iteration counts reached while sampling.
  - `BlackAndWhite`: Draws a pixel black if the maximum iteration count (`max_iter`) has been reached, otherwise white.

  Here, all `map` fields must be one of the following options:

  - `Linear`
  - `Squared`
  - `Powf(exp)`

- `sampling`: Set sampling options.

  - `level`: Set sampling level: higher values take more samples and (hopefully) give a smoother result. Available options are:
    - `Exploration`
    - `Low`
    - `Medium`
    - `High`
    - `Ultra`
    - `Extreme`
    - `Extreme1`
    - `Extreme2`
    - `Extreme3`
  - `random_offsets` _(bool)_: Enable or disable random offsets. They are used to get rid of moiré patterns but they make noise appear on some fractals so it might be useful to disable them.

- `custom_gradient` _(optional)_: Set a custom gradient. This is an array of array of the form `[t, [r, g, b]]` where `t` is a float between 0 and 1 and `r`, `g`, `b` the color at that point in the gradient. Colors in between are interpolated.

  Example:

  ```rust
  (
    ..
    custom_gradient: Some([
        (0., (10, 2, 20)),
        (0.1, (200, 40, 230)),
        (0.25, (20, 160, 230)),
        (0.4, (60, 230, 80)),
        (0.55, (255, 230, 20)),
        (0.7, (255, 120, 20)),
        (0.85, (255, 40, 60)),
        (0.95, (2, 0, 4))
    ]),
    ..
  )
  ```

- `diverging_areas` _(optional)_: This allows setting areas where computing pixel values will be skipped assuming they diverge.

  ```rust
  (
    ..
    diverging_areas: Some([
        (min_x, max_x, min_y, max_y),
        ..
    ]),
    ..
  )
  ```

- `dev_options` _(optional)_: For development purposes.

  - `save_sampling_pattern` _(bool)_: Save the sampling pattern as an image.

  - `display_gradient` _(bool)_: Draw the gradient used for coloring in the bottom right corner of the image.
