use super::path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::geo::*;
use super::super::super::line::*;
use super::super::super::coordinate::*;

use std::fmt;
use std::mem;
use std::ops::Range;
use std::cmp::Ordering;

const CLOSE_DISTANCE: f64 = 0.01;

///
/// Kind of a graph path edge
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphPathEdgeKind {
    /// An edge that hasn't been categorised yet
    Uncategorised,

    /// An exterior edge
    /// 
    /// These edges represent a transition between the inside and the outside of the path
    Exterior, 

    /// An interior edge
    /// 
    /// These edges are on the inside of the path
    Interior
}

///
/// Reference to a graph edge
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphEdgeRef {
    /// The index of the point this edge starts from
    start_idx: usize,

    /// The index of the edge within the point
    edge_idx: usize,

    /// True if this reference is for the reverse of this edge
    reverse: bool
}

///
/// Enum representing an edge in a graph path
/// 
#[derive(Clone, Debug)]
struct GraphPathEdge<Point, Label> {
    /// The label attached to this edge
    label: Label,

    /// The kind of this edge
    kind: GraphPathEdgeKind,

    /// Position of the first control point
    cp1: Point,

    /// Position of the second control point
    cp2: Point,

    /// The index of the target point
    end_idx: usize
}

///
/// Struct representing a point in a graph path
///
#[derive(Clone, Debug)]
struct GraphPathPoint<Point, Label> {
    /// The position of this point
    position: Point,

    /// The edges attached to this point
    forward_edges: Vec<GraphPathEdge<Point, Label>>,

    /// The points with edges connecting to this point
    connected_from: Vec<usize>
}

impl<Point, Label> GraphPathPoint<Point, Label> {
    ///
    /// Creates a new graph path point
    ///
    fn new(position: Point, forward_edges: Vec<GraphPathEdge<Point, Label>>, connected_from: Vec<usize>) -> GraphPathPoint<Point, Label> {
        GraphPathPoint { position, forward_edges, connected_from }
    }
}

impl<Point: Coordinate, Label> GraphPathEdge<Point, Label> {
    ///
    /// Creates a new graph path edge
    /// 
    #[inline]
    fn new(kind: GraphPathEdgeKind, (cp1, cp2): (Point, Point), end_idx: usize, label: Label) -> GraphPathEdge<Point, Label> {
        GraphPathEdge {
            label, kind, cp1, cp2, end_idx
        }
    }

    ///
    /// Updates the control points of this edge
    /// 
    #[inline]
    fn set_control_points(&mut self, (cp1, cp2): (Point, Point), end_idx: usize) {
        self.cp1 = cp1;
        self.cp2 = cp2;
        self.end_idx = end_idx;
    }
}

///
/// A graph path is a path where each point can have more than one connected edge. Edges are categorized
/// into interior and exterior edges depending on if they are on the outside or the inside of the combined
/// shape.
/// 
#[derive(Clone, Debug)]
pub struct GraphPath<Point, Label> {
    /// The points in this graph and their edges. Each 'point' here consists of two control points and an end point
    points: Vec<GraphPathPoint<Point, Label>>
}

impl<Point: Coordinate, Label> Geo for GraphPath<Point, Label> {
    type Point = Point;
}

impl<Point: Coordinate+Coordinate2D, Label: Copy> GraphPath<Point, Label> {
    ///
    /// Creates a new graph path with no points
    ///
    pub fn new() -> GraphPath<Point, Label> {
        GraphPath {
            points: vec![]
        }
    }

    ///
    /// Creates a graph path from a bezier path
    /// 
    pub fn from_path<P: BezierPath<Point=Point>>(path: &P, label: Label) -> GraphPath<Point, Label> {
        // All edges are exterior for a single path
        let mut points = vec![];

        // Push the start point (with an open path)
        let start_point = path.start_point();
        points.push(GraphPathPoint::new(start_point, vec![], vec![]));

        // We'll add edges to the previous point
        let mut last_point = 0;
        let mut next_point = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Push the points
            points.push(GraphPathPoint::new(end_point, vec![], vec![]));

            // Add an edge from the last point to the next point
            points[last_point].forward_edges.push(GraphPathEdge::new(GraphPathEdgeKind::Uncategorised, (cp1, cp2), next_point, label));

            // Update the last/next pooints
            last_point += 1;
            next_point += 1;
        }

