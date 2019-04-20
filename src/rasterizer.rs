/* Copyright 2013 Jeff Muizelaar
 *
 * Use of this source code is governed by a MIT-style license that can be
 * found in the LICENSE file.
 *
 * Portions Copyright 2006 The Android Open Source Project
 *
 * Use of that source code is governed by a BSD-style license that can be
 * found in the LICENSE.skia file.
 */

// One reason to have separate Edge/ActiveEdge is reduce the
// memory usage of inactive edges. On the other hand
// managing the lifetime of ActiveEdges is a lot
// trickier than Edges. Edges can stay alive for the entire
// rasterization. ActiveEdges will come and go in a much
// less predictable order. On the other hand having the
// ActiveEdges close together in memory would help
// avoid cache misses. If we did switch to having separate
// active edges it might be wise to store the active edges
// in an array instead of as a linked list. This will work
// well for the bubble sorting, but will cause more problems
// for insertion.

use typed_arena::Arena;

use crate::types::Point;

use std::ptr::NonNull;

struct Edge {
	//XXX: it is probably worth renaming this to top and bottom
	x1: i32,
	y1: i32,
	x2: i32,
	y2: i32,
	control_x: i32,
	control_y: i32,
}


// it is possible to fit this into 64 bytes on x86-64
// with the following layout:
//
// 4 x2,y2
// 8 shape
// 8 next
// 6*4 slope_x,fullx,next_x,next_y, old_x,old_y
// 4*4 dx,ddx,dy,ddy
// 2 cury
// 1 count
// 1 shift
//
// some example counts 5704 curves, 1720 lines 7422 edges
pub struct ActiveEdge {
	x2: i32,
	y2: i32,
	next: Option<NonNull<ActiveEdge>>,
	slope_x: i32,
	fullx: i32,
	next_x: i32,
	next_y: i32,

	dx: i32,
	ddx: i32,
	dy: i32,
	ddy: i32,

	old_x: i32,
	old_y: i32,

	shift: i32,
	// we need to use count so that we make sure that we always line the last point up
	// exactly. i.e. we don't have a great way to know when we're at the end implicitly.
	count: i32,
	winding: i8,
}

impl ActiveEdge {
	fn new() -> ActiveEdge {
		ActiveEdge {x2: 0, y2: 0, next: None, slope_x: 0, fullx: 0, next_x: 0, next_y: 0,
		dx: 0, ddx: 0, dy:0, ddy: 0, old_x: 0, old_y: 0, shift: 0, count: 0, winding: 0}
	}

	// we want this to inline into step_edges() to
	// avoid the call overhead
	fn step(&mut self, cury: i32) {
		// if we have a shift that means we have a curve
		if self.shift != 0 {
			//printf("inner cur %d,%d next %d %d %f\n", curx, cury, next_x>>16, next_y>>16, fnext_y);
			if cury >= (self.next_y>>16) {
				self.old_y = self.next_y;
				self.old_x = self.next_x;
				self.fullx = self.next_x;
				// increment until we have a next_y that's greater
				while (self.count > 0 && (cury >= (self.next_y>>16))) {
					self.next_x += self.dx >> self.shift;
					self.dx += self.ddx;
					self.next_y += self.dy >> self.shift;
					self.dy += self.ddy;
					self.count -= 1;
				}
				if self.count == 0 {
					// for the last line sgement we can
					// just set next_y,x to the end point
					self.next_y = self.y2<<16;
					self.next_x = self.x2<<16;
				}
				// update slope if we're going to be using it
				// we want to avoid dividing by 0 which can happen if we exited the loop above early
				if (cury+1) < self.y2 {
					// the maximum our x value can be is 4095 (which is 12 bits).
					// 12 + 3 + 16 = 31 which gives us an extra bit of room
					// to handle overflow.
					self.slope_x = ((self.next_x - self.old_x)<<3)/((self.next_y - self.old_y)>>13);
				}
			}
			self.fullx += self.slope_x;
		} else {
			// XXX: look into bresenham to control error here
			self.fullx += self.slope_x;
		}
		//cury += 1;
	}
}

