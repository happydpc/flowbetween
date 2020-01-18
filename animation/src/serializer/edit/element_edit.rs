use super::super::target::*;
use super::super::super::traits::*;

impl ElementEdit {
    ///
    /// Generates a serialized version of this edit on the specified data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, data: &mut Tgt) {
        use self::ElementEdit::*;

        match self {
            AddAttachment(elem)         => { data.write_chr('+'); elem.serialize(data); }
            RemoveAttachment(elem)      => { data.write_chr('-'); elem.serialize(data); }
            Order(ordering)             => { data.write_chr('O'); ordering.serialize(data); }
            Delete                      => { data.write_chr('X'); }
            DetachFromFrame             => { data.write_chr('D'); }

            SetControlPoints(points)    => { 
                data.write_chr('C'); 
                data.write_usize(points.len());

                let mut last_point = (0.0f32, 0.0f32);
                for (x, y) in points.iter() {
                    data.write_next_f64(last_point.0 as f64, *x as f64);
                    data.write_next_f64(last_point.1 as f64, *y as f64);

                    last_point = (*x, *y);
                }
            }

            SetPath(path_components)    => { 
                data.write_chr('P'); 
                data.write_usize(path_components.len()); 

                let mut last_point = PathPoint::new(0.0, 0.0);
                for component in path_components.iter() {
                    last_point = component.serialize_next(&last_point, data);
                }
            }
        }
    }
}