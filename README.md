# Fractal Renderer

This is a very simple program used to render fractals to images using a parameter json file.

It includes different fractal kinds among which the Mandelbrot set and a (potentially new) kind of fractal I came up with by using second- and third-degree recursive sequences instead of the classic first-degree recursive pattern used to draw the Mandelbrot set..

One of the available coloring modes is cumulative histogram coloring (which was used to render the [presets](#preset-renders)). In the future I would like to add more modes and improve the current ones.

It also features oversampling to reduce artifact pixels.

# How to use

Create a json file that must have the following structure:

```jsonc
{
  "img_width": ..., // make the image as big as you want (not too big tho)
  "img_height": ...,
  // zoom into the fractal by decreasing this
  "zoom": 1.,
  // change this...
  "center_x": 0.0,
  // ... and this to change the render position
  "center_y": 0.0,
  // change max iteration count
  "max_iter": 80000,
  // (optional) takes multiple samples per pixel to improve image quality
  "supersampling": 4,
  // this is the fractal kind (see presets)
  "fractal_kind": ...,
  // cumulative histogram recommended
  "coloring_mode": "CumulativeHistogram"
}
```

Next, you need to download the program from the [releases tab](https://github.com/valflrt/fractal_renderer/releases/latest).

Then, in order to render your fractal, run the following command:

```
fractal_renderer fractal.json fractal.png
```

alternatively, if you have rust installed and downloaded this repository:

```
cargo run -r -- fractal.json fractal.png
```


> [!NOTE]
>
> - You can change the file names
> - Supported image formats are png and jpg (extension used to guess image format)

# Preset renders

These are preset renders I find pretty, you can get their json parameters files by clicking on the title. There are some more in [`presets/`](./presets/).

### [ukhbrp.json](./presets/ukhbrp.json)

![ukhbrp.png](./presets/ukhbrp.png)

### [ebidxr.json](./presets/ebidxr.json)

![ebidxr.png](./presets/ebidxr.png)

### [mzfyje.json](./presets/mzfyje.json)

![mzfyje.png](./presets/mzfyje.png)

### [ztkhky.json](./presets/ztkhky.json)

![ztkhky.png](./presets/ztkhky.png)

### [idkzrg.json](./presets/idkzrg.json)

![idkzrg.png](./presets/idkzrg.png)

### [datgdv.json](./presets/datgdv.json)

![datgdv.png](./presets/datgdv.png)