pub struct Rasterizer<'a>
{
	/*
	Rasterizer(int width, int height);
	~Rasterizer() { delete[] edge_starts; };
*/

	edge_starts: Vec<Option<NonNull<ActiveEdge>>>,
	width: i32,
	height: i32,
	cur_y: i32,
	pub buf: Vec<u32>,
	active_edges: Option<NonNull<ActiveEdge>>,

	edge_arena: &'a mut Arena<ActiveEdge>,
}


impl<'a> Rasterizer<'a> {
	pub fn new(edge_arena: &'a mut Arena<ActiveEdge>, width: i32, height: i32) -> Rasterizer<'a> {

		let mut edge_starts = Vec::new();
		for _ in 0..(height * 4) {
			edge_starts.push(None);
		}
		Rasterizer {
			width: width * 4,
			height: height * 4,
			cur_y: 0,
			buf: vec![0; (width * height) as usize],
			edge_starts,
			edge_arena,
			active_edges: None
		}
	}
}

	/*

void
Rasterizer::reset() {
	cur_y = 0;
	active_edges = nullptr;
	shapes = nullptr;
	for (int i = 0; i < this->height; i++) {
		edge_starts[i] = NULL;
	}
}*/

fn Sk2Abs32(mut value: i32) -> i32 {
    if (value < 0) {
		value = -value;
	}
    return value;
}

fn Sk2CLZ(x: u32) -> u32 {
    if (x == 0) {
        return 32;
    }

	let mut zeros = ((x >> 16) - 1) >> 31 << 4;
    let mut x = x << zeros;

    let mut nonzero = ((x >> 24) - 1) >> 31 << 3;
    zeros += nonzero;
    x = x << nonzero;

    nonzero = ((x >> 28) - 1) >> 31 << 2;
    zeros += nonzero;
    x = x << nonzero;

    nonzero = ((x >> 30) - 1) >> 31 << 1;
    zeros += nonzero;
    x = x << nonzero;

    zeros += (!x) >> 31;

    return zeros;
}

// See also: http://www.flipcode.com/archives/Fast_Approximate_Distance_Functions.shtml
fn cheap_distance(mut dx: i32, mut dy: i32) -> i32
{
    dx = Sk2Abs32(dx);
    dy = Sk2Abs32(dy);
    // return max + min/2
    if (dx > dy) {
		dx += dy >> 1;
	} else {
		dx = dy + (dx >> 1);
	}
    return dx;
}

fn diff_to_shift(dx: i32, dy: i32) -> i32
{
    //printf("diff_to_shift: %d %d\n", dx, dy);
    // cheap calc of distance from center of p0-p2 to the center of the curve
    let mut dist = cheap_distance(dx, dy);

    //printf("dist: %d\n", dist);
    // shift down dist (it is currently in dot6)
    // down by 5 should give us 1/2 pixel accuracy (assuming our dist is accurate...)
    // this is chosen by heuristic: make it as big as possible (to minimize segments)
    // ... but small enough so that our curves still look smooth
    //printf("%d dist\n", dist);
    dist = (dist + (1 << 4)) >> 5;

    // each subdivision (shift value) cuts this dist (error) by 1/4
    return ((32 - Sk2CLZ(dist as u32) as i32))>> 1;
}

// this metric is taken from skia
fn compute_curve_steps(e: &Edge) -> i32
{
	let dx = ((e.control_x << 1) - e.x1 - e.x2);
	let dy = ((e.control_y << 1) - e.y1 - e.y2);
	let shift = diff_to_shift(dx<<4, dy<<4);
	assert!(shift >= 0);
	return shift;
}

const SAMPLE_SIZE: f32 = 4.;
const SAMPLE_SHIFT: i32 = 2;

const SHIFT: i32 = 2;
const SCALE: i32 = (1 << SHIFT);
const MASK: i32 =   (SCALE - 1);
const SUPER_Mask: i32 =  ((1 << SHIFT) - 1);
// An example number of edges is 7422 but
// can go as high as edge count: 374640
// with curve count: 67680
impl<'a> Rasterizer<'a> {
	pub fn add_edge(&mut self, mut start: Point, mut end: Point, curve: bool, control: Point)
	{
		//static int count;
		//printf("edge count: %d\n",++count);
		// order the points from top to bottom
		if (end.y < start.y) {
			std::mem::swap(&mut start, &mut end);
		}

		// how do we deal with edges to the right and left of the canvas?
		let e = self.edge_arena.alloc(ActiveEdge::new());
		let mut edge = Edge {
			x1: (start.x * SAMPLE_SIZE) as i32,
			y1: (start.y * SAMPLE_SIZE) as i32,
			control_x: (control.x * SAMPLE_SIZE) as i32,
			control_y: (control.y * SAMPLE_SIZE) as i32,
			x2: (end.x * SAMPLE_SIZE) as i32,
			y2: (end.y * SAMPLE_SIZE) as i32,
		};
		e.x2 = edge.x2;
		e.y2 = edge.y2;
		/*
        if (curve)
        printf("%d %d, %d %d, %d %d\n",
        e.edge.x1,
        e.edge.y1,
        e.edge.control_x,
        e.edge.control_y,
        e.edge.x2,
        e.edge.y2);
        */
		e.next = None;
		//e.curx = e.edge.x1;
		let mut cury = edge.y1;
		e.fullx = edge.x1 << 16;

		// if the edge is completely above or completely below we can drop it
		if (edge.y2 < 0 || edge.y1 > self.height) {
			return;
		}

		// drop horizontal edges
		if (cury >= e.y2) {
			return;
		}

		if (curve) {
			// Based on Skia
			// we'll iterate t from 0..1 (0-256)
			// range of A is 4 times coordinate-range
			// we can get more accuracy here by using the input points instead of the rounded versions
			let mut A = (edge.x1 - edge.control_x - edge.control_x + edge.x2) << 15;
			let mut B = (edge.control_x - edge.x1);
			let mut C = edge.x1;
			let shift = compute_curve_steps(&edge);
			e.shift = shift;
			e.count = 1 << shift;
			e.dx = 2 * A * (1 << (16 - shift)) + B * 65536;
			e.dx = 2 * (A >> shift) + 2 * B * 65536;
			e.ddx = 2 * (A >> (shift - 1));

			A = (edge.y1 - edge.control_y - edge.control_y + edge.y2) << 15;
			B = (edge.control_y - edge.y1);
			C = edge.y1;
			e.dy = 2 * A * (1 << (16 - shift)) + (B) * 65536;
			e.ddy = 2 * A * (1 << (16 - shift));
			e.dy = 2 * (A >> shift) + 2 * B * 65536;
			e.ddy = 2 * (A >> (shift - 1));

			// compute the first next_x,y
			e.count -= 1;
			e.next_x = (e.fullx) + (e.dx >> e.shift);
			e.next_y = (cury * 65536) + (e.dy >> e.shift);
			e.dx += e.ddx;
			e.dy += e.ddy;

			// skia does this part in UpdateQuad. unfortunately we duplicate it
			while (e.count > 0 && cury >= (e.next_y >> 16)) {
				e.next_x += e.dx >> shift;
				e.dx += e.ddx;
				e.next_y += e.dy >> shift;
				e.dy += e.ddy;
				e.count -= 1;
			}
			if (e.count == 0) {
				e.next_y = edge.y2 << 16;
				e.next_x = edge.x2 << 16;
			}
			e.slope_x = ((e.next_x - (e.fullx)) << 2) / ((e.next_y - (cury << 16)) >> 14);
		} else {
			e.shift = 0;
			e.slope_x = ((edge.x2 - edge.x1) * (1 << 16)) / (edge.y2 - edge.y1);
		}

		if (cury < 0) {
			// XXX: we could compute an intersection with the top and bottom so we don't need to step them into view
			// for curves we can just step them into place.
			while (cury < 0) {
				e.step(cury);
				cury += 1;
			}

			// cury was adjusted so check again for horizontal edges
			if (cury >= e.y2) {
				return;
			}
		}

		// add to the begining of the edge start list
		// if edges are added from left to right
		// the'll be in this list from right to left
		// this works out later during insertion
		e.next = self.edge_starts[cury as usize];
		self.edge_starts[cury as usize] = Some(unsafe { NonNull::new_unchecked(e as *mut _) });
	}

