use super::*;

use std::time::Duration;

///
/// Represents a vector layer in a SQLite database
/// 
pub struct SqliteVectorLayer {
    /// The ID that was assigned to this layer
    assigned_id: u64,

    /// The ID of this layer
    layer_id: i64,

    /// The type of this layer
    _layer_type: i64,

    /// Database core
    core: Arc<Desync<AnimationDbCore>>
}

///
/// Enumeration values for the vector elements
///
pub struct VectorElementEnumValues {
    pub brush_definition:   i32,
    pub brush_properties:   i32,
    pub brush_stroke:       i32
}

impl VectorElementEnumValues {
    ///
    /// Reads the enum values
    ///
    pub fn new(sqlite: &Connection) -> Result<VectorElementEnumValues> {
        // Define a function to read values
        let read_value = |name: &str| {
            sqlite.query_row(
                "SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = \"VectorElementType\" AND ApiName = ?",
                &[&name],
                |row| row.get(0)
            )
        };

        // Read the values for the element values
        let brush_definition    = read_value("BrushDefinition")?;
        let brush_properties    = read_value("BrushProperties")?;
        let brush_stroke        = read_value("BrushStroke")?;

        // Turn into an enum values object
        Ok(VectorElementEnumValues {
            brush_definition:   brush_definition,
            brush_properties:   brush_properties,
            brush_stroke:       brush_stroke
        })
    }

    ///
    /// Retrieves the type ID for a vector element
    ///
    fn get_vector_type(&self, vector: &Vector) -> i32 {
        use animation::Vector::*;

        match vector {
            &BrushDefinition(_) => self.brush_definition,
            &BrushProperties(_) => self.brush_properties,
            &BrushStroke(_)     => self.brush_stroke
        }
    }
}

impl AnimationDb {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn get_layer_with_id(&self, assigned_id: u64) -> Option<SqliteVectorLayer> {
        SqliteVectorLayer::from_assigned_id(&self.core, assigned_id)
    }
}

impl SqliteVectorLayer {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn from_assigned_id(core: &Arc<Desync<AnimationDbCore>>, assigned_id: u64) -> Option<SqliteVectorLayer> {
        // Query for the 'real' layer ID
        let layer: Result<(i64, i64)> = core.sync(|core| {
            // Fetch the layer data (we need the 'real' ID here)
            let mut get_layer = core.sqlite.prepare(
                "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                        INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                        WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?;")?;
            
            let layer = get_layer.query_row(
                &[&core.animation_id, &(assigned_id as i64)],
                |layer| {
                    (layer.get(0), layer.get(1))
                }
            )?;

            Ok(layer)
        });

        // If the layer exists, create a SqliteVectorLayer
        layer.ok()
            .map(|(layer_id, layer_type)| {
                SqliteVectorLayer {
                    assigned_id:    assigned_id,
                    layer_id:       layer_id,
                    _layer_type:    layer_type,
                    core:           Arc::clone(core)
                }
            })
    }
}

impl SqliteVectorLayer {
    ///
    /// Performs an async operation on the database
    /// 
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore) -> Result<()>>(&self, action: TFn) {
        self.core.async(move |core| {
            // Only run the function if there has been no failure
            if core.failure.is_none() {
                // Run the function and update the error status
                let result      = action(core);
                core.failure    = result.err();
            }
        })
    }
}

impl Layer for SqliteVectorLayer {
    fn id(&self) -> u64 {
        self.assigned_id
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![LayerEditType::Vector]
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        unimplemented!()
    }

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        let keyframes = self.core.sync(|core| {
            // Query for the microsecond times from the database
            let mut get_key_frames  = core.sqlite.prepare("SELECT AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ?")?;
            let key_frames          = get_key_frames.query_map(
                &[&self.layer_id],
                |time| { let i: i64 = time.get(0); i }
            )?;

            // Convert to micros to produce the final result
            let key_frames: Vec<Duration> = key_frames
                .map(|micros| AnimationDbCore::from_micros(micros.unwrap()))
                .collect();
            
            Ok(key_frames)
        });

        // Turn into an iterator
        let keyframes = keyframes.unwrap_or_else(|_: Error| vec![]);
        let keyframes = Box::new(keyframes.into_iter());

        keyframes
    }

    fn add_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            let mut insert_key_frame    = core.sqlite.prepare("INSERT INTO Flo_LayerKeyFrame (LayerId, AtTime) VALUES (?, ?)")?;
            let at_time                 = AnimationDbCore::get_micros(&when);

            insert_key_frame.execute(&[&layer_id, &at_time])?;

            Ok(())
        });
    }

    fn remove_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            let mut insert_key_frame    = core.sqlite.prepare("DELETE FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime = ?")?;
            let at_time                 = AnimationDbCore::get_micros(&when);

            insert_key_frame.execute(&[&layer_id, &at_time])?;

            Ok(())
        });
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Reader<'a, VectorLayer>> {
        let vector_layer = self as &VectorLayer;

        Some(Reader::new(vector_layer))
    }

    fn edit_vectors<'a>(&'a mut self) -> Option<Editor<'a, VectorLayer>> {
        let vector_layer = self as &mut VectorLayer;
 
        Some(Editor::new(vector_layer))
    }
}

impl SqliteVectorLayer {
    ///
    /// Creates a new vector element in an animation DB core
    ///
    /// The element is created without its associated data.
    ///
    fn create_new_element(core: &mut AnimationDbCore, layer_id: i64, when: Duration, element: &Vector) -> i64 {
        let mut element_id: i64 = -1;

        // Ensure that the vector enum is populated for the edit
        if core.vector_enum.is_none() {
            core.vector_enum = Some(VectorElementEnumValues::new(&core.sqlite).unwrap());
        }

        core.edit(move |sqlite, animation_id, core| {
            // Want the list of enumeration values for the vector elements
            let vector_enum = core.vector_enum.as_ref().unwrap();

            // Convert when to microseconds
            let when = AnimationDbCore::get_micros(&when);

            // SQL statements: find the frame that this time represents and insert a new element
            // We'd like to preserve these statments between calls but rusqlite imposes lifetime limits that 
            // force us to use prepare_cached (or muck around with reference objects).
            let mut get_key_frame   = sqlite.prepare_cached("SELECT TOP 1 KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime <= ? ORDER BY AtTime DESC")?;
            let mut create_element  = sqlite.prepare_cached("INSERT INTO Flo_VectorElement (KeyFrameId, VectorElementType, AtTime) VALUES (?, ?, ?)")?;

            // Find the keyframe that we can add this element to
            let (keyframe, keyframe_time): (i64, i64) = get_key_frame.query_row(&[&layer_id, &when], |row| (row.get(0), row.get(1)))?;

            // Fetch the element type
            let element_type = vector_enum.get_vector_type(element);

            // Create the vector element
            element_id = create_element.insert(&[&keyframe, &element_type, &(when-keyframe_time)])?;

            Ok(())
        });

        // Return the element ID
        element_id
    }
}

impl VectorLayer for SqliteVectorLayer {
    fn add_element(&mut self, when: Duration, new_element: Vector) {
        let layer_id = self.layer_id;

        self.core.async(move |core| {
            use animation::Vector::*;

            // Create a new element
            let element_id = Self::create_new_element(core, layer_id, when, &new_element);

            // Record the details of the element itself
            match new_element {
                BrushDefinition(brush_definition)   => unimplemented!(),
                BrushProperties(brush_properties)   => unimplemented!(),
                BrushStroke(brush_stroke)           => unimplemented!(),
            }
        });
    }

    fn active_brush(&self, when: Duration) -> Arc<Brush> {
        unimplemented!()
    }
}
