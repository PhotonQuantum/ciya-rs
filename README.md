# ciya-rs

Ciyaify your image.

![from](examples/original.png)
![to](examples/result.png)

## Get Started

> Currently only Linux is supported.

### Linux

- Install OpenCV library. Make sure to install `-dev` packages if your distribution provides.

> For Archlinux users:
> 
> Install OpenCV 4.5.1, not the latest version which is unsupported.

- ``` make cli ```
- Built binaries are located in `dist` directory.

## Todo

- [ ] `detectors::StandardDetector`
- [ ] tg bot
- [ ] release (deal with onnxruntime)
- [ ] add a proper license