	fn step_edges(&mut self)
	{
		let mut prev_ptr = &mut self.active_edges as *mut _;
		let mut edge = self.active_edges;
		let cury = self.cur_y; // avoid any aliasing problems
		while let Some(mut e_ptr) = edge {
			let e = unsafe { e_ptr.as_mut() };
			e.step(cury);
			// avoid aliasing between edge->next and prev_ptr so that we can reuse next
			let next = e.next;
			// remove any finished edges
			if ((cury+1) >= e.y2) {
				// remove from active list
				unsafe { *prev_ptr = next } ;
			} else {
				prev_ptr = &mut e.next;
			}
			edge = next;
		}
	}
	/*
	int comparisons;
	static inline void dump_edges(ActiveEdge *e)
	{
	while (e) {
	printf("%d ", e.fullx);
	e = e.next;
	}
	printf("\n");
	}
*/
	// Insertion sort the new edges into the active list
// The new edges could be showing up at any x coordinate
// but existing active edges will be sorted.
//
// Merge in the new edges. Since both lists are sorted we can do
// this in a single pass.
// Note: we could do just O(1) append the list of new active edges
// to the existing active edge list, but then we'd have to sort
// the entire resulting list
	fn insert_starting_edges(&mut self)
	{
		let mut new_edges: Option<NonNull<ActiveEdge>> = None;
		let mut edge = self.edge_starts[self.cur_y as usize];
		// insertion sort all of the new edges
		while let Some(mut e_ptr) = edge {
			let e = unsafe { e_ptr.as_mut() };
			let mut prev_ptr = &mut new_edges as *mut _;
			let mut new = new_edges;
			while let Some(mut new_ptr) = new {
				let a = unsafe { new_ptr.as_mut() };
				if e.fullx > a.fullx { break; }
				// comparisons++;
				prev_ptr = &mut a.next;
				new = a.next;
			}
			edge = e.next;
			e.next = new;
			unsafe { *prev_ptr  = Some(e_ptr) };
		}


		// merge the sorted new_edges into active_edges
		let mut prev_ptr = &mut self.active_edges as *mut _;
		let mut active = self.active_edges;
		let mut edge = new_edges;
		while let Some(mut e_ptr) = edge {
			let e = unsafe { e_ptr.as_mut() };
			while let Some(mut a_ptr) = active {
				let a = unsafe { a_ptr.as_mut() };
				if e.fullx > a.fullx { break; }

				// comparisons++;
				prev_ptr = &mut a.next;
				active = a.next;
			}
			edge = e.next;
			e.next = active;
			let next_prev_ptr = &mut e.next as *mut _;
			unsafe { *prev_ptr = Some(e_ptr) };
			prev_ptr = next_prev_ptr;
		}
	}
}
		fn coverage_to_alpha(mut aa: i32) -> u32
		{
			aa <<= 8 - 2 * SHIFT;
			aa -= aa >> (8 - SHIFT - 1);
			return aa as u32;
		}

impl<'a> Rasterizer<'a> {
		// Skia does stepping and scanning of edges in a single
// pass over the edge list.
		fn scan_edges(&mut self)
		{
			let mut edge = self.active_edges;
			let mut winding = 0;

			// handle edges that begin to the left of the bitmap
			while let Some(mut e_ptr) = edge {
				let e = unsafe { e_ptr.as_mut() };
				if (e.fullx < 0) { break }
				winding += 1;
				edge = e.next;
			}

			let mut prevx = 0;
			while let Some(mut e_ptr) = edge {
				let e = unsafe { e_ptr.as_mut() };
				if ((e.fullx >> 16) >= self.width) {
					break;
				}

				if (winding & 1 != 0) {
					self.blit_span((prevx + (1 << 15)) >> 16, (e.fullx + (1 << 15)) >> 16);
				}
				winding += 1;
				prevx = e.fullx;
				edge = e.next;
			}

			// we don't need to worry about any edges beyond width
		}

