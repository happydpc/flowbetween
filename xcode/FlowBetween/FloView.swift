//
//  FloView.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 03/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

///
/// Class used to manage a view in FlowBetween
///
public class FloView : NSObject {
    /// The view that this will display
    fileprivate var _view: NSView!;
    
    /// The layout bounds of this view
    fileprivate var _bounds: Bounds;
    
    override init() {
        _bounds = Bounds(
            x1: Position.Start,
            y1: Position.Start,
            x2: Position.End,
            y2: Position.End
        );
    }
    
    ///
    /// The view that this is managing
    ///
    public var view: NSView! {
        get { return _view; }
    }
    
    ///
    /// Creates an empty view
    ///
    @objc public func setupAsEmpty() {
        // Just a standard NSView
        _view = NSView.init();
        
        // Create core animation views wherever possible
        _view.wantsLayer = true;
    }
    
    ///
    /// Removes this view from its superview
    ///
    @objc public func viewRemoveFromSuperview() {
        _view?.removeFromSuperview();
    }
    
    ///
    /// Adds a subview to this view
    ///
    @objc(viewAddSubView:) public func viewAddSubView(subview: FloView!) {
        if let subview = subview._view {
            _view?.addSubview(subview);
        }
    }
    
    ///
    /// Sets the position of a side of the view
    ///
    func set_side_position(_ side: Int32, _ position: Position) {
        switch (side) {
        case 0: _bounds.x1 = position;
        case 1: _bounds.y1 = position;
        case 2: _bounds.x2 = position;
        case 3: _bounds.y2 = position;
        default: break;
        }
    }
    
    @objc(viewSetSide:at:) public func viewSetSide(side: Int32, at: Float32) {
        set_side_position(side, Position.At(at));
    }

    @objc(viewSetSide:offset:) public func viewSetSide(side: Int32, offset: Float32) {
        set_side_position(side, Position.Offset(offset));
    }

    @objc(viewSetSide:stretch:) public func viewSetSide(side: Int32, stretch: Float32) {
        set_side_position(side, Position.Stretch(stretch));
    }

    @objc(viewSetSideAtStart:) public func viewSetSideAtStart(side: Int32) {
        set_side_position(side, Position.Start);
    }

    @objc(viewSetSideAtEnd:) public func viewSetSideAtEnd(side: Int32) {
        set_side_position(side, Position.End);
    }

    @objc(viewSetSideAfter:) public func viewSetSideAfter(side: Int32) {
        set_side_position(side, Position.After);
    }
}
