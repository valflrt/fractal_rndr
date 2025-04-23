# Fractal Renderer

This is a program used to render fractals using a [RON](https://docs.rs/ron/latest/ron/) parameter file. It also has a very simple gui for an easier navigation.

### Table of contents

- [Fractal Renderer](#fractal-renderer)
- [How to install](#how-to-install)
- [How to use](#how-to-use)
- [Preset renders](#preset-renders)
- [Ideas](#ideas)
- [Notes](#notes)

# How to install

If you have cargo installed:

```
cargo install fractal_rndr
```

Otherwise, you can download the latest executable from the [releases tab](https://github.com/valflrt/fractal_rndr/releases/latest).

# How to use

Start the app using:

```
fractal_rndr path/to/param_file.ron path/to/output_image.png --gui
```

> [!NOTE]
> Supported image formats are png and jpg (the extension is used to guess the format)

This is what the app looks like:

![gui](/img/gui.png)

If there is no parameter file, the app will start with a preset fractal. You can save the parameter file anytime by clicking the corresponding button.

Enjoy !

# Preset renders

These are preset renders I like, you can access their parameter files by clicking on the title. There are some more in [`presets/`](presets/).

> The renders found in `presets/` are licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)

### [cyggmf.ron](presets/cyggmf.ron)

> Fractal: `Tdrge`

![cyggmf.png](presets/cyggmf.png)

### [iabppp.ron](presets/iabppp.ron)

> Fractal: `Mandelbrot`

![iabppp.png](presets/iabppp.png)

### [ukhbrp.ron](presets/ukhbrp.ron)

> Fractal: `Tdrge`

![ukhbrp.png](presets/ukhbrp.png)

### [ffayuk.ron](presets/ffayuk.ron)

> Fractal: `Sdrge`

![ffayuk.png](presets/ffayuk.png)

### [mzfyje.ron](presets/mzfyje.ron)

> Fractal: `Sdrge`

![mzfyje.png](presets/mzfyje.png)

### [ecwfwb.ron](presets/ecwfwb.ron)

> Fractal: `SdrgeParam`

https://github.com/user-attachments/assets/a328b7b6-0e66-490a-9a35-ef8e93178f94

### [txktfj.ron](presets/txktfj.ron)

> Fractal: `SdrgeCustomExp(8)`

![txktfj.png](presets/txktfj.png)

### [xvebhd.ron](presets/xvebhd.ron)

> Fractal: `Sdrge`

![xvebhd.png](presets/xvebhd.png)

### [quhuap.ron](presets/quhuap.ron)

> Fractal: `Iigdzh`

![quhuap.png](presets/quhuap.png)

### [ebidxr.ron](presets/ebidxr.ron)

> Fractal: `Tdrge`

![ebidxr.png](presets/ebidxr.png)

### [ajwrkx.ron](presets/ajwrkx.ron)

> Fractal: `Mjygzr`

![ajwrkx.png](presets/ajwrkx.png)

### [wztpft.ron](presets/wztpft.ron)

> Fractal: `Vshqwj`

![wztpft.png](presets/wztpft.png)

### [dvzrjn.ron](presets/dvzrjn.ron)

> Fractal: `Iigdzh`

![dvzrjn.png](presets/dvzrjn.png)

### [ckvjjj.ron](presets/ckvjjj.ron)

> Fractal: `SecondDegreeThirtySevenBlend`

![ckvjjj.png](presets/ckvjjj.png)

### [phgzbz.ron](presets/phgzbz.ron)

> Fractal: `Wmriho(a_re: -0.1, a_im: 0)`

![phgzbz.png](presets/phgzbz.png)

### [gqwzzr.ron](presets/gqwzzr.ron)

> Fractal: `ComplexLogisticMapLike`

https://github.com/user-attachments/assets/83793c10-4d2a-47f2-8e0b-7cee47c27e6b

### [dmgtyz.ron](presets/dmgtyz.ron)

> Fractal: `Sdrage`

![dmgtyz.png](presets/dmgtyz.png)

### [datgdv.ron](presets/datgdv.ron)

> Fractal: `Sdrge`

![datgdv.png](presets/datgdv.png)

# Ideas

- use wgpu to perform calculations ? see [this](https://github.com/gfx-rs/wgpu/blob/trunk/examples%2Fsrc%2Fhello_compute%2Fmod.rs) and especially [this](https://github.com/gfx-rs/wgpu/blob/trunk/examples%2Fsrc%2Frepeated_compute%2Fmod.rs)
- use opencl to perform calculations ? see [this](https://docs.rs/opencl3/latest/opencl3/)
- Make a new program using this one that is a purely gui program with progressive rendering
  - Progressive rendering ? Save a global `raw_image` and sample continuously from another thread to improve image quality
    - How to sample ? Use `Low` or `Medium` for first pass then do other passes with `High` (as the number of passes increases, the value of each pixel gets more and more accurate)
    - Careful: The average between new passes and the current values must be weighted: `(sampling_point_count_from_start * stored_value + sampling_point_count_for_current_pass * new_value) / (sampling_point_count_from_start + sampling_point_count_for_current_pass)`

# Notes

- To create a video from the frames:
  ```bash
  ffmpeg -framerate <fps> -pattern_type glob -i 'frames/*.png' -c:v libx264 -pix_fmt yuv420p video.mp4
  ```
