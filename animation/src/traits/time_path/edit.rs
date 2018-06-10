use super::convert::*;
use super::time_curve::*;
use super::time_point::*;

use std::time::Duration;

/// Minimum time between points where we allow curves to be divided (we'll edit existing points rather than allow sections
/// that are shorter than this)
const MIN_TIME_MILLISECONDS: f32 = 5.0;

//
// Supplies editing functions for a time curve
//
impl TimeCurve {
    ///
    /// Generates a time curve with the point at a particular time moved to a new location
    /// 
    /// If the duration is before the start or after the end of the curve, this will add a
    /// new point instead. If it's close to an existing point, that point will just be moved.
    /// Otherwise, if the point is in the middle of a curve, the curve will be subdivided.
    /// 
    pub fn set_point_at_time(&self, when: Duration, new_location: (f32, f32)) -> TimeCurve {
        let when_millis = to_millis(when) as f32;
        let (x, y)      = new_location;

        if self.points.len() < 2 {

            // If there's no points in the curve, then just generate a new curve
            TimeCurve::new(TimePoint::new(x, y, when), TimePoint::new(x, y, when))

        } else if when_millis <= self.points[0].point.milliseconds() - MIN_TIME_MILLISECONDS {

            // Time is before the current start of the motion.
            let mut new_points = self.points.clone();

            // Add a new point at the start
            let copy_point = new_points[0].clone();
            new_points.insert(0, copy_point);

            // Move the initial point to its new location
            new_points[0].move_to(x, y, when_millis);

            // Return the new curve
            TimeCurve { points: new_points }

        } else if when_millis >= self.points[self.points.len()-1].point.milliseconds() + MIN_TIME_MILLISECONDS {

            // Time is after the current start of the motion
            let mut new_points  = self.points.clone();
            let final_point     = new_points.len()-1;

            // Add a new point at the end
            let copy_point  = new_points[final_point].clone();
            let final_point = final_point + 1;  
            new_points.push(copy_point);

            // Move the final point to its new location
            new_points[final_point].move_to(x, y, when_millis);

            // Return the new curve
            TimeCurve { points: new_points }

        } else {

            // Point is within the existing curve
            // TODO!
            unimplemented!()
            
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn setting_point_before_start_creates_new_point() {
        let curve       = TimeCurve::new(TimePoint(40.0, 40.0, 40.0), TimePoint(50.0, 50.0, 50.0));
        let moved_curve = curve.set_point_at_time(Duration::from_millis(5), (10.0, 10.0));

        assert!(moved_curve.points.len() == 3);
        assert!(moved_curve.points[0].point == TimePoint(10.0, 10.0, 5.0));
        assert!(moved_curve.points[0].future == moved_curve.points[1].future - TimePoint(30.0, 30.0, 35.0));
        assert!(moved_curve.points[1].point == TimePoint(40.0, 40.0, 40.0));
        assert!(moved_curve.points[2].point == TimePoint(50.0, 50.0, 50.0));
    }

    #[test]
    fn setting_point_after_end_creates_new_point() {
        let curve       = TimeCurve::new(TimePoint(40.0, 40.0, 40.0), TimePoint(50.0, 50.0, 50.0));
        let moved_curve = curve.set_point_at_time(Duration::from_millis(60), (10.0, 10.0));

        assert!(moved_curve.points.len() == 3);
        assert!(moved_curve.points[2].point == TimePoint(10.0, 10.0, 60.0));
        assert!(moved_curve.points[2].future == moved_curve.points[1].future + TimePoint(-40.0, -40.0, 10.0));
        assert!(moved_curve.points[0].point == TimePoint(40.0, 40.0, 40.0));
        assert!(moved_curve.points[1].point == TimePoint(50.0, 50.0, 50.0));
    }

    #[test]
    fn moving_start_point_only_changes_position() {
        let curve       = TimeCurve::new(TimePoint(40.0, 40.0, 40.0), TimePoint(50.0, 50.0, 50.0));
        let moved_curve = curve.set_point_at_time(Duration::from_millis(40), (10.0, 10.0));

        assert!(moved_curve.points.len() == 2);
        assert!(moved_curve.points[0].point == TimePoint(10.0, 10.0, 40.0));
        assert!(moved_curve.points[1].point == TimePoint(50.0, 50.0, 50.0));
    }

    #[test]
    fn moving_instant_start_point_changes_both_start_and_end_point() {
        let curve       = TimeCurve::new(TimePoint(40.0, 40.0, 40.0), TimePoint(40.0, 40.0, 40.0));
        let moved_curve = curve.set_point_at_time(Duration::from_millis(40), (10.0, 10.0));

        assert!(moved_curve.points.len() == 2);
        assert!(moved_curve.points[0].point == TimePoint(10.0, 10.0, 40.0));
        assert!(moved_curve.points[1].point == TimePoint(10.0, 10.0, 40.0));
    }
}