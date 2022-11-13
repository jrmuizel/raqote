The overall design of raqote is very conservative. It contains nothing novel and is mostly
a repackaging of classic techniques used elsewhere. It borrows heavily from Skia.

The rasterizer is a relatively straightforward 4x4 supersampling scanline
rasterizer. It includes some tricks taken from Skia:
1. Monotonic quadratic curve edges can be used directly instead of having to flatten them.
2. Partial results are accumulated directly into a scanline with some approximations to avoid overflow at 255
3. An alpha mask for the entire shape is produced and then composited. However the intention is to switch
   to Skia like run length representation and only shade the parts of the mask where there is coverage.

The stroker is a classic postscript style stroker that works on flattened paths. It does not try
avoid overlap and uses distinct subpaths for each line segment, join and cap.

The dasher just chops a flattened paths into subpaths for each dash. It does
not try to handle zero-length segments.

The compositor is designed around shading a scanline at a time. Gradients are sampled from a lookup
table and bilinear filtering is a lower precision approximation that's cheaper to compute on the cpu.

Global alpha is implemented by having shaders handle it manually.

Prior Art:
- Skia
- Cairo
- Fitz
- Blend2D
- Qt
- Libart
- Antigrain
- Java2D (Pisces/Marlin https://github.com/bourgesl/marlin-renderer)
