use super::ink::*;
use super::super::tools::*;
use super::super::model::*;

use binding::*;
use animation::*;

use futures::*;
use std::sync::*;

///
/// The Eraser tool (Erasers control points of existing objects)
/// 
pub struct Eraser { 
    ink: Ink
}

impl Eraser {
    ///
    /// Creates a new instance of the Eraser tool
    /// 
    pub fn new() -> Eraser {
        Eraser {
            ink: Ink::new()
        }
    }
}

impl<Anim: Animation+'static> Tool<Anim> for Eraser {
    type ToolData   = InkData;
    type Model      = InkModel;

    fn tool_name(&self) -> String { "Eraser".to_string() }

    fn image_name(&self) -> String { "eraser".to_string() }

    fn create_model(&self) -> InkModel {
        let mut model = InkModel::new();

        model.size.set(20.0);

        model
    }

    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &InkModel) -> Box<Stream<Item=ToolAction<InkData>, Error=()>+Send> {
        // Fetch the brush properties
        let brush_properties    = tool_model.brush_properties.clone();
        let selected_layer      = flo_model.timeline().selected_layer.clone();

        // Create a computed binding that generates the data for the brush
        let ink_data            = computed(move || {
            InkData {
                brush:              BrushDefinition::Ink(InkDefinition::default()),
                brush_properties:   brush_properties.get(),
                selected_layer:     selected_layer.get().unwrap_or(0)
            }
        });

        // Turn the computed values into a stream and update the brush whenever the values change
        Box::new(follow(ink_data).map(|ink_data| ToolAction::Data(ink_data)))
    }

    fn actions_for_input<'a>(&'a self, data: Option<Arc<InkData>>, input: Box<'a+Iterator<Item=ToolInput<InkData>>>) -> Box<'a+Iterator<Item=ToolAction<InkData>>> {
        use self::ToolAction::*;
        use self::BrushPreviewAction::*;

        let ink: &Tool<Anim, ToolData=InkData, Model=InkModel> = &self.ink;

        // As for the ink tool, except that we use the eraser drawing style
        let actions = ink.actions_for_input(data, input)
            .map(|action| {
                match action {
                    BrushPreview(BrushDefinition(brush, BrushDrawingStyle::Draw)) => BrushPreview(BrushDefinition(brush, BrushDrawingStyle::Erase)),
                    
                    other => other
                }
            });

        Box::new(actions)
    }
}
