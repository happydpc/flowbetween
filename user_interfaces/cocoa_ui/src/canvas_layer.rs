use flo_canvas::*;

use super::canvas_state::*;
use super::core_graphics_ffi::*;

///
/// Processes canvas draw commands onto a core graphics context
/// 
/// This assumes that all the commands are intended for a specific layer: ie, layer switch commands
/// are ignored.
///
pub struct CanvasLayer {
    /// The location of the viewport origin for this canvas layer (the point that we should consider as 0,0)
    viewport_origin: (f64, f64),

    /// The width and height of the viewport for this layer
    viewport_size: (f64, f64),

    /// The width and height of the canvas for this layer (canvas is assumed to have an origin at 0,0)
    canvas_size: (f64, f64),

    /// Tracks the current state of the context
    state: CanvasState,

    /// The CGContext that drawing commands for this layer should be sent to
    context: CFRef<CGContextRef>
}

impl CanvasLayer {
    ///
    /// Creates a new canvas layer that will render to the specified context
    ///
    pub unsafe fn new(context: CFRef<CGContextRef>, viewport_origin: (f64, f64), viewport_size: (f64, f64), canvas_size: (f64, f64)) -> CanvasLayer {
        // Colours are in the SRGB colourspace
        let srgb        = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
        let mut state   = CanvasState::new(CFRef::from(srgb));

        state.activate_context(context.clone());

        let mut new_layer = CanvasLayer {
            viewport_origin:    viewport_origin,
            viewport_size:      viewport_size,
            canvas_size:        canvas_size,
            context:            context,
            state:              state
        };

        new_layer.state.set_transform(new_layer.get_identity_transform());

        new_layer
    }

    ///
    /// Computes the identity transform for this canvas
    ///
    fn get_identity_transform(&self) -> CGAffineTransform {
        unsafe {
            let (origin_x, origin_y)    = self.viewport_origin;
            let (width, height)         = self.canvas_size;
            let scale                   = (height as CGFloat)/2.0;

            let transform = CGAffineTransformIdentity;
            let transform = CGAffineTransformTranslate(transform, origin_x as CGFloat, origin_y as CGFloat);
            let transform = CGAffineTransformTranslate(transform, (width as CGFloat)/2.0, (height as CGFloat)/2.0);
            let transform = CGAffineTransformScale(transform, scale, -scale);

            transform
        }
    }

    ///
    /// Draws on this canvas
    ///
    pub fn draw(&mut self, draw: &Draw) {
        use self::Draw::*;

        unsafe {
            match draw {
                NewPath                                             => { CGContextBeginPath(*self.context); }
                Move(x, y)                                          => { CGContextMoveToPoint(*self.context, *x as CGFloat, *y as CGFloat); }
                Line(x, y)                                          => { CGContextAddLineToPoint(*self.context, *x as CGFloat, *y as CGFloat); }
                BezierCurve((ex, ey), (c1x, c1y), (c2x, c2y))       => { CGContextAddCurveToPoint(*self.context, *c1x as CGFloat, *c1y as CGFloat, *c2x as CGFloat, *c2y as CGFloat, *ex as CGFloat, *ey as CGFloat); }
                ClosePath                                           => { CGContextClosePath(*self.context); }
                Fill                                                => { CGContextFillPath(*self.context); }
                Stroke                                              => { CGContextStrokePath(*self.context); }
                LineWidth(width)                                    => { CGContextSetLineWidth(*self.context, *width as CGFloat); }
                LineWidthPixels(width_pixels)                       => { /* TODO */ }
                LineJoin(join)                                      => { /* TODO */ }
                LineCap(cap)                                        => { /* TODO */ }
                NewDashPattern                                      => { /* TODO */ }
                DashLength(len)                                     => { /* TODO */ }
                DashOffset(offset)                                  => { /* TODO */ }
                FillColor(col)                                      => { self.state.set_fill_color(col); }
                StrokeColor(col)                                    => { self.state.set_stroke_color(col); }
                BlendMode(blend)                                    => { /* TODO */ }
                IdentityTransform                                   => { self.state.set_transform(self.get_identity_transform()); }
                CanvasHeight(height)                                => { /* TODO */ }
                CenterRegion((minx, miny), (maxx, maxy))            => { /* TODO */ }
                MultiplyTransform(transform)                        => { /* TODO */ }
                Unclip                                              => { /* TODO */ }
                Clip                                                => { /* TODO */ }
                Store                                               => { /* TODO */ }
                Restore                                             => { /* TODO */ }
                FreeStoredBuffer                                    => { /* TODO */ }
                PushState                                           => { self.state.push_state(); }
                PopState                                            => { self.state.pop_state(); }
                ClearCanvas                                         => { /* TODO */ }
                Layer(layer_id)                                     => { /* TODO */ }
                LayerBlend(layer_id, blend)                         => { /* TODO */ }
                ClearLayer                                          => { /* TODO */ }
            }
        }
    }
}