        // Close the path
        if last_point > 0 {
            // Graph actually has some edges
            if start_point.distance_to(&points[last_point].position) < CLOSE_DISTANCE {
                // Remove the last point (we're replacing it with an edge back to the start)
                points.pop();
                last_point -= 1;

                // Change the edge to point back to the start
                points[last_point].forward_edges[0].end_idx = 0;
            } else {
                // Need to draw a line to the last point
                let close_vector    = points[last_point].position - start_point;
                let cp1             = close_vector * 0.33 + start_point;
                let cp2             = close_vector * 0.66 + start_point;

                points[last_point].forward_edges.push(GraphPathEdge::new(GraphPathEdgeKind::Uncategorised, (cp1, cp2), 0, label));
            }
        } else {
            // Just a start point and no edges: remove the start point as it doesn't really make sense
            points.pop();
        }

        // Create the graph path from the points
        let mut path = GraphPath {
            points: points
        };
        path.recalculate_reverse_connections();
        path
    }

    ///
    /// Creates a new graph path by merging (not colliding) a set of paths with their labels
    ///
    pub fn from_merged_paths<'a, P: 'a+BezierPath<Point=Point>, PathIter: IntoIterator<Item=(&'a P, Label)>>(paths: PathIter) -> GraphPath<Point, Label> {
        // Create an empty path
        let mut merged_path = GraphPath::new();

        // Merge each path in turn
        for (path, label) in paths {
            let path    = GraphPath::from_path(path, label);
            merged_path = merged_path.merge(path);
        }

        merged_path
    }

    ///
    /// Recomputes the list of items that have connections to each point
    ///
    fn recalculate_reverse_connections(&mut self) {
        // Reset the list of connections to be empty
        for point_idx in 0..(self.points.len()) {
            self.points[point_idx].connected_from = vec![];
        }

        // Add a reverse connection for every edge
        for point_idx in 0..(self.points.len()) {
            for edge_idx in 0..(self.points[point_idx].forward_edges.len()) {
                let end_idx = self.points[point_idx].forward_edges[edge_idx].end_idx;
                self.points[end_idx].connected_from.push(point_idx);
            }
        }
    }

    ///
    /// Returns the number of points in this graph. Points are numbered from 0 to this value.
    /// 
    #[inline]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    ///
    /// Returns an iterator of all edges in this graph
    ///
    #[inline]
    pub fn all_edges<'a>(&'a self) -> impl 'a+Iterator<Item=GraphEdge<'a, Point, Label>> {
        (0..(self.points.len()))
            .into_iter()
            .flat_map(move |point_num| self.edges_for_point(point_num))
    }

    ///
    /// Returns an iterator of the edges that leave a particular point
    /// 
    /// Edges are directional: this will provide the edges that leave the supplied point
    ///
    #[inline]
    pub fn edges_for_point<'a>(&'a self, point_num: usize) -> impl 'a+Iterator<Item=GraphEdge<'a, Point, Label>> {
        (0..(self.points[point_num].forward_edges.len()))
            .into_iter()
            .map(move |edge_idx| GraphEdge::new(self, GraphEdgeRef { start_idx: point_num, edge_idx: edge_idx, reverse: false }))
    }

    ///
    /// Returns an iterator of the edges that arrive at a particular point
    /// 
    /// Edges are directional: this will provide the edges that connect to the supplied point
    ///
    pub fn reverse_edges_for_point<'a>(&'a self, point_num: usize) -> impl 'a+Iterator<Item=GraphEdge<'a, Point, Label>> {
        // Fetch the points that connect to this point
        self.points[point_num].connected_from
            .iter()
            .flat_map(move |connected_from| {
                let connected_from = *connected_from;

                // Any edge that connects to the current point, in reverse
                (0..(self.points[connected_from].forward_edges.len()))
                    .into_iter()
                    .filter_map(move |edge_idx| {
                        if self.points[connected_from].forward_edges[edge_idx].end_idx == point_num {
                            Some(GraphEdgeRef { start_idx: connected_from, edge_idx: edge_idx, reverse: true })
                        } else {
                            None
                        }
                    })
            })
            .map(move |edge_ref| GraphEdge::new(self, edge_ref))
    }

    ///
    /// Merges in another path
    /// 
    /// This adds the edges in the new path to this path without considering if they are internal or external 
    ///
    pub fn merge(self, merge_path: GraphPath<Point, Label>) -> GraphPath<Point, Label> {
        // Copy the points from this graph
        let mut new_points  = self.points;

        // Add in points from the merge path
        let offset          = new_points.len();
        new_points.extend(merge_path.points.into_iter()
            .map(|mut point| {
                // Update the offsets in the edges
                for mut edge in &mut point.forward_edges {
                    edge.end_idx += offset;
                }

                // Generate the new edge
                point
            }));

        // Combined path
        GraphPath {
            points: new_points
        }
    }

    /// 
    /// True if the t value is effectively at the start of the curve
    /// 
    #[inline]
    fn t_is_zero(t: f64) -> bool { t < 0.01 }

    ///
    /// True if the t value is effective at the end of the curve
    /// 
    #[inline]
    fn t_is_one(t: f64) -> bool { t > 0.99 }

    ///
    /// Joins two edges at an intersection, returning the index of the intersection point
    /// 
    /// For t=0 or 1 the intersection point may be one of the ends of the edges, otherwise
    /// this will divide the existing edges so that they both meet at the specified mid-point.
    /// 
    /// Note that the case where t=1 is the same as the case where t=0 on a following edge.
    /// The split algorithm is simpler if only the t=0 case is considered.
    /// 
    #[inline]
    fn join_edges_at_intersection(&mut self, edge1: (usize, usize), edge2: (usize, usize), t1: f64, t2: f64) -> Option<usize> {
        // Do nothing if the edges are the same (they're effectively already joined)
        if edge1 == edge2 { return None; }

        // Get the edge indexes
        let (edge1_idx, edge1_edge_idx) = edge1;
        let (edge2_idx, edge2_edge_idx) = edge2;

        // Create representations of the two edges
        let edge1 = Curve::from_curve(&GraphEdge::new(self, GraphEdgeRef { start_idx: edge1_idx, edge_idx: edge1_edge_idx, reverse: false }));
        let edge2 = Curve::from_curve(&GraphEdge::new(self, GraphEdgeRef { start_idx: edge2_idx, edge_idx: edge2_edge_idx, reverse: false }));

        // Create or choose a point to collide at
        // (If t1 or t2 is 0 or 1 we collide on the edge1 or edge2 points, otherwise we create a new point to collide at)
        let collision_point = if Self::t_is_zero(t1) {
            edge1_idx
        } else if Self::t_is_one(t1) {
            self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx
        } else if Self::t_is_zero(t2) {
            edge2_idx
        } else if Self::t_is_one(t2) {
            self.points[edge2_idx].forward_edges[edge1_edge_idx].end_idx
        } else {
            // Point is a mid-point of both lines

            // Work out where the mid-point is (use edge1 for this always: as this is supposed to be an intersection this shouldn't matter)
            // Note that if we use de Casteljau's algorithm here we get a subdivision for 'free' but organizing the code around it is painful
            let mid_point = edge1.point_at_pos(t1);

            // Add to this list of points
            let mid_point_idx = self.points.len();
            self.points.push(GraphPathPoint::new(mid_point, vec![], vec![]));

            // New point is the mid-point
            mid_point_idx
        };

        // Subdivide the edges
        let (edge1a, edge1b) = edge1.subdivide::<Curve<_>>(t1);
        let (edge2a, edge2b) = edge2.subdivide::<Curve<_>>(t2);

        // The new edges have the same kinds as their ancestors
        let edge1_kind      = self.points[edge1_idx].forward_edges[edge1_edge_idx].kind;
        let edge2_kind      = self.points[edge2_idx].forward_edges[edge2_edge_idx].kind;
        let edge1_label     = self.points[edge1_idx].forward_edges[edge1_edge_idx].label;
        let edge2_label     = self.points[edge2_idx].forward_edges[edge2_edge_idx].label;
        let edge1_end_idx   = self.points[edge1_idx].forward_edges[edge1_edge_idx].end_idx;
        let edge2_end_idx   = self.points[edge2_idx].forward_edges[edge2_edge_idx].end_idx;

        // The 'b' edges both extend from our mid-point to the existing end point (provided
        // t < 1.0)
        if !Self::t_is_one(t1) && !Self::t_is_zero(t1) {
            // If t1 is zero or one, we're not subdividing edge1
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge1_kind, edge1b.control_points(), edge1_end_idx, edge1_label));
        }
        if !Self::t_is_one(t2) && !Self::t_is_zero(t2) {
            // If t2 is zero or one, we're not subdividing edge2
            // If zero, we're just adding the existing edge again to the collision point (so we do nothing)
            self.points[collision_point].forward_edges.push(GraphPathEdge::new(edge2_kind, edge2b.control_points(), edge2_end_idx, edge2_label));
        }

        // The 'a' edges both update the initial edge, provided t is not 0
        if !Self::t_is_zero(t1) && !Self::t_is_one(t1) {
            self.points[edge1_idx].forward_edges[edge1_edge_idx].set_control_points(edge1a.control_points(), collision_point);

            // If t1 is zero, we're not subdividing edge1
            // If t1 is one this should leave the edge alone
            // If t1 is not one, then the previous step will have added the remaining part of
            // edge1 to the collision point
        }
        if !Self::t_is_zero(t2) {
            self.points[edge2_idx].forward_edges[edge2_edge_idx].set_control_points(edge2a.control_points(), collision_point);

            // If t1 is one, this should leave the edge alone
            if Self::t_is_one(t2) {
                // If t2 is one, this will have redirected the end point of t2 to the collision point: we need to move all of the edges
                let mut edge2_end_edges = vec![];
                mem::swap(&mut self.points[edge2_end_idx].forward_edges, &mut edge2_end_edges);
                self.points[collision_point].forward_edges.extend(edge2_end_edges);
            }
        }
        
        if Self::t_is_zero(t2) && collision_point != edge2_idx {
            // If t2 is zero and the collision point is not the start of edge2, then edge2 should start at the collision point instead of where it does now

            // All edges that previously went to the end point now go to the collision point
            for point in self.points.iter_mut() {
                for edge in point.forward_edges.iter_mut() {
                    if edge.end_idx == edge2_idx {
                        edge.end_idx = collision_point;
                    }
                }
            }

            // All edges that currently come from edge2 need to be moved to the collision point
            let mut edge2_edges = vec![];
            mem::swap(&mut self.points[edge2_idx].forward_edges, &mut edge2_edges);
            self.points[collision_point].forward_edges.extend(edge2_edges);
        }

        Some(collision_point)
    }

    ///
    /// Searches two ranges of points in this object and detects collisions between them, subdividing the edges
    /// and creating branch points at the appropriate places.
    /// 
    fn detect_collisions(&mut self, collide_from: Range<usize>, collide_to: Range<usize>, accuracy: f64) {
        // Put the collide_to items in a vec, so if we subdivide any of these items, we can re-read them next time through
        let collide_to = collide_to.into_iter().collect::<Vec<_>>();

        // Vector of all of the collisions found in the graph
        let mut collisions = vec![];

        // TODO: for complicated paths, maybe some pre-processing for bounding boxes to eliminate trivial cases would be beneficial for performance

        // The points that have had collisions exactly on them (we only collide them once)
        let mut collided = vec![false; self.points.len()];

        // Iterate through the edges in the 'from' range
        for src_idx in collide_from {
            for src_edge_idx in 0..self.points[src_idx].forward_edges.len() {
                // Compare to each point in the collide_to range
                for tgt_idx in collide_to.iter() {
                    for tgt_edge_idx in 0..self.points[*tgt_idx].forward_edges.len() {
                        // Don't collide edges against themselves
                        if src_idx == *tgt_idx && src_edge_idx == tgt_edge_idx { continue; }

                        // Create edge objects for each side
                        let src_curve           = GraphEdge::new(self, GraphEdgeRef { start_idx: src_idx, edge_idx: src_edge_idx, reverse: false });
                        let tgt_curve           = GraphEdge::new(self, GraphEdgeRef { start_idx: *tgt_idx, edge_idx: tgt_edge_idx, reverse: false });

                        // Quickly reject edges with non-overlapping bounding boxes
                        let src_edge_bounds     = src_curve.fast_bounding_box::<Bounds<_>>();
                        let tgt_edge_bounds     = tgt_curve.fast_bounding_box::<Bounds<_>>();
                        if !src_edge_bounds.overlaps(&tgt_edge_bounds) { continue; }

                        // Find the collisions between these two edges
                        let curve_collisions    = curve_intersects_curve_clip(&src_curve, &tgt_curve, accuracy);

                        // The are the points we need to divide the existing edges at and add branches
                        let tgt_idx = *tgt_idx;
                        for (src_t, tgt_t) in curve_collisions {
                            // A collision at t=1 is the same as a collision on t=0 on a following edge
                            // Edge doesn't actually matter for these (as the point will collide with )
                            let (src_idx, src_edge_idx, src_t) = if Self::t_is_one(src_t) {
                                (self.points[src_idx].forward_edges[src_edge_idx].end_idx, 0, 0.0)
                            } else {
                                (src_idx, src_edge_idx, src_t)
                            };

                            let (tgt_idx, tgt_edge_idx, tgt_t) = if Self::t_is_one(tgt_t) {
                                (self.points[tgt_idx].forward_edges[tgt_edge_idx].end_idx, 0, 0.0)
                            } else {
                                (tgt_idx, tgt_edge_idx, tgt_t)
                            };

                            // Allow only one collision exactly on a point
                            if Self::t_is_zero(src_t) {
                                if collided[src_idx] { 
                                    continue;
                                } else {
                                    collided[src_idx] = true;
                                }
                            }

                            if Self::t_is_zero(tgt_t) {
                                if collided[tgt_idx] { 
                                    continue;
                                } else {
                                    collided[tgt_idx] = true;
                                }
                            }

                            // Add this as a collision
                            collisions.push(((src_idx, src_edge_idx, src_t), (tgt_idx, tgt_edge_idx, tgt_t)));
                        }
                    }
                }
            }
        }

        // Apply the divisions to the edges
        while let Some(((src_idx, src_edge, src_t), (tgt_idx, tgt_edge, tgt_t))) = collisions.pop() {
            // Join the edges
            let new_mid_point = self.join_edges_at_intersection((src_idx, src_edge), (tgt_idx, tgt_edge), src_t, tgt_t);

            // Update the remainder of the collisions if any point at the source or target edge
            if let Some(new_mid_point) = new_mid_point {
                // Usually new_mid_point is a new point, but it can be an existing point in the event the collision was at an existing point on the path

                // TODO(?): this just iterates through the collisions, not clear if this will always be fast enough
                for ((ref mut other_src_idx, ref mut other_src_edge, ref mut other_src_t), (ref mut other_tgt_idx, ref mut other_tgt_edge, ref mut other_tgt_t)) in collisions.iter_mut() {
                    // If the src edge was divided...
                    if other_src_idx == &src_idx && other_src_edge == &src_edge {
                        if *other_src_t < src_t {
                            // Before the midpoint. Edge is the same, just needs to be modified.
                            *other_src_t /= src_t;
                        } else {
                            // After the midpoint. Edge needs to be adjusted. Source edge is always the first on the midpoint
                            *other_src_t     = (*other_src_t - src_t) / (1.0-src_t);
                            *other_src_idx   = new_mid_point;
                            *other_src_edge  = 0;
                        }
                    }

                    // If the target edge was divided...
                    if other_tgt_idx == &tgt_idx && other_tgt_edge == &tgt_edge {
                        if *other_tgt_t < tgt_t {
                            // Before the midpoint. Edge is the same, just needs to be modified.
                            *other_tgt_t /= tgt_t;
                        } else {
                            // After the midpoint. Edge needs to be adjusted. Target edge is always the second on the midpoint.
                            *other_tgt_t     = (*other_tgt_t - tgt_t) / (1.0-tgt_t);
                            *other_tgt_idx   = new_mid_point;
                            *other_tgt_edge  = 1;
                        }
                    }
                }
            }
        }

        // Recompute the reverse connections
        self.recalculate_reverse_connections();
    }

    ///
    /// Collides this path against another, generating a merged path
    /// 
    /// Anywhere this graph intersects the second graph, a point with two edges will be generated. All edges will be left as
    /// interior or exterior depending on how they're set on the graph they originate from.
    /// 
    /// Working out the collision points is the first step to performing path arithmetic: the resulting graph can be altered
    /// to specify edge types - knowing if an edge is an interior or exterior edge makes it possible to tell the difference
    /// between a hole cut into a shape and an intersection.
    /// 
    pub fn collide(mut self, collide_path: GraphPath<Point, Label>, accuracy: f64) -> GraphPath<Point, Label> {
        // Generate a merged path with all of the edges
        let collision_offset    = self.points.len();
        self                    = self.merge(collide_path);

        // Search for collisions between our original path and the new one
        let total_points = self.points.len();
        self.detect_collisions(0..collision_offset, collision_offset..total_points, accuracy);

        // Return the result
        self
    }

    ///
    /// Finds the exterior edge (and t value) where a line first collides with this path (closest to the line
    /// start point)
    /// 
    pub fn ray_collisions<'a, L: Line<Point=Point>>(&'a self, ray: &L) -> Vec<(GraphEdge<'a, Point, Label>, f64, f64)> {
        // We'll store the result after visiting all of the edges
        let mut collision_result = vec![];

        // Visit every edge in this graph
        for point_idx in 0..(self.points.len()) {
            for edge in self.edges_for_point(point_idx) {
                // Find out where the line collides with this edge
                let collisions = curve_intersects_ray(&edge, ray);

                for (curve_t, line_t, _collide_pos) in collisions {
                    collision_result.push((edge.clone(), curve_t, line_t));
                }
            }
        }

        collision_result.sort_by(|(_edge_a, _curve_t_a, line_t_a), (_edge_b, _curve_t_b, line_t_b)| line_t_a.partial_cmp(line_t_b).unwrap_or(Ordering::Equal));
        collision_result
    }

    ///
    /// Remove any edges marked as interior
    ///
    pub fn remove_interior_edges(&mut self) {
        for point_idx in 0..(self.points.len()) {
            self.points[point_idx].forward_edges.retain(|edge| edge.kind != GraphPathEdgeKind::Interior);
        }
    }

    ///
    /// Starting at a particular point, marks any connected edge that is not marked as exterior as interior
    ///
    fn mark_connected_edges_as_interior(&mut self, start_point: usize) {
        // Points that have been visited
        let mut visited     = vec![false; self.points.len()];

        // Stack of points waiting to be visited
        let mut to_visit    = vec![];
        to_visit.push(start_point);

        while let Some(next_point) = to_visit.pop() {
            // If we've already visited this point, mark it as visited
            if visited[next_point] { continue; }
            visited[next_point] = true;

            // Mark any uncategorised edges as interior, and visit the points they connect to
            for mut edge in self.points[next_point].forward_edges.iter_mut() {
                to_visit.push(edge.end_idx);

                if edge.kind == GraphPathEdgeKind::Uncategorised {
                    edge.kind = GraphPathEdgeKind::Interior;
                }
            }
        }
    }

    ///
    /// Given a descision function, determines which edges should be made exterior. The start edge is always made external.
    /// Any edges connected to the start edge that are not picked by the picking function are marked as interior.
    ///
    /// This can be used to implement path arithmetic algorithms by deciding which edges from the shared path should
    /// become the exterior edges of a new path.
    ///
    /// The picking function is supplied a list of possible edges and should pick the edge that represents the following
    /// exterior edge.
    ///
    pub fn classify_exterior_edges<PickEdgeFn>(&mut self, start_edge: GraphEdgeRef, pick_exterior_edge: PickEdgeFn)
    where PickEdgeFn: Fn(&Self, GraphEdge<'_, Point, Label>, &Vec<GraphEdge<'_, Point, Label>>) -> GraphEdgeRef {
        let mut current_edge_ref = start_edge;

        loop {
            // If we've arrived back at an exterior edge, we've finished marking edges as exterior
            if self.points[current_edge_ref.start_idx].forward_edges[current_edge_ref.edge_idx].kind == GraphPathEdgeKind::Exterior {
                break;
            }
            
            // Mark the current edge as exterior
            self.points[current_edge_ref.start_idx].forward_edges[current_edge_ref.edge_idx].kind = GraphPathEdgeKind::Exterior;

            // Get the end of the current edge
            let end_point_idx = if current_edge_ref.reverse {
                current_edge_ref.start_idx 
            } else {
                self.points[current_edge_ref.start_idx].forward_edges[current_edge_ref.edge_idx].end_idx
            };

            // Fetch the next external edge using the decision function (pick_external_edge)
            let next_edge = {
                // If there's only one possible edge to follow then always follow that, otherwise ask the picking function
                if !current_edge_ref.reverse && self.points[end_point_idx].forward_edges.len() == 1 {
                    // Only one edge in the current direction: no intersection to decide upon
                    GraphEdgeRef {
                        start_idx:  end_point_idx,
                        edge_idx:   0,
                        reverse:    false
                    }
                } else if current_edge_ref.reverse && self.points[end_point_idx].connected_from.len() == 1 {
                    // Only one edge in the current direction: no intersection to decide upon
                    self.reverse_edges_for_point(end_point_idx).nth(0).unwrap().into()
                } else {
                    let last_edge = GraphEdge::new(self, current_edge_ref);

                    // Gather the uncategorised edges for the current point
                    // The edge we just visited will just have been marked as exterior so it will be excluded here
                    // Also, if we revisit a point we'll only ask the algorithm to pick from the remaining edges
                    let edges = self.edges_for_point(end_point_idx)
                        .chain(self.reverse_edges_for_point(end_point_idx))
                        .filter(|edge| edge.kind() == GraphPathEdgeKind::Uncategorised)
                        .collect();

                    // Pass to the selection function to pick the next edge we go to
                    pick_exterior_edge(self, last_edge, &edges)
                }
            };

            // Set the current edge
            current_edge_ref = next_edge;
        }

        // Go around the loop again and mark any edges still uncategorized as interior
        self.mark_connected_edges_as_interior(current_edge_ref.start_idx);
    }

    ///
    /// Finds the exterior edges and turns them into a series of paths
    ///
    pub fn exterior_paths<POut: BezierPathFactory<Point=Point>>(&self) -> Vec<POut> {
        let mut exterior_paths = vec![];

        // Array of visited points
        let mut visited = vec![false; self.points.len()];

        for point_idx in 0..(self.points.len()) {
            // Ignore this point if we've already visited it as part of a path
            if visited[point_idx] {
                continue;
            }

            // Find the first exterior point
            let exterior_edge = self.edges_for_point(point_idx)
                .filter(|edge| edge.kind() == GraphPathEdgeKind::Exterior)
                .nth(0);

            if let Some(exterior_edge) = exterior_edge {
                // Follow the edge around to generate the path (we expect exterior edges to form a complete path)
                let start_point         = exterior_edge.start_point();
                let mut current_edge    = exterior_edge;
                let mut path_points     = vec![];

                loop {
                    let current_point_idx = current_edge.start_point_index();

                    // Stop once we reach a point we've already visited
                    if visited[current_point_idx] {
                        break;
                    }

                    // Mark the current point as visited
                    visited[current_point_idx] = true;

                    // Add the next edge to the path
                    let (cp1, cp2) = current_edge.control_points();
                    path_points.push((cp1, cp2, current_edge.end_point()));

                    // Find the next edge (next exterior edge in either direction that is not back the way we came)
                    let next_point_idx  = current_edge.end_point_index();
                    let next_edge       = self.edges_for_point(next_point_idx)
                        .chain(self.reverse_edges_for_point(next_point_idx))
                        .filter(|edge| edge.end_point_index() != current_point_idx)
                        .filter(|edge| edge.kind() == GraphPathEdgeKind::Exterior)
                        .nth(0);

                    if let Some(next_edge) = next_edge {
                        // Move on to the next point on this path
                        current_edge = next_edge;
                    } else {
                        // Partial path
                        // TODO: or, reversal of direction...
                        break;
                    }
                }

                // Turn into a path
                let path = POut::from_points(start_point, path_points);
                exterior_paths.push(path);
            }
        }

        // Return the set of exterior paths
        exterior_paths
    }
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a, Label: 'a> {
    /// The graph that this point is for
    graph: &'a GraphPath<Point, Label>,

    /// A reference to the edge this point is for
    edge: GraphEdgeRef
}