		fn blit_span(&mut self, x1: i32, x2: i32)
		{
			//printf("%d %d\n", x1, x2);
			let max: u32 = ((1 << (8 - SHIFT)) - (((self.cur_y & MASK) + 1) >> SHIFT)) as u32;
			let mut b: *mut u32 = &mut self.buf[(self.cur_y / 4 * self.width / 4 + (x1 >> SHIFT)) as usize];

			let mut fb = x1 & SUPER_Mask;
			let fe = x2 & SUPER_Mask;
			let mut n = (x2 >> SHIFT) - (x1 >> SHIFT) - 1;

			// invert the alpha on the left side
			if (n < 0) {
				unsafe { *b += coverage_to_alpha(fe - fb) * 0x1010101 };
			} else {
				fb = (1 << SHIFT) - fb;
				unsafe { *b += coverage_to_alpha(fb) * 0x1010101 };
				unsafe {  b = b.offset(1); };
				while (n != 0) {
					unsafe { *b += max * 0x1010101 };
					unsafe { b = b.offset(1) };

					n -= 1;
				}
				unsafe { *b  += coverage_to_alpha(fe) * 0x1010101 };
			}
		}

// You may have heard that one should never use a bubble sort.
// However in our situation a bubble sort is actually a good choice.
// The input list will be mostly sorted except for a couple of lines
// that have need to be swapped. Further it is common that our edges are
// already sorted and bubble sort lets us avoid doing any memory writes.

