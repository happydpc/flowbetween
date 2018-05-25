use super::db_enum::*;
use super::flo_store::*;
use super::vector_layer::*;

use animation::*;

use rusqlite::*;
use std::sync::*;
use std::time::Duration;
use std::collections::HashMap;

///
/// Core data structure used by the animation database
/// 
pub struct AnimationDbCore<TFile: FloFile+Send> {
    /// The database connection
    pub db: TFile,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<Error>,
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Performs an edit on this core if the failure condition is clear
    /// 
    pub fn edit<TEdit: FnOnce(&mut TFile) -> Result<()>>(&mut self, edit: TEdit) {
        // Perform the edit if there is no failure
        if self.failure.is_none() {
            self.failure = edit(&mut self.db).err();
        }
    }

    ///
    /// Creates a new vector element in an animation DB core, leaving the element ID, key frame ID and time pushed on the DB stack
    ///
    /// The element is created without its associated data.
    ///
    fn create_new_element(db: &mut TFile, layer_id: i64, when: Duration, element: &PaintEdit) -> Result<()> {
        if let ElementId::Assigned(assigned_id) = element.id() {
            db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PushNearestKeyFrame(when),
                DatabaseUpdate::PushVectorElementType(VectorElementType::from(element), when),
                DatabaseUpdate::PushElementAssignId(assigned_id)
            ])?;
        } else {
            db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PushNearestKeyFrame(when),
                DatabaseUpdate::PushVectorElementType(VectorElementType::from(element), when)
            ])?;
        }

        Ok(())
    }

    ///
    /// Writes a brush properties element to the database (popping the element ID)
    ///
    fn create_brush_properties(db: &mut TFile, properties: BrushProperties) -> Result<()> {
        AnimationDbCore::insert_brush_properties(db, &properties)?;

        // Create the element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushPropertiesElement
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush definition element to the database (popping the element ID)
    ///
    fn create_brush_definition(db: &mut TFile, definition: BrushDefinition, drawing_style: BrushDrawingStyle) -> Result<()> {
        // Create the brush definition
        AnimationDbCore::insert_brush(db, &definition)?;

        // Insert the properties for this element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::from(&drawing_style))
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush stroke to the database (popping the element ID)
    ///
    fn create_brush_stroke(db: &mut TFile, brush_stroke: Arc<Vec<RawPoint>>) -> Result<()> {
        // TODO: we need to convert the raw points to brush points here

        db.update(vec![
            DatabaseUpdate::PopBrushPoints(brush_stroke)
        ])?;

        Ok(())
    }

    ///
    /// Adds a new vector element to a vector layer
    /// 
    fn paint_vector_layer(&mut self, layer_id: i64, when: Duration, new_element: PaintEdit) -> Result<()> {
        use animation::PaintEdit::*;

        // Update the state of this object based on the element
        match new_element {
            SelectBrush(_id, ref brush_definition, drawing_style)   => {
                // TODO!
                // self.active_brush = Some((when, create_brush_from_definition(brush_definition, drawing_style)));
            },

            _ => ()
        }

        // Create a new element
        Self::create_new_element(&mut self.db, layer_id, when, &new_element)?;

        // Record the details of the element itself
        match new_element {
            SelectBrush(_id, brush_definition, drawing_style)   => Self::create_brush_definition(&mut self.db, brush_definition, drawing_style)?,
            BrushProperties(_id, brush_properties)              => Self::create_brush_properties(&mut self.db, brush_properties)?,
            BrushStroke(_id, brush_stroke)                      => Self::create_brush_stroke(&mut self.db, brush_stroke)?,
        }

        // create_new_element pushes an element ID, a key frame ID and a time. The various element actions pop the element ID so we need to pop the frame ID and time
        self.db.update(vec![
            DatabaseUpdate::Pop,
            DatabaseUpdate::Pop
        ])?;

        Ok(())
    }

    ///
    /// Performs a layer edit to a vector layer
    /// 
    pub fn edit_vector_layer(&mut self, layer_id: i64, edit: LayerEdit) -> Result<()> {
        use self::LayerEdit::*;

        // Note that we can't access the core at this point (the database implies that the core is already in use)

        match edit {
            AddKeyFrame(when) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopAddKeyFrame(when)
                ])?;
            },

            RemoveKeyFrame(when) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopRemoveKeyFrame(when)
                ])?;
            },

            Paint(when, edit) => {
                self.paint_vector_layer(layer_id, when, edit)?;
            }
        }

        Ok(())
    }

    ///
    /// Performs an edit on this core
    /// 
    pub fn perform_edit(&mut self, edit: AnimationEdit) -> Result<()> {
        use self::AnimationEdit::*;

        match edit {
            SetSize(width, height) => {
                self.db.update(vec![
                    DatabaseUpdate::UpdateCanvasSize(width, height)
                ])?;
            },

            AddNewLayer(new_layer_id) => {
                // Create a layer with the new ID
                self.db.update(vec![
                    DatabaseUpdate::PushLayerType(LayerType::Vector),
                    DatabaseUpdate::PushAssignLayer(new_layer_id),
                    DatabaseUpdate::Pop
                ])?;
            },

            RemoveLayer(old_layer_id) => {
                // Delete this layer
                self.db.update(vec![
                    DatabaseUpdate::PushLayerForAssignedId(old_layer_id),
                    DatabaseUpdate::PopDeleteLayer
                ])?;
            },

            Layer(layer_id, layer_edit) => {
                self.edit_vector_layer(layer_id as i64, layer_edit);
            },

            Element(id, when, edit) => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