impl<'a, Point: 'a, Label: 'a+Copy> GraphEdge<'a, Point, Label> {
    ///
    /// Creates a new graph edge (with an edge kind of 'exterior')
    /// 
    #[inline]
    fn new(graph: &'a GraphPath<Point, Label>, edge: GraphEdgeRef) -> GraphEdge<'a, Point, Label> {
        GraphEdge {
            graph:  graph,
            edge:   edge
        }
    }

    ///
    /// Returns true if this edge is going backwards around the path
    ///
    #[inline]
    pub fn is_reversed(&self) -> bool {
        self.edge.reverse
    }

    ///
    /// Retrieves a reference to the edge in the graph
    ///
    #[inline]
    fn edge<'b>(&'b self) -> &'b GraphPathEdge<Point, Label> {
        &self.graph.points[self.edge.start_idx].forward_edges[self.edge.edge_idx]
    }

    ///
    /// Returns if this is an interior or an exterior edge in the path
    /// 
    pub fn kind(&self) -> GraphPathEdgeKind {
        self.edge().kind
    }

    ///
    /// Returns the index of the start point of this edge
    /// 
    #[inline]
    pub fn start_point_index(&self) -> usize {
        if self.edge.reverse {
            self.edge().end_idx
        } else {
            self.edge.start_idx
        }
    }

    ///
    /// Returns the index of the end point of this edge
    /// 
    #[inline]
    pub fn end_point_index(&self) -> usize {
        if self.edge.reverse {
            self.edge.start_idx
        } else {
            self.edge().end_idx
        }
    }

    ///
    /// The label attached to this edge
    ///
    #[inline]
    pub fn label(&self) -> Label {
        self.edge().label
    }
}

