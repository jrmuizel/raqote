
pub struct Point
{
	pub x: f32,
	pub y: f32,
}
/*
#include "matrix.h"
#include "fixedpoint.h"

struct Color
{
	int r;
	int g;
	int b;
	int a;
};

// A class that we use for computation of intermediate
// color values. We use this to accumulate the results
// of 4x4 subpixels. For this to be exact we need
// to be able to store 16*255 or 4 extra bits per component.
// XXX: we could split out the types for the accumulating ones
// and the plain values so that they are not confused.
struct Intermediate
{
	// use a SWAR approach:
	//      aaaaaaaa rrrrrrrr gggggggg bbbbbbbb
	// ag = aaaaaaaaaaaaaaaaa ggggggggggggggggg
	// rb = rrrrrrrrrrrrrrrrr bbbbbbbbbbbbbbbbb
	//
	// This cuts the number of additions in half,
	// is more compact and easier to finalize,
	// into back into argb
	int ag;
	int rb;

	Intermediate() : ag(0), rb(0)
	{
	}

	static Intermediate expand(uint32_t color)
	{
		Intermediate i;
		i.ag = (color>>8) & 0xff00ff;
		i.rb = color & 0xff00ff;
		return i;
	}

	void accumulate(Intermediate i)
	{
		ag += i.ag;
		rb += i.rb;
	}

	// XXX: this needs to be fleshed out
	// how do we do 'over' with immediates'
	Intermediate
	over(Intermediate c)
	{
		if ((c.ag & 0xff0000) == 0xff0000) {
			this->ag = c.ag;
			this->rb = c.rb;
		} else {
			// a fast approximation of OVER
			// XXX: should use 256 instead of 0xff
			int alpha = 0xff - (c.ag >> 16);
			this->ag = (((this->ag * alpha) >> 8) & 0xff00ff) + c.ag;
			this->rb = (((this->rb * alpha) >> 8) & 0xff00ff) + c.rb;
		}
		return *this;
	}

	void
	assign(Color c)
	{
		this->ag = c.a << 16 | c.g;
		this->rb = c.r << 16 | c.b;
	}

	uint32_t finalize_unaccumulated() {
		return (ag << 8) | rb;
	}

	uint32_t finalize() {
		uint32_t result;
		result  = (ag << 4) & 0xff00ff00;
		result |= (rb >> 4) & 0x00ff00ff;
		return result;
	}
};


struct Span;

struct GradientStop
{
	float position;
	uint32_t color;
};

struct Gradient
{
	FixedMatrix matrix;
	// using a 257 entry lookup table
	// lets us use a 1.8 fixed point implementation
	uint32_t lookup[257];
};

struct Bitmap
{
	int width;
	int height;
	FixedMatrix matrix;
	uint32_t *data;
};


struct Shape
{
	Shape() {}
	int fill_style;
	bool opaque;
	// we can union the different fill style implementations here.
	// e.g. a pointer to an image fill or gradient fill
	union {
		Intermediate color;
		Gradient *gradient;
		Bitmap *bitmap;
	};
	int z;
	void (*fill)(Shape *s, uint32_t *buf, int x, int y, int w);
	Intermediate (*eval)(Shape *s, int x, int y);
#ifndef NDEBUG
	Shape *next;
#endif
	Span *span_begin;
};


#endif
*/