		// Some statistics from using a bubble sort on an
// example scene. You can see that bubble sort does
// noticably better than O (n lg n).
// summary(edges*bubble_sort_iterations)
//   Min. 1st Qu.  Median    Mean 3rd Qu.    Max.
//    0.0     9.0    69.0   131.5   206.0  1278.0
// summary(edges*log2(edges))
//   Min. 1st Qu.  Median    Mean 3rd Qu.    Max.    NA's
//   0.00   28.53  347.10  427.60  787.20 1286.00    2.00
		fn sort_edges(&mut self)
		{
			if (self.active_edges.is_none()) {
				return;
			}
			let mut swapped;
			loop {
				swapped = false;
				let mut edge = self.active_edges.unwrap();
				let mut next_edge = unsafe { edge.as_mut() }.next;
				let mut prev = &mut self.active_edges as *mut _;
				while let Some(mut next_ptr) = next_edge {
					let next = unsafe { next_ptr.as_mut()};
					if (unsafe { edge.as_mut() }.fullx > next.fullx) {
						// swap edge and next
						unsafe { edge.as_mut() }.next = next.next;
						next.next = Some(edge);
						unsafe { (*prev) = Some(next_ptr) };
						swapped = true;
					}
					prev = (&mut unsafe { edge.as_mut() }.next) as *mut _;
					edge = next_ptr;
					next_edge = unsafe { edge.as_mut() }.next;
				}
				if swapped {
					break
				}
			}
		}

		pub fn rasterize(&mut self) {
			let mut cur_y = 0;
			while cur_y < self.height {
				// we do 4x4 super-sampling so we need
				// to scan 4 times before painting a line of pixels
				for _ in 0..4 {
					// insert the new edges into the sorted list
					self.insert_starting_edges();
					// scan over the edge list producing a list of spans
					self.scan_edges();
					// step all of the edges to the next scanline
					// dropping the ones that end
					self.step_edges();
					// sort the remaning edges
					self.sort_edges();
					cur_y += 1;
				}
			}
			// edge_arena.reset();
			// printf("comparisons: %d\n", comparisons);
		}
	}