impl<'a, Point: 'a+Coordinate, Label: 'a> Geo for GraphEdge<'a, Point, Label> {
    type Point = Point;
}

impl<'a, Point: 'a+Coordinate, Label: 'a+Copy> BezierCurve for GraphEdge<'a, Point, Label> {
    ///
    /// The start point of this curve
    /// 
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.graph.points[self.start_point_index()].position.clone()
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.graph.points[self.end_point_index()].position.clone()
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        let edge = self.edge();

        if self.edge.reverse {
            (edge.cp2.clone(), edge.cp1.clone())
        } else {
            (edge.cp1.clone(), edge.cp2.clone())
        }
    }
}

///
/// A GraphEdgeRef can be created from a GraphEdge in order to release the borrow
///
impl<'a, Point: 'a+Coordinate, Label: 'a+Copy> From<GraphEdge<'a, Point, Label>> for GraphEdgeRef {
    fn from(edge: GraphEdge<'a, Point, Label>) -> GraphEdgeRef {
        edge.edge
    }
}

///
/// A GraphEdgeRef can be created from a GraphEdge in order to release the borrow
///
impl<'a, 'b, Point: 'a+Coordinate, Label: 'a+Copy> From<&'b GraphEdge<'a, Point, Label>> for GraphEdgeRef {
    fn from(edge: &'b GraphEdge<'a, Point, Label>) -> GraphEdgeRef {
        edge.edge
    }
}

impl<'a, Point: fmt::Debug, Label: 'a+Copy> fmt::Debug for GraphEdge<'a, Point, Label> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} -> {:?} ({:?} -> {:?} ({:?}, {:?}))", self.edge.start_idx, self.edge().end_idx, self.graph.points[self.edge.start_idx].position, self.graph.points[self.edge().end_idx].position, self.edge().cp1, self.edge().cp2)
    }
